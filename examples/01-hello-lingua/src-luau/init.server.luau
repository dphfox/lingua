--!nocheck
-- See src/main.rs for an introduction to this example.

local lingua = require(script.Parent.lingua)
local example_wasm_loader = require(script.Parent.target_luau.example_wasm)

local add_panic_handler = require(script.add_panic_handler)

-- Lingua will return an API for us later, when the module is initialised.
local lingua_api: lingua.Api

local extern_fns = {
	ask_luau_to_say_hello = function(): lingua.DataFromLuauHandle
		print("Saying hello to Rust...")
		local data = {
			greeting_from_luau = "Hello, Rust!"
		}
		local handle = lingua_api.send_to_rust(data)
		return handle
	end,

	respond_to_luau_greeting = function(
		response: lingua.DataFromRustHandle
	): ()
		local data = lingua_api.receive_from_rust(response)
		print("Rust has responded:", data)
	end
}
add_panic_handler(extern_fns)

local wasm_env = { func_list = extern_fns }
local finish_lingua_init = lingua.init(wasm_env)
local example_wasm_module = example_wasm_loader({env = wasm_env})
lingua_api = finish_lingua_init(example_wasm_module)

example_wasm_module.func_list.main()