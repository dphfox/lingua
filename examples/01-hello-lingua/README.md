# 01 - Hello Lingua

**A miniature example of Lingua transmitting data between Rust and Luau.** You can try using this example as a template
for your own experimentation.

## Author comments

When you include a Rust module in a Luau project with Wasynth, you can't easily send complex data between the two.
Extern functions can only send simple numbers.

Lingua gets around this by letting you turn a complex piece of data into a simple number. You can pass this simple
number between Rust and Luau however you want to. At the final destination, you can turn the simple number back into the
original piece of data.

You don't need to worry about how this works for the most part. As long as your data can be represented neatly as JSON,
it'll transfer to the other side just fine.