--[[ EXPECTED OUTPUT
✅ convert lua table to json string
END EXPECTED OUTPUT --]]

test("convert lua table to json string", function()
    expect_equal(json_encode({
        some_object = {
            some_key = "some_value",
            some_list = {1,2,3},
        }
    }), [[{"some_object":{"some_key":"some_value","some_list":[1,2,3]}}]])
end)
