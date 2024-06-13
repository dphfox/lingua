-- When Rust panics, it doesn't exit helpfully by default. This function gives
-- the Rust side an external function it can call with details of the panic,
-- so the error can be emitted on the Luau side instead.

return function(extern_fns)
	local current_panic = nil
	function extern_fns.panic_reporter(
		len_or_byte: number
	): ()
		if current_panic == nil then
			current_panic = {
				len = len_or_byte,
				bytes = ""
			}
		else
			current_panic.bytes ..= string.char(len_or_byte)
			if #current_panic.bytes == current_panic.len then
				error(current_panic.bytes, 0)
			end
		end
	end
end