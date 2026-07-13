--[[ EXPECTED OUTPUT
📁 fail_fast.lua
  ✅ first test passes
  ❌ second test fails
Not equal!
Left:  1
Right: 2
END EXPECTED OUTPUT --]]

-- OPTIONS: --fail-fast

test("first test passes", function()
    expect_equal(1, 1)
end)

test("second test fails", function()
    expect_equal(1, 2)
end)

print("top-level code after failure does not run")

test("third test does not run", function()
    expect_equal(1, 1)
end)
