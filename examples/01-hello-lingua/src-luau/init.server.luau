--!nocheck
-- See src/main.rs for an introduction to this example.

local lingua = require(script.Parent.lingua)
local example_wasm_loader = require(script.Parent.target_luau.example_wasm)

local wasm_env = {}
wasm_env.func_list = {}

local lingua_api: lingua.Api

function wasm_env.func_list.ask_luau_to_say_hello(): lingua.DataFromLuauHandle
	print("Saying hello to Rust...")
	local data = {
		greeting_from_luau = "Hello, Rust!"
	}
	local handle = lingua_api.send_to_rust(data)
	return handle
end

function wasm_env.func_list.respond_to_luau_greeting(
	response: lingua.DataFromRustHandle
): ()
	local data = lingua_api.receive_from_rust(response)
	print("Rust has responded:", data)
end

local finish_lingua_init = lingua.init(wasm_env)
local example_wasm_module = example_wasm_loader({env = wasm_env})
lingua_api = finish_lingua_init(example_wasm_module)

example_wasm_module.func_list.main()