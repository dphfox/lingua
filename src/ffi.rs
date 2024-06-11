use std::{collections::HashMap, mem::ManuallyDrop, cell::RefCell};

struct StringAllocs {
	ptr_map: HashMap<*mut u8, ManuallyDrop<String>>
}

impl StringAllocs {
	pub fn new() -> Self {
		Self {
			ptr_map: HashMap::new()
		}
	}

	pub fn alloc(
		&mut self,
		capacity: usize
	) -> *mut u8 {
		let mut str = ManuallyDrop::new(String::with_capacity(capacity));
		let ptr = str.as_mut_ptr();
		self.ptr_map.insert(ptr, str);
		ptr
	}

	pub fn get(
		&mut self,
		ptr: *mut u8
	) -> Option<&str> {
		let Some(str) = self.ptr_map.get(&ptr) else {return None};
		Some(str)
	}

	pub fn get_mut(
		&mut self,
		ptr: *mut u8
	) -> Option<&mut str> {
		let Some(str) = self.ptr_map.get_mut(&ptr) else {return None};
		Some(str)
	}

	pub fn dealloc(
		&mut self,
		ptr: *mut u8
	) -> bool {
		let Some(str) = self.ptr_map.remove(&ptr) else {return false};
		drop(ManuallyDrop::into_inner(str));
		true
	}
}

thread_local! {
    static FOREIGN_STRING_ALLOCS: RefCell<StringAllocs> = RefCell::new(StringAllocs::new());
}

#[no_mangle]
pub extern "C" fn lingua_alloc_string(
	capacity: usize
) -> *mut u8 {
	FOREIGN_STRING_ALLOCS.with_borrow_mut(|allocs| {
		allocs.alloc(capacity)
	})
}

#[no_mangle]
pub extern "C" fn lingua_dealloc_string(
	ptr: *mut u8
) -> u8 {
	FOREIGN_STRING_ALLOCS.with_borrow_mut(|allocs| {
		allocs.dealloc(ptr) as u8
	})
}