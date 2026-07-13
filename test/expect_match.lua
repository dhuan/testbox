--[[ EXPECTED OUTPUT
✅ expect_match matches partial object fields
✅ expect_match matches nested object fields partially
✅ expect_match rejects top-level scalars
❌ expect_match reports changed field values
Not matching!
At:    $.hello
Left:  "world"
Right: "changed!"
❌ expect_match reports missing fields
Not matching!
Missing key: $.hello
Right:       "world"
END EXPECTED OUTPUT --]]

test("expect_match matches partial object fields", function()
    expect_match({
        foo = "bar",
        hello = "world",
    }, {
        hello = "world",
    })
end)

test("expect_match matches nested object fields partially", function()
    expect_match({
        name = "box",
        nested = {
            count = 1,
            label = "one",
        },
    }, {
        nested = {
            label = "one",
        },
    })
end)

test("expect_match rejects top-level scalars", function()
    local ok, err = pcall(function()
        expect_match("world", {
            hello = "world",
        })
    end)

    expect_equal(ok, false)
    expect_equal(string.match(tostring(err), "expect_match expects two tables") ~= nil, true)
end)

test("expect_match reports changed field values", function()
    expect_match({
        hello = "world",
    }, {
        hello = "changed!",
    })
end)

test("expect_match reports missing fields", function()
    expect_match({
        foo = "bar",
    }, {
        hello = "world",
    })
end)
