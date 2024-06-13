#[cfg(not(target_arch = "wasm32"))]
compile_error!("This project must target WebAssembly to compile correctly.");

use std::collections::HashMap;

pub mod ffi;

fn calculate_fridge_value(
	prices: HashMap<String, f64>,
	fridge: Vec<String>
) -> Result<f64, String> {
	fridge.into_iter()
		.map(|item| prices.get(&item))
		.sum::<Option<_>>()
		.ok_or_else(|| String::from("An item in the fridge didn't have a price."))
}