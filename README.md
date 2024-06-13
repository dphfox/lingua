<p align="center">
<img src="./gh-assets/lingua.svg" alt="Lingua logo">
</p>
<h3 align="center">
Send complex data between Rust and Roblox Luau via Wasynth using JSON.
</h3>

## Install

```
cargo add lingua_luau
```

## But why‽

The Rust ecosystem contains a lot of valuable tooling for Luau developers. However, for many of us, we're not able to incorporate it into our work, because we're restricted to working inside of sandboxed Luau runtimes.

Wasynth provides an exciting opportunity to bridge that gap, allowing basic WASM to run completely under Luau for the first time. However, it's pretty low level, meaning developers have to implement bespoke strategies for communicating across the WASM boundary. Generating bindings could help here, but a more immediate, compatible, and technologically straightforward solution is desirable.

That's where Lingua comes in. Leveraging Rust's serde-json crate and Roblox's JSON encoding methods, Lingua allows Rust and Luau code to freely communicate complex structured data without setting up dedicated bindings or complex binary formats.

Lingua provides friendly APIs on both the Rust and Luau side so that end users don't have to worry about memory management or other implementation details.

## Long-term vision

Lingua's ultimate goal is to become obsolete as Rust/Luau interop becomes more complete. An obvious first step would be to replace JSON with a dedicated binary format. Ultimately, it would be good to see proper bindgen tech built out, to minimise the overhead of interfacing between Rust and Luau.

## Contributions and maintenance

This is [a certified Daniel P H Fox Side Project™](https://fluff.blog/2024/04/10/i-dont-want-to-be-a-maintainer.html), which I am sharing because I personally wanted it to exist in the world. I might maintain it. I might not.
Contributions are welcome, but I do not make guarantees about those either.

Feel free to use Lingua, but if you're about to depend on it big time, the security audit's on you. If, for whatever reason, you end up in a spot of bother, you should probably not have used a random project from someone's GitHub without inspecting what it does properly. I take no responsibility for that.

If you're a state sponsored malicious actor... just kidding.
