use std::{cell::RefCell, collections::HashMap, num::Wrapping, panic::{catch_unwind, AssertUnwindSafe, UnwindSafe}};

use serde::Serialize;
use thiserror::Error;

/// The return values of Lingua FFI functions are used to indicate whether the
/// FFI call was successful. Note that this has nothing to do with the specific
/// operation - it's specifically used to communicate low-level failures.
/// Returned data or operation-specific errors should be sent some other way.
/// The only exception are memory allocations, which must return a pointer for
/// pragmatic reasons - in this case, the null pointer represents failure.
mod return_codes {
	#[repr(u8)]
	pub enum Luau {
		/// The FFI call was successfully handled without an unexpected error.
		Success = 0,
		/// A Luau error occured outside of a protected block in an FFI call.
		UncaughtErrorAtFfiBoundary = 1,
		/// An FFI call was received before Lingua's Luau API knew about the
		/// module that was sending the data.
		LuauApiNotReady = 2
	}
	
	#[repr(u8)]
	pub enum Rust {
		/// The FFI call was successfully handled without panicking.
		Success = 0,
		/// A Rust panic unwound up to the FFI boundary.
		PanicAtFfiBoundary = 1
	}
}

enum JustReceivedJson {
	AllocatedOnly(String),
	Ready(String)
}

impl JustReceivedJson {
	pub fn ready(self) -> Self {
		match self {
			Self::AllocatedOnly(str) => Self::Ready(str),
			x => x
		}
	}
}

/// The internal state used by Lingua's Rust-side API.
/// This should be treated as a singleton representing the whole module.
struct ApiState {
	/// When the Luau side sends JSON strings, they're stored here, indexed by
	/// which handle the Luau side decided to use.
	just_received_json: HashMap<u32, JustReceivedJson>,
	/// To uniquely identify JSON strings the Rust side sends to the Luau side,
	/// this handle is incremented. Handles generated on the Rust side may
	/// collide with handles generated on the Luau side because no
	/// synchronisation is done.
	next_rust_handle: Wrapping<u32>
}

impl ApiState {
	pub fn new() -> Self {
		Self {
			just_received_json: HashMap::new(),
			next_rust_handle: Wrapping(0)
		}
	}
}

thread_local! {
    static API_STATE: RefCell<ApiState> = RefCell::new(ApiState::new());
}

/// This is used in FFI functions to ensure that panics do not unwind across the
/// FFI boundary. Panics log at error level and return `PanicAtFfiBoundary`.
/// Otherwise, the `Success` code is returned.
fn ffi_panic_boundary<Func: FnOnce() -> () + UnwindSafe>(
	f: Func
) -> return_codes::Rust {
	match catch_unwind(f) {
		Ok(()) => return_codes::Rust::Success,
		Err(e) => {
			log::error!("[lingua] panic at ffi boundary: {e:?}");
			return_codes::Rust::PanicAtFfiBoundary
		}
	}
}

extern "C" {
	fn lingua_send_json_to_luau(
		rust_handle: u32,
		ptr: *mut u8,
		len: u32
	) -> ();
}

#[no_mangle]
extern "C" fn lingua_send_json_to_rust_alloc(
	luau_handle: u32,
	len: u32
) -> *mut u8 {
	let mut return_ptr = 0 as *mut u8;
	ffi_panic_boundary(AssertUnwindSafe(|| {
		// Fill the string with something that's easy to recognise if part of 
		// the string remains uninitialised.
		let mut str = String::from_iter((0..len).map(|_| '£'));
		assert!(
			str.capacity() >= len as usize,
			"[lingua] sanity check failed: send_json_to_rust_alloc string does \
			not have the right capacity for the requested data length"
		);
		let str_ptr = str.as_mut_ptr();
		API_STATE.with_borrow_mut(|api_state: &mut ApiState| {
			assert!(
				!api_state.just_received_json.contains_key(&luau_handle),
				"[lingua] luau handle {luau_handle} is already in use - \
				ensure you're reading all data sent to the rust side"
			);
			api_state.just_received_json.insert(
				luau_handle, 
				JustReceivedJson::AllocatedOnly(str)
			);
		});
		return_ptr = str_ptr;
	}));
	return_ptr
}

#[no_mangle]
extern "C" fn lingua_send_json_to_rust(
	luau_handle: u32
) -> u8 {
	ffi_panic_boundary(|| {
		API_STATE.with_borrow_mut(|api_state: &mut ApiState| {
			let Some(data) = api_state.just_received_json.remove(&luau_handle) else {
				panic!(
					"[lingua] luau handle {luau_handle} has no data - handles \
					should be generated on the sending side and are single use"
				);
			};
			api_state.just_received_json.insert(luau_handle, data.ready());
		});
	}) as u8
}

#[repr(transparent)]
pub struct DataFromRustHandle(u32);
impl From<DataFromRustHandle> for u32 {
	fn from(
		value: DataFromRustHandle
	) -> Self {
		value.0
	}
}

#[repr(transparent)]
pub struct DataFromLuauHandle(u32);
impl From<u32> for DataFromLuauHandle {
	fn from(value: u32) -> Self {
		Self(value)
	}
}

#[derive(Debug, Error)]
pub enum SendToLuauError {
	#[error("error while serializing")]
	SerdeError(serde_json::Error),
	#[error("could not convert serialized form to C string")]
	CStringError
}

#[derive(Debug, Error)]
pub enum ReceiveFromLuauError {
	#[error("error while deserializing")]
	SerdeError(serde_json::Error),
	#[error("luau handle has no data")]
	NoDataError,
	#[error("luau handle has allocated memory but has not submitted data yet")]
	AllocatedOnlyError
}	

pub fn send_to_luau<S: Serialize>(
	data: &S
) -> Result<DataFromRustHandle, SendToLuauError> {
	API_STATE.with_borrow_mut(|api_state: &mut ApiState| {
		let mut str = serde_json::to_string(data).map_err(|e| SendToLuauError::SerdeError(e))?;
		let rust_handle = api_state.next_rust_handle.0;
		api_state.next_rust_handle += 1;
		let len = str.len() as u32;
		let ptr = str.as_mut_ptr();
		unsafe {
			lingua_send_json_to_luau(rust_handle, ptr, len);
		}
		Ok(DataFromRustHandle(rust_handle))
	})
}