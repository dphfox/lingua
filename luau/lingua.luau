--!strict
--Licensed under MIT from Lingua, (c) Daniel P H Fox 2024

type ptr_mut_u8 = number
type u32 = number
type u8 = number

type table = {[unknown]: unknown}

type WasmEnvironment = {
	func_list: {[string]: (...any) -> (...any)}
}

type WasmModule = {
	func_list: {
		lingua_send_json_alloc: (
			user_handle: u32,
			len: u32
		) -> ptr_mut_u8,
		lingua_send_json_invoke: (
			user_handle: u32
		) -> u8
	},
	rt: {
		store: {
			string: (
				memory: WasmMemory,
				ptr: ptr_mut_u8,
				str: string
			) -> ()
		},
		load: {
			string: (
				memory: WasmMemory, 
				ptr: ptr_mut_u8, 
				len: u32
			) -> string
		}
	},
	memory_list: {
		memory: WasmMemory
	}
}

type WasmMemory = unknown

type ApiState = {
	module: WasmModule,
	just_received_json: {
		[u32]: string
	},
	next_user_handle: u32
}

local function encode_json(
	t: table
): string
	local HttpService = game:GetService("HttpService")
	return HttpService:JSONEncode(t)
end

local function decode_json(
	str: string
): table
	local HttpService = game:GetService("HttpService")
	return HttpService:JSONDecode(str)
end

local function next_u32(
	current: u32
): u32
	return (current + 1) % 4294967295
end

local function ffi_error_boundary<Args...>(
	f: (Args...) -> (),
	...: Args...
): u8
	local ok = xpcall(
		f, 
		function(err)
			print(`[lingua] error at ffi boundary: {err}`)
		end, 
		...
	)
	return if ok then 0 else 1
end

local extern_fn = {}
function extern_fn.receive_from_extern(
	api_state: ApiState,
	user_handle: u32,
	ptr: ptr_mut_u8,
	len: u32
): ()
	assert(
		api_state.just_received_json[user_handle] == nil, 
		`user handle {user_handle} is already in use - ensure you're reading all data sent to the luau side`
	)
	local str = api_state.module.rt.load.string(
		api_state.module.memory_list.memory, 
		ptr, 
		len
	)
	api_state.just_received_json[user_handle] = str
end

local api = {}
function api.send(
	api_state: ApiState,
	data: table
): u32
	local str = encode_json(data)
	local user_handle = api_state.next_user_handle
	api_state.next_user_handle = next_u32(user_handle)
	local len = string.len(str)
	local ptr = api_state.module.func_list.lingua_send_json_alloc(user_handle, len)
	assert(
		ptr ~= 0,
		"rust side failed to allocate memory for data to be sent"
	)
	api_state.module.rt.store.string(
		api_state.module.memory_list.memory,
		ptr,
		str
	)
	return user_handle
end
function api.receive(
	api_state: ApiState,
	user_handle: u32
): table
	local str = api_state.just_received_json[user_handle]
	assert(
		str ~= nil, 
		`user handle {user_handle} has no data - handles should be generated on the rust side and are single use`
	)
	return decode_json(str)
end

local lingua = {}

function lingua.init()
	local env, module = _G.todo()

	for fn_name, fn in pairs(extern_fn) do
		env.func_list["lingua_"..fn_name] = function(...)
			return ffi_error_boundary(fn, module, ...)
		end
	end
	return env
end

return lingua