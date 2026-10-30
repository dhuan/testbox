--[[ EXPECTED OUTPUT
📁 json_decode.lua
✅ convert json string to lua table
✅ preserve JSON null in objects and arrays
END EXPECTED OUTPUT --]]

test("convert json string to lua table", function()
    expect_equal(json_decode([[{"foo":"bar"}]]), {
        foo = "bar"
    })
end)

test("preserve JSON null in objects and arrays", function()
    local decoded = json_decode([[{"object_null":null,"array":[null,"value"]}]])

    expect_equal(decoded.object_null, JSON_NULL)
    expect_equal(decoded.array[1], JSON_NULL)
    expect_equal(decoded.array[2], "value")
end)
