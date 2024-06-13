# 02 - Simple Library

**A tiny library written in Rust, which can be called naturally from Luau.** You can try using this example as a 
template for your own experimentation.

## Author comments

Lingua understands any data types that Serde can understand. That makes it incredibly easy to write functions in Rust
that can be called from Luau.

In the main Rust file, a function is defined using a range of Rust-native types. Notice that it pretty much looks 
identical to what you'd normally write!

However, the function is private, because we don't want to expose it yet. Instead, there's a public `ffi` module inside
the library, meant to contain public WASM-compatible variants of the library functions.

Public `ffi` functions only accept and returns `u32` values; they use Lingua to convert into the data format that the
private library functions accept.

Over on the Luau side, an idiomatic wrapper has been written around the `ffi` module exported by Rust. This wrapper
accepts Luau data, sends the data to Lingua, and calls into `ffi` with the resulting handles. After `ffi` returns, it 
similarly uses Lingua to retrieve the result.

This general structure lets the pure Luau code deal with pure Luau data, and also lets the pure Rust code deal with pure 
Rust data. The `ffi` module and Luau library wrapper work together with Lingua to completely abstract away the boundary
between the two languages.