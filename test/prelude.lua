--[[ EXPECTED OUTPUT
📁 prelude.lua
✅ sum
✅ multiply
END EXPECTED OUTPUT --]]

-- OPTIONS: -p 'test/utils_1.lua' -p 'test/utils_2.lua'

test("sum", function()
    expect_equal(sum(1, 2), 3)
end)

test("multiply", function()
    expect_equal(multiply(2, 5), 10)
end)
