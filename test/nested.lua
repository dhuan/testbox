--[[ EXPECTED OUTPUT
📁 nested.lua
⌛ test nested - all passing
  ✅ first nested test
  ✅ second nested test
✅ test nested - all passing
⌛ test nested - failing
  ❌ first nested test
  Not equal!
  Left:  1
  Right: 2
❌ test nested - failing
END EXPECTED OUTPUT --]]

-- EXIT CODE: 1

test("test nested - all passing", function()
    test("first nested test", function()
        expect_equal(1, 1)
    end)

    test("second nested test", function()
        expect_equal(1, 1)
    end)
end)

test("test nested - failing", function()
    test("first nested test", function()
        expect_equal(1, 2)
    end)

    test("second nested test", function()
        expect_equal(1, 1)
    end)
end)
