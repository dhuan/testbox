--[[ EXPECTED OUTPUT
📁 fail_fast_short.lua
❌ first test fails
Not equal!
Left:  1
Right: 2
END EXPECTED OUTPUT --]]

-- OPTIONS: -x

test("first test fails", function()
    expect_equal(1, 2)
end)

test("second test does not run", function()
    expect_equal(1, 1)
end)
