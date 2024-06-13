--!nocheck
--selene: allow(incorrect_standard_library_use)
local build_single_example = require("../../build-examples/build_single_example")

build_single_example("02-simple-library", "lib")