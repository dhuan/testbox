--[[ EXPECTED OUTPUT
📁 json_decode.lua
  ✅ convert json string to lua table
END EXPECTED OUTPUT --]]

test("convert json string to lua table", function()
    expect_equal(json_decode([[{"foo":"bar"}]]), {
        foo = "bar"
    })
end)
