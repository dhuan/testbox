--[[ EXPECTED OUTPUT
📁 fetch.lua
✅ assert response data
✅ POST request
END EXPECTED OUTPUT --]]

local mock_server = [[
mock serve -p 4000 \
    --route test --status-code 205 --header 'foo: bar' --response 'Hello, world!' \
    --route test-post --method post --exec 'printf "This is the POST endpoint.
Request payload: %s
Request header some-header-key: %s
" "$(mock get-payload)" "$(mock get-header -v some-header-key)" | mock write' \
    >&2
]]

function serve_mock_and_request(fetch_options)
    exec_bg(mock_server)

    return fetch(fetch_options)
end

test("assert response data", function()
    local response = serve_mock_and_request({
        url = "http://localhost:4000/test",
    })

    expect_equal(response.body, "Hello, world!")
    expect_equal(response.status, 205)
    expect_equal(response.headers["foo"], "bar")
    expect_equal(response.headers["bar"], nil)
end)

test("POST request", function()
    local response = serve_mock_and_request({
        url = "http://localhost:4000/test-post",
        method = "post",
        headers = {
            ["some-header-key"] = "some header value",
        },
        body = json_encode({
            foo = "bar",
        }),
    })

    expect_equal(response.body, [[This is the POST endpoint.
Request payload: {"foo":"bar"}
Request header some-header-key: some header value
]])
end)
