--[[ EXPECTED OUTPUT
📁 interpolate.lua
✅ replace string and number placeholders
✅ replace repeated placeholders
✅ leave unknown placeholders unchanged
✅ convert boolean values to strings
END EXPECTED OUTPUT --]]

test("replace string and number placeholders", function()
    expect_equal(
        "My name is joe, I'm 200 years old.",
        interpolate("My name is {name}, I'm {age} years old.", {
            name = "joe",
            age = 200,
        })
    )
end)

test("replace repeated placeholders", function()
    expect_equal(interpolate("{name} and {name}", { name = "Ada" }), "Ada and Ada")
end)

test("leave unknown placeholders unchanged", function()
    expect_equal(interpolate("Hello, {name}!", {}), "Hello, {name}!")
end)

test("convert boolean values to strings", function()
    expect_equal(interpolate("Active: {active}", { active = true }), "Active: true")
end)
