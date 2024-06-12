use serde::{Serialize, Deserialize};

use lingua_luau;

#[derive(Serialize, Deserialize)]
struct LuauGreeting {
	greeting_from_lua: String
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

	// When you include a Rust module in a Luau project with Wasynth, you can't
	// easily send complex data between the two. Extern functions can only send
	// simple numbers.

	// Lingua gets around this by letting you turn a complex piece of data into
	// a simple number. You can pass this simple number between Rust and Luau 
	// however you want to. At the final destination, you can turn the simple
	// number back into the original piece of data.

	// You don't need to worry about how this works for the most part. As long
	// as your data can be represented neatly as JSON, it'll transfer to the
	// other side just fine.

	
	let luau_greeting: LuauGreeting = {
		let handle = unsafe { ask_luau_to_say_hello() }.into();
		lingua_luau::receive_from_luau(handle).unwrap()
	};

	let rust_greeting = RustGreeting {
		in_response_to: luau_greeting,
		greeting_from_rust: String::from("Hello, Luau!")
	};

	{
		let handle = lingua_luau::send_to_luau(rust_greeting).unwrap();
		unsafe { respond_to_luau_greeting(handle.into()) };
	}
}
