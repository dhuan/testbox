--[[ EXPECTED OUTPUT
✅ exec with awk to uppercase text
✅ exec with failing status code
✅ exec with stdin
END EXPECTED OUTPUT --]]

test("exec with awk to uppercase text", function()
    local result = exec([[echo foobar | awk '{print toupper($0)}']])

    expect_equal(result.stdout, "FOOBAR")
    expect_equal(result.stderr, "")
    expect_equal(result.status, 0)
end)

test("exec with failing status code", function()
    local result = exec([[ls this_should_fail]])

    expect_equal(result.status > 0, true)
    expect_equal(string.match(result.stderr, "this_should_fail") ~= nil, true)
    expect_equal(result.stdout, "")
end)

test("exec with stdin", function()
    local result = exec([[grep foo]], {
        stdin = [[hello
world
foo
bar
done
]]
    })

    expect_equal(result.stdout, "foo")
end)
