--!nocheck
-- See src/main.rs for an introduction to this example.

local library = require(script.library)

local prices = {
	apple = 10,
	orange = 5,
	banana = 25
}

local fridge = {
	"apple",
	"apple",
	"banana",
	"orange",
	"apple",
	"orange",
	"orange",
	"banana",
	"banana"
}

local fridge_value = library.calculate_fridge_value(prices, fridge)

print(`The value of my fridge is {fridge_value}`)