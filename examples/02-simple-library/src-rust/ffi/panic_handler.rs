// This forwards on any panics to the Luau side, where they can become visible
// in the output.

use std::panic;

extern "C" {
	fn panic_reporter(
		len_or_byte: u32
	);
}

pub fn connect() {
	panic::set_hook(
		Box::new(|panic| {
			let foo = format!("{panic}");
			unsafe { panic_reporter(foo.len() as u32); }
			foo.bytes().for_each(|byte| unsafe { panic_reporter(byte as u32); });
		})
	);
}