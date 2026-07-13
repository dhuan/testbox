--[[ EXPECTED OUTPUT
📁 merge.lua
  ✅ merge returns a new table with overwritten keys
  ✅ merge accepts multiple source tables
  ✅ merge is shallow
END EXPECTED OUTPUT --]]

test("merge returns a new table with overwritten keys", function()
    local obj_a = {
        foo = "bar",
        hello = "world",
    }

    local obj_b = merge(obj_a, {
        new_key = "new_value",
        hello = "changed!",
    })

    expect_equal(obj_b, {
        foo = "bar",
        new_key = "new_value",
        hello = "changed!",
    })
    expect_equal(obj_a, {
        foo = "bar",
        hello = "world",
    })
    expect_equal(obj_b == obj_a, false)
end)

test("merge accepts multiple source tables", function()
    expect_equal(merge(
        { a = 1, b = 1 },
        { b = 2, c = 2 },
        { c = 3, d = 3 }
    ), {
        a = 1,
        b = 2,
        c = 3,
        d = 3,
    })
end)

test("merge is shallow", function()
    local nested = { count = 1 }
    local copied = merge({ nested = nested }, {})

    nested.count = 2

    expect_equal(copied.nested.count, 2)
end)
