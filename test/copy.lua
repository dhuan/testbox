--[[ EXPECTED OUTPUT
📁 copy.lua
✅ deep copy lua table
END EXPECTED OUTPUT --]]

test("deep copy lua table", function()
    local key = { id = "key" }
    local source = {
        name = "source",
        nested = {
            count = 1,
            list = { "a", "b" },
        },
        [key] = {
            value = "keyed",
        },
    }
    source.self = source

    local copied = copy(source)

    expect_equal(copied.name, "source")
    expect_equal(copied.nested, {
        count = 1,
        list = { "a", "b" },
    })
    expect_equal(copied.self == copied, true)

    source.nested.count = 2
    source.nested.list[1] = "changed"
    key.id = "changed"

    expect_equal(copied.nested, {
        count = 1,
        list = { "a", "b" },
    })

    local copied_key = nil
    for candidate_key, value in pairs(copied) do
        if type(candidate_key) == "table" then
            copied_key = candidate_key
            expect_equal(value, {
                value = "keyed",
            })
        end
    end

    expect_equal(copied_key.id, "key")
end)
