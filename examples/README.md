# How to run examples

## Prerequisites

You will need `lune`, `cargo`, `wasm2luau` and `rojo` installed.

*Tip: `wasm2luau` can be installed with this command:*

```
cargo install --git https://github.com/Rerumu/Wasynth codegen-luau
```

[See Wasynth on GitHub for more details.](https://github.com/Rerumu/Wasynth/wiki/From-Rust,-to-Lua)

## Steps

- In a terminal, `cd` into the folder of the example you want to run
- Enter `lune run build-this-example`
- Open the `build.rbxl` that was set up inside `target-luau`.
- In Roblox Studio, choose 'Run' (you do not need to 'Play' the file)