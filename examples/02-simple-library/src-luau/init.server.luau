--!nocheck
-- See src/lib.rs for an introduction to this example.

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

local invalid_fridge = {
	"garden chair",
	"basket",
	"clothes pegs",
	"tennis ball"
}
-- This will error!
library.calculate_fridge_value(prices, invalid_fridge)