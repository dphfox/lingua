// Previously, this was a crate-wide conditional compile that'd simply not
// export any of lingua's members. However, this was extremely confusing for
// projects that weren't properly configured to target WebAssembly, because it'd
// look like the import was supposed to work. To avoid confusion, a compile time
// error is now thrown. Users can still explicitly declare that they want to
// compile for other targets without Lingua, but the default is now to notify
// users of the configuration error, with the expectation being that most people
// are only compiling for WebAssembly.
#[cfg(not(target_arch = "wasm32"))]
compile_error!(
	"Lingua only works with WebAssembly targets. \
	Either configure your project's target triple, \
	or conditionally depend on Lingua.");

use std::{cell::RefCell, collections::HashMap, num::Wrapping, panic::{catch_unwind, AssertUnwindSafe, UnwindSafe}};

use serde::{de::DeserializeOwned, Serialize};
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
	impl Luau {
		pub fn interpret(
			value: u8
		) -> Option<Self> {
			match value {
				0 => Some(Luau::Success),
				1 => Some(Luau::UncaughtErrorAtFfiBoundary),
				2 => Some(Luau::LuauApiNotReady),
				_ => None
			}
		}
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
			log::error!("[lingua] panic at ffi boundary\n\ncaused by:\n{e:?}");
			return_codes::Rust::PanicAtFfiBoundary
		}
	}
}

extern "C" {
	/// The Rust side calls this function when it sends a JSON string. It's
	/// called with the handle that it generated, the pointer to the string, and
	/// the length of that string. To minimise the chance of errors at the FFI
	/// boundary, the string is saved without decoding the data or invoking any
	/// user callbacks.
	#[must_use]
	fn lingua_send_json_to_luau(
		rust_handle: u32,
		ptr: *mut u8,
		len: u32
	) -> u8;
}

/// The Luau side calls this function to initiate sending a JSON string. It's
/// called with the handle that it generated and the length of the string that
/// it would like to transfer. The Rust side is expected to reserve space for
/// the string and return a pointer to this reserved space, with a null pointer
/// representing a failure to allocate space.
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

/// The Luau side is expected to call this function once it has finished writing
/// to space previously allocated for the transfer of JSON data. This signals to
/// the Rust side that it is safe to access the data.
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
			match data {
				JustReceivedJson::AllocatedOnly(str) =>
					api_state.just_received_json.insert(
						luau_handle,
						JustReceivedJson::Ready(str)
					),
				JustReceivedJson::Ready(_) => 
					panic!(
						"[lingua] luau handle {luau_handle} was already sent - \
						handles are single use and should only be sent once"
					)
			}
			
		});
	}) as u8
}

/// When data is sent from Rust, this opaque handle is generated to concisely
/// refer to that data. This handle must always be sent to the Luau side; this
/// is done by converting it into a `u32` and sending it through an `extern fn`.
#[repr(transparent)]
pub struct DataFromRustHandle(u32);
impl From<DataFromRustHandle> for u32 {
	fn from(
		value: DataFromRustHandle
	) -> Self {
		value.0
	}
}

/// When data is sent from Luau, this opaque handle is generated to concisely
/// refer to that data. This handle is received across the FFI boundary by
/// obtaining it through an `extern fn` and converting it from a `u32`.
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
	CStringError,
	#[error("the luau side encountered an error at the ffi boundary")]
	LuauErrorAtFfiBoundaryError,
	#[error("the luau side is not initialised yet, so cannot handle incoming data")]
	LuauApiNotReadyError,
	#[error("the luau side indicated an error, but it's not a known error code")]
	LuauUnknownError
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

/// Sends some data to the Luau side. An opaque `DataFromRustHandle` is
/// returned; you are always expected to send this handle to the Luau side.
pub fn send_to_luau<S: Serialize>(
	data: &S
) -> Result<DataFromRustHandle, SendToLuauError> {
	API_STATE.with_borrow_mut(|api_state: &mut ApiState| {
		let mut str = serde_json::to_string(data).map_err(|e| SendToLuauError::SerdeError(e))?;
		let rust_handle = api_state.next_rust_handle.0;
		api_state.next_rust_handle += 1;
		let len = str.len() as u32;
		let ptr = str.as_mut_ptr();
		let status = return_codes::Luau::interpret(unsafe {
			lingua_send_json_to_luau(rust_handle, ptr, len)
		});
		match status {
			Some(status) => match status {
				return_codes::Luau::Success => 
					Ok(DataFromRustHandle(rust_handle)),
				return_codes::Luau::UncaughtErrorAtFfiBoundary => 
					Err(SendToLuauError::LuauErrorAtFfiBoundaryError),
				return_codes::Luau::LuauApiNotReady => 
					Err(SendToLuauError::LuauApiNotReadyError)
			},
			None => Err(SendToLuauError::LuauUnknownError)
		}
	})
}

/// Receives some data from the Luau side. You need to generate and send a
/// `DataFromLuauHandle` yourself from within Luau.
pub fn receive_from_luau<D: DeserializeOwned>(
	luau_handle: DataFromLuauHandle
) -> Result<D, ReceiveFromLuauError> {
	API_STATE.with_borrow_mut(|api_state: &mut ApiState| {
		let str = api_state.just_received_json.remove(&luau_handle.0);
		match str {
			None => 
				Err(ReceiveFromLuauError::NoDataError),
			Some(JustReceivedJson::AllocatedOnly(_)) =>
				Err(ReceiveFromLuauError::AllocatedOnlyError),
			Some(JustReceivedJson::Ready(str)) =>
				serde_json::from_str(&str)
				.map_err(|e| ReceiveFromLuauError::SerdeError(e))
		}
	})
}