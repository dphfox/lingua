local lingua = require(script.Parent.Parent.lingua)
local library_wasm_loader = require(script.Parent.Parent.target_luau.example_wasm)

local add_panic_handler = require(script.Parent.add_panic_handler)

local wasm_env = { func_list = {} }
add_panic_handler(wasm_env.func_list)
local finish_lingua_init = lingua.init(wasm_env)
local library_wasm_module = library_wasm_loader({env = wasm_env})
local lingua_api = finish_lingua_init(library_wasm_module)

local library = {}

function library.calculate_fridge_value(
	prices: {[string]: number},
	fridge: {string}
): number
	local result = lingua_api.receive_from_rust(
		library_wasm_module.func_list.calculate_fridge_value(
			lingua_api.send_to_rust(prices),
			lingua_api.send_to_rust(fridge)
		)
	)
	return result.Ok or error(result.Err)
end

return library