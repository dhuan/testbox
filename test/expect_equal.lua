--[[ EXPECTED OUTPUT
📁 expect_equal.lua
✅ test 1 equals 1
❌ test 1 equals 2
Not equal!
Left:  1
Right: 2
END EXPECTED OUTPUT --]]

test("test 1 equals 1", function()
    expect_equal(1, 1)
end)

test("test 1 equals 2", function()
    expect_equal(1, 2)
end)
