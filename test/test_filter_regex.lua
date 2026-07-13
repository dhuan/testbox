--[[ EXPECTED OUTPUT
📁 test_filter_regex.lua
✅ test two
END EXPECTED OUTPUT --]]

-- OPTIONS: -t t[a-z]o

test("test one", function()
end)

test("test two", function()
end)

test("test three", function()
end)
