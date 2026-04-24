# testbox

A fast test runner that works out of the box. Zero dependencies.

* Tests are written in Lua (but you don't need Lua installed on your system)
* Batteries included for API testing
* Watch mode runs your tests whenever you change files

```sh
$ testbox - <<EOF
test("just a test", function()
    expect_equal(2, 1 + 1)
end)

test("this will fail!", function()
    expect_equal(1, 2)
end)
EOF

# Prints out:

✅ just a test
❌ this will fail!
Not equal!
Left:  1
Right: 2
```

## Installation

If you have cargo/rust on your system:

```sh
$ cargo install testbox

...

$ testbox --version
```

Otherwise you can just download the compiled executable from the [releases page](https://github.com/dhuan/testbox/releases).

## Examples

### Run your API server and fetch

```sh
$ testbox - <<EOF
test("API test: get users", function()
    exec_bg("cd path/to/my/app ; npm run dev", {
        wait = function(stdout, stderr)
            return string.match(stdout, "Server is ready") ~= nil
        end
    })

    local response = fetch({
        url = "http://localhost:3000/api/users",
        method = "GET",
    })

    expect_equal(response.json.users[1].email, "someuser@example.com")
end)
EOF
```

### Test command-line tools

```sh
testbox - <<EOF
test("uppercase text with awk", function()
    expect_equal(
        exec([[echo foobar | awk '{print toupper($0)}']]).output,
        "FOOBAR",
    )
end)
EOF
```

## License

[MIT](LICENSE)
