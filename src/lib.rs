use std::ffi::CString;

use serde::{de::DeserializeOwned, Serialize};
use thiserror::Error;

mod ffi;

#[derive(Debug, Error)]
pub enum DeserializeError {
	#[error("error while deserializing")]
	SerdeError(serde_json::Error),
	#[error("invalid pointer input")]
	InvalidPointerError
}

pub fn deserialize<D: DeserializeOwned>(
	ptr: *mut u8
) -> Result<D, DeserializeError> {
	ffi::FOREIGN_STRING_ALLOCS.with_borrow_mut(|allocs| {
		let Some(str) = allocs.get(ptr) else {return Err(DeserializeError::InvalidPointerError)};
		serde_json::from_str(str).map_err(|e| DeserializeError::SerdeError(e))
	})
}

#[derive(Debug, Error)]
pub enum SerializeError {
	#[error("error while deserializing")]
	SerdeError(serde_json::Error),
	#[error("could not convert serialized form to C string")]
	CStringError
}

pub fn serialize<S: Serialize>(
	data: &S
) -> Result<*mut u8, SerializeError> {
	let str = serde_json::to_string(data).map_err(|e| SerializeError::SerdeError(e))?;
	let c_str = CString::new(str).map_err(|_| SerializeError::CStringError)?;
	// Don't forget to call `lingua_dealloc_received_string` once done with this
	// on the Luau side
	Ok(c_str.into_raw() as *mut u8)
}