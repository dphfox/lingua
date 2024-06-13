#[cfg(not(target_arch = "wasm32"))]
compile_error!("This project must target WebAssembly to compile correctly.");

use std::collections::HashMap;

pub mod ffi;

// Lingua understands any data types that Serde can understand. That makes it
// incredibly easy to write functions in Rust that can be called from Luau.

// In this file, a function is defined using a range of Rust-native types.
// Notice that it pretty much looks identical to what you'd normally write!

// However, the function is private, because we don't want to expose it yet.
// Instead, there's a public `ffi` module inside this library, which contains
// a public variant of this function.

// The public `ffi` variant only accepts and returns `u32` values; it uses
// Lingua to convert into the data format that this private variant accepts.

// Over on the Luau side, an idiomatic wrapper has been written around the
// `ffi` variant of the function. This wrapper accepts Luau data, sends the data
// to Lingua, and calls the `ffi` function with the resulting handles. After
// the `ffi` function returns, it similarly uses Lingua to retrieve the result.

// This general structure lets the pure Luau code deal with pure Luau data, and
// also lets the pure Rust code deal with pure Rust data. The `ffi` module and 
// library wrapper work together with Lingua to completely abstract away the
// boundary between the two languages.

fn calculate_fridge_value(
	prices: HashMap<String, f64>,
	fridge: Vec<String>
) -> Result<f64, String> {
	fridge.into_iter()
		.map(|item| prices.get(&item))
		.sum::<Option<_>>()
		.ok_or_else(|| String::from("An item in the fridge didn't have a price."))
}