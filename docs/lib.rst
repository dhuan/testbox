Testbox Library
###############

.. contents::
   :local:

.. _lib_exec:
exec
====

Executes a shell command, receiving an object containing the output data and
the command's exit status.

.. code::

    print("The current folder is: " .. exec("pwd").stdout)

You can pass ``stdin`` in an options table to send input to the command.

.. code::

    local result = exec("cat", { stdin = "hello" })
    expect_equal("hello", result.stdout)

The returned object is structured as follows:

.. code::

   stdout: A string containing the output from stdout.
   stderr: A string containing the output from stderr.
   status: The exit status code from the executed command.

Environment variables can be passed to the executed program using the ``env`` options parameter:

.. code::

    expect_equal(
        exec([[echo "FOO=$FOO"]], { env = { FOO = "BAR" } }).stdout,
        "FOO=BAR"
    )

exec_bg
=======

Executes a shell command in the background. This is useful when a test needs to
start another process, such as an API server, before making assertions against
it.

.. code::

    local server = exec_bg("cd path/to/my/app ; npm run dev", {
        wait = function(stdout, stderr)
            return string.match(stdout, "Server is ready") ~= nil
        end
    })

The optional ``wait`` callback receives the current stdout and stderr strings.
When the callback returns true, ``exec_bg`` returns to the test.

The returned object is structured as follows:

.. code::

   stdout: A string containing output captured from stdout.
   stderr: A string containing output captured from stderr.

Like `exec <lib_exec_>`, `exec_bg` can receive environment variables through the options object.

fetch
=====

Makes an HTTP request and returns the response data.

.. code::

    local response = fetch({
        url = "http://localhost:3000/api/users",
        method = "GET",
        headers = {
            ["accept"] = "application/json",
        },
    })

    expect_equal(200, response.status)
    expect_equal("someuser@example.com", response.json.users[1].email)

The options table supports:

.. code::

   url: The request URL. Required.
   method: The HTTP method. Defaults to GET.
   headers: A table of request headers.
   body: A string request body.

The returned object includes the response status, response body text, decoded
JSON when the response body contains JSON, and response headers.


expect_equal
============

Fails the current test unless both values are equal.

.. code::

    expect_equal(2, 1 + 1)

Tables are compared by value, so nested table contents can be asserted directly.

.. code::

    expect_equal({ name = "Ada", roles = { "admin" } }, user)


expect_match
============

Fails the current test unless the actual value matches the expected partial
value.

.. code::

    expect_match({ id = 1, name = "Ada" }, { id = 1 })

This is commonly used when an object contains extra fields that are not relevant
to the test.


json_encode
===========

Encodes a Lua table as a JSON string.

.. code::

    local body = json_encode({
        name = "Ada",
        active = true,
        tags = EMPTY_ARRAY,
    })

Use ``EMPTY_ARRAY`` when a field should encode as ``[]``. A plain empty Lua
table still encodes as ``{}``.


json_decode
===========

Decodes a JSON string into Lua values.

.. code::

    local payload = json_decode([[{"name":"Ada","active":true}]])

    expect_equal("Ada", payload.name)
    expect_equal(true, payload.active)


random_chars
============

Returns a random alphanumeric string with the requested length.

.. code::

    local username = "test-" .. random_chars(8)


merge
=====

Returns a table containing the keys from both input tables. Values from the
second table replace values from the first table when both tables contain the
same key.

.. code::

    local options = merge({
        method = "GET",
        headers = {
            ["accept"] = "application/json",
        },
    }, {
        method = "POST",
        body = json_encode({ name = "Ada" }),
    })

    expect_equal("POST", options.method)
