-- Lune-style imports make both Luau LSP and Selene very unhappy...
--!nocheck
--!nolint LocalShadow
--selene: allow(incorrect_standard_library_use)
local process = require("@lune/process")
--selene: allow(incorrect_standard_library_use)
local stdio = require("@lune/stdio")
--selene: allow(incorrect_standard_library_use)
local fs = require("@lune/fs")

local RED = stdio.color("red")
local BLUE = stdio.color("blue")
local YELLOW = stdio.color("yellow")
local MAGENTA = stdio.color("magenta")
local RESET = stdio.color("reset")

local function get_example_dir(
	example_name: string
): string
	return `../{example_name}`
end

local function build_single_example(
	example_name: string,
	_example_type: "lib" | "bin"
): ()
	local working_dir = get_example_dir(example_name)
	print(`{YELLOW}[Build Example]{RESET} Building example: {BLUE}{example_name}{RESET}`)

	-- Compile Rust source code to WASM
	do
		print(`{BLUE}[Build Example > WASM]{RESET} Compiling WASM module with cargo...`)
		local result = process.spawn(
			"cargo",
			{
				"build",
				"--release",
				"--target-dir",
				"target"
			},
			{
				cwd = working_dir
			}
		)
		if not result.ok then
			print(result.stdout)
			print(result.stderr)
			print(`{BLUE}[Build Example > WASM]{RESET} WASM compile failed!`)
			return
		end
		print(`{BLUE}[Build Example > WASM]{RESET} WASM compile successful!`)
	end

	-- Convert WASM to Luau with Wasynth
	do
		print(`{MAGENTA}[Build Example > Wasynth]{RESET} Transpiling WASM to Luau with Wasynth...`)
		local result = process.spawn(
			"wasm2luau",
			{
				"target/wasm32-unknown-unknown/release/example_wasm.wasm"
			},
			{
				cwd = working_dir
			}
		)
		if not result.ok then
			print(result.stdout)
			print(result.stderr)
			print(`{MAGENTA}[Build Example > Wasynth]{RESET} Transpilation failed!`)
			return
		end

		print(`{MAGENTA}[Build Example > Wasynth]{RESET} Saving transpilation to file...`)

		pcall(
			fs.writeDir,
			working_dir .. "/target-luau"
		)
		local ok, result = pcall(
			fs.writeFile,
			working_dir .. "/target-luau/example_wasm.luau",
			"--!nocheck\n" .. result.stdout
		)
		if not ok then
			print(tostring(result))
			print(`{MAGENTA}[Build Example > Wasynth]{RESET} Transpilation failed!`)
			return
		end

		print(`{MAGENTA}[Build Example > Wasynth]{RESET} Transpilation succesful!`)
	end

	-- Build Rojo project
	do
		print(`{RED}[Build Example > Rojo]{RESET} Building Roblox project with Rojo...`)
		local result = process.spawn(
			"rojo",
			{
				"build",
				"example.project.json",
				"--output",
				"target-luau/build.rbxl"
			},
			{
				cwd = working_dir
			}
		)
		if not result.ok then
			print(result.stdout)
			print(result.stderr)
			print(`{RED}[Build Example > Rojo]{RESET} Rojo build failed!`)
			return
		end
		print(`{RED}[Build Example > Rojo]{RESET} Rojo build successful!`)
	end

	print(`{YELLOW}[Build Example]{RESET} Build finished!`)
	print(`{YELLOW}[Build Example]{RESET} The example project has been set up inside {BLUE}target-luau/build.rbxl{RESET} for you.`)
	print(`{YELLOW}[Build Example]{RESET} You can continuously serve this example by running {BLUE}rojo serve example.project.json{RESET}.`)
end

return build_single_example