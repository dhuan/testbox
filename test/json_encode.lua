--[[ EXPECTED OUTPUT
📁 json_encode.lua
✅ convert lua table to json string
✅ use the helper for empty arrays
✅ use the helper for a top-level empty array
END EXPECTED OUTPUT --]]

test("convert lua table to json string", function()
    expect_equal(json_encode({
        some_object = {
            some_key = "some_value",
            some_list = {1,2,3},
        }
    }), [[{"some_object":{"some_key":"some_value","some_list":[1,2,3]}}]])
end)

test("use the helper for empty arrays", function()
    expect_equal(json_encode({
        some_object = {
            some_key = "some_value",
            some_list = EMPTY_ARRAY,
        }
    }), [[{"some_object":{"some_key":"some_value","some_list":[]}}]])
end)

test("use the helper for a top-level empty array", function()
    expect_equal(json_encode(EMPTY_ARRAY), "[]")
end)
