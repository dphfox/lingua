#[cfg(not(target_arch = "wasm32"))]
compile_error!("This project must target WebAssembly to compile correctly.");

use lingua::{receive_from_luau, send_to_luau};
use serde::{Serialize, Deserialize};

mod panic_handler;

#[derive(Serialize, Deserialize)]
struct LuauGreeting {
	greeting_from_luau: String
}

#[derive(Serialize)]
struct RustGreeting {
	in_response_to: LuauGreeting,
	greeting_from_rust: String
}

extern "C" {
	fn ask_luau_to_say_hello() -> u32;
	fn respond_to_luau_greeting(response: u32);
}

fn main() {
	panic_handler::connect();
	
	unsafe {
		let luau_greeting: LuauGreeting = receive_from_luau(
			ask_luau_to_say_hello().into()
		).unwrap();

		let rust_greeting = RustGreeting {
			in_response_to: luau_greeting,
			greeting_from_rust: String::from("Hello, Luau!")
		};

		respond_to_luau_greeting(
			send_to_luau(&rust_greeting).unwrap().into()
		);
	}
}
