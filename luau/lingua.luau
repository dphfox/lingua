--!strict
--Licensed under MIT from Lingua, (c) Daniel P H Fox 2024

--------------------------------------------------------------------------------
-- TYPE DEFINITIONS ------------------------------------------------------------
--------------------------------------------------------------------------------

-- Type aliases for convenience and documentation
type ptr_mut_u8 = number
type u32 = number
type u8 = number
type table = {
	[unknown]: unknown
}

-- Passed to Wasynth when initialising a WebAssembly module.
-- Lingua injects some of its own members into this environment, but otherwise
-- does not care about the rest of the contents.
type WasmEnvironment = {
	func_list: {
		[string]: (...any) -> (...any)
	}
}

-- Returned by Wasynth when initialising a WebAssembly module.
-- Lingua expects certain members to be present - in particular, some extern fn
-- definitions from the Rust side.
type WasmModule = {
	func_list: {
		lingua_send_json_to_rust_alloc: (
			luau_handle: u32,
			len: u32
		) -> ptr_mut_u8,
		lingua_send_json_to_rust: (
			luau_handle: u32
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

-- An entry in the `memory_list` dictionary returned by Wasynth.
-- Lingua doesn't care about its specific type - it's treated opaquely.
type WasmMemory = unknown

-- The internal state used by Lingua's Luau-side API.
-- This is not shared between WASM modules.
type ApiState = {
	-- This module provides the low level FFI that the API functions use.
	module: WasmModule,
	-- When the Rust side sends JSON strings, they're stored here, indexed by
	-- which handle the Rust side decided to use.
	just_received_json: {
		[u32]: string
	},
	-- To uniquely identify JSON strings the Luau side sends to the Rust side,
	-- this handle is incremented and wraps at the 32-bit limit. Handles
	-- generated on the Luau side may collide with handles generated on the Rust
	-- side because no synchronisation is done.
	next_luau_handle: u32
}

-- The public API surface exposed by Lingua after initialisation.
-- The API state is expected to be encapsulated as upvalues in closures.
export type Api = {
	-- Sends some data to the Rust side. An opaque handle is returned, which
	-- uniquely identifies this data. This handle can be sent to the Rust side
	-- separately, and on the Rust side, it can be used to receive the data.
	send_to_rust: (
		data: table
	) -> u32,

	-- Receives some data from the Rust side. You should have received an opaque
	-- handle from the Rust side, which uniquely identifies the data that was
	-- sent. The received data will be decoded into a Luau table.
	receive_from_rust: (
		rust_handle: u32
	) -> table
}

--------------------------------------------------------------------------------
-- CONSTANTS -------------------------------------------------------------------
--------------------------------------------------------------------------------

-- The return values of Lingua FFI functions are used to indicate whether the
-- FFI call was successful. Note that this has nothing to do with the specific
-- operation - it's specifically used to communicate low-level failures.
-- Returned data or operation-specific errors should be sent some other way.
-- The only exception are memory allocations, which must return a pointer for
-- pragmatic reasons - in this case, the null pointer represents failure.
local RETURN_CODES = {
	LUAU = {
		-- The FFI call was successfully handled without an unexpected error.
		SUCCESS = 0,
		-- A Luau error occurred outside of a protected block in an FFI call.
		UNCAUGHT_ERROR_AT_FFI_BOUNDARY = 1,
		-- An FFI call was received before Lingua's Luau API knew about the 
		-- module that was sending the data.
		LUAU_API_NOT_READY = 2
	},
	RUST = {
		-- The FFI call was successfully handled without panicking.
		SUCCESS = 0,
		-- A Rust panic unwound up to the FFI boundary.
		PANIC_AT_FFI_BOUNDARY = 1
	}
}

--------------------------------------------------------------------------------
-- UTILITIES -------------------------------------------------------------------
--------------------------------------------------------------------------------

-- Serialise a table into a JSON string. Right now, this uses Roblox API under
-- the hood, but there's no reason a pure Luau solution can't be used if you're
-- using a standalone runtime like Lune.
local function encode_json(
	t: table
): string
	return game:GetService("HttpService"):JSONEncode(t)
end

-- Deserialise a JSON string into a table. Right now, this uses Roblox API under
-- the hood, but there's no reason a pure Luau solution can't be used if you're
-- using a standalone runtime like Lune.
local function decode_json(
	str: string
): table
	return game:GetService("HttpService"):JSONDecode(str)
end

-- Increment a u32 number. If the next number would be outside of the range that
-- u32 numbers can represent, wrap around to 0 again.
local function next_u32(
	current: u32
): u32
	return (current + 1) % (2 ^ 32)
end

-- This is used in FFI functions to ensure that errors do not spill across the
-- FFI boundary. Errors print and return `UNCAUGHT_ERROR_AT_FFI_BOUNDARY`.
-- Otherwise, the `SUCCESS` code is returned.
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
	return if ok then 
		RETURN_CODES.LUAU.SUCCESS 
	else 
		RETURN_CODES.LUAU.UNCAUGHT_ERROR_AT_FFI_BOUNDARY
end

--------------------------------------------------------------------------------
-- FOREIGN FUNCTION INTERFACE --------------------------------------------------
--------------------------------------------------------------------------------

local extern_fn = {}

-- The Rust side calls this function when it sends a JSON string. It's called
-- with the handle that it generated, the pointer to the string, and the length
-- of that string. To minimise the chance of errors at the FFI boundary, the
-- string is saved without decoding the data or invoking any user callbacks.
function extern_fn.send_json_to_luau(
	api_state: ApiState,
	rust_handle: u32,
	ptr: ptr_mut_u8,
	len: u32
): ()
	assert(
		api_state.just_received_json[rust_handle] == nil, 
		`[lingua] rust handle {rust_handle} is already in use - ensure you're reading all data sent to the luau side`
	)
	local str = api_state.module.rt.load.string(
		api_state.module.memory_list.memory, 
		ptr, 
		len
	)
	api_state.just_received_json[rust_handle] = str
end

--------------------------------------------------------------------------------
-- LUAU SIDE API ---------------------------------------------------------------
--------------------------------------------------------------------------------

local api_fn = {}

-- See the `Api` type for user-facing documentation.
function api_fn.send_to_rust(
	api_state: ApiState,
	data: table
): u32
	local str = encode_json(data)
	local luau_handle = api_state.next_luau_handle
	api_state.next_luau_handle = next_u32(luau_handle)
	local len = string.len(str)
	local ptr = api_state.module.func_list.lingua_send_json_to_rust_alloc(luau_handle, len)
	assert(
		ptr ~= 0,
		"[lingua] rust side failed to allocate memory for data to be sent"
	)
	api_state.module.rt.store.string(
		api_state.module.memory_list.memory,
		ptr,
		str
	)
	local status = api_state.module.func_list.lingua_send_json_to_rust(luau_handle)
	if status ~= RETURN_CODES.RUST.SUCCESS then
		local codename = "(unknown)"
		for name, code in pairs(RETURN_CODES.RUST) do
			if status == code then
				codename = name
				break
			end
		end
		error(`[lingua] rust side returned error code {status}: {codename}`)
	end
	return luau_handle
end

-- See the `Api` type for user-facing documentation.
function api_fn.receive_from_rust(
	api_state: ApiState,
	user_handle: u32
): table
	local str = api_state.just_received_json[user_handle]
	assert(
		str ~= nil, 
		`[lingua] handle {user_handle} has no data - handles should be generated on the sending side and are single use`
	)
	return decode_json(str)
end

--------------------------------------------------------------------------------
-- INITIALISATION --------------------------------------------------------------
--------------------------------------------------------------------------------

local lingua = {}

-- Lingua needs to be initialised in two stages; firstly, it needs to register
-- its own extern functions into the WASM environment, and secondly, it needs to
-- obtain a reference to the WASM module so that it can invoke functions that
-- exist on the Rust side. Once both have been received, you will receive an
-- `Api` that you can use with the provided module.
function lingua.init(
	env: WasmEnvironment
)
	local api_state: ApiState? = nil
	for fn_name, fn in pairs(extern_fn) do
		env.func_list["lingua_"..fn_name] = function(...)
			if api_state == nil then
				print("[lingua] api not initialised yet on luau side - cannot handle incoming data")
				return RETURN_CODES.LUAU.LUAU_API_NOT_READY
			end
			return ffi_error_boundary(fn, api_state, ...)
		end
	end
	return function(
		module: WasmModule
	): Api
		assert(api_state == nil, "[lingua] only one module allowed per environment")
		api_state = {
			module = module,
			just_received_json = {},
			next_luau_handle = 0
		}
		local api = {}
		for fn_name, fn in pairs(api) do
			api[fn_name] = function(...)
				return fn(api_state, ...)
			end
		end
		return api :: any
	end
end

return lingua