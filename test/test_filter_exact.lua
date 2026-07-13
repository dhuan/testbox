--[[ EXPECTED OUTPUT
📁 test_filter_exact.lua
  ✅ test two
END EXPECTED OUTPUT --]]

-- OPTIONS: -T 'test TWO'

test("test one", function()
end)

test("test two", function()
end)

test("test three", function()
end)
