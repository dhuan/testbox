--[[ EXPECTED OUTPUT
✅ exec with awk to uppercase text
✅ exec with failing status code
END EXPECTED OUTPUT --]]

test("exec with awk to uppercase text", function()
    local result = exec([[echo foobar | awk '{print toupper($0)}']])

    expect_equal(result.output, "FOOBAR")
    expect_equal(result.status, 0)
end)

test("exec with failing status code", function()
    local result = exec([[ls this_should_faill]])

    expect_equal(result.status > 0, true)
end)
