--[[ EXPECTED OUTPUT
✅ assert response data
✅ POST request
END EXPECTED OUTPUT --]]

local mock_server = [[
mock serve -p 4000 \
    --route test --status-code 205 --header 'foo: bar' --response 'Hello, world!' \
    --route test-post --method post --response 'This is the POST endpoint.' \
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
        body: to_json({
            foo = "bar",
        }),
    })

    expect_equal(response.body, "This is the POST endpoint.")
end)
