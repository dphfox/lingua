use lingua::{receive_from_luau, send_to_luau};

mod panic_handler;

#[no_mangle]
pub extern "C" fn calculate_fridge_value(
	prices: u32,
	fridge: u32
) -> u32 {
	panic_handler::connect();

	let result = super::calculate_fridge_value(
		receive_from_luau(prices.into()).unwrap(), 
		receive_from_luau(fridge.into()).unwrap()
	);

	send_to_luau(&result).unwrap().into()
}