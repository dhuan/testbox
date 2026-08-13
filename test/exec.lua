--[[ EXPECTED OUTPUT
📁 exec.lua
✅ exec with awk to uppercase text
✅ exec with failing status code
✅ exec with stdin
✅ exec with env
✅ exec requires env vars to be strings
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

test("exec with env", function()
    local inherited_path = exec([[printf "%s" "$PATH"]]).stdout
    local result = exec([[printf "%s:%s:%s:%s" "$TESTBOX_ENV_ONE" "$TESTBOX_ENV_TWO" "$TESTBOX_ENV_THREE" "$PATH"]], {
        env = {
            TESTBOX_ENV_ONE = "one",
            TESTBOX_ENV_TWO = "two",
            TESTBOX_ENV_THREE = "three",
        }
    })

    expect_equal(result.stdout, "one:two:three:" .. inherited_path)

    local override_result = exec([[printf "%s" "$PATH"]], {
        env = {
            PATH = "/tmp/testbox-path",
        }
    })

    expect_equal(override_result.stdout, "/tmp/testbox-path")
end)

test("exec requires env vars to be strings", function()
    local ok, err = pcall(function()
        exec("true", {
            env = {
                TESTBOX_ENV_INVALID = 1,
            }
        })
    end)

    expect_equal(ok, false)
    expect_equal(string.match(tostring(err), "env values must be strings") ~= nil, true)
end)
