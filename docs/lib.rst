Testbox Library
###############

.. contents::
   :local:

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

Environment variables can be passed to the command environment using the ``env`` options parameter:

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

Like `exec <exec_>`_, ``exec_bg`` can receive environment variables through the options object.

.. code::

    local server = exec_bg("npm run dev", {
        env = {
            PORT = "3001",
        },
        wait = function(stdout, stderr)
            return string.match(stdout, "Server is ready") ~= nil
        end
    })

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


fs
==

Provides basic filesystem operations. Relative paths are resolved from the
process's current working directory. Symlinks are not supported: operations on
symlink paths, and directory listings that contain symlinks, fail with
``ERR_SYMLINK_NOT_SUPPORTED``.

fs.read_to_string
-----------------

Reads a UTF-8 text file. It throws ``ERR_FILE_DOES_NOT_EXIST`` if the file
does not exist and ``ERR_NOT_A_FILE`` if the path is not a file.

.. code::

    local ok, content = pcall(fs.read_to_string, "some/file.txt")
    if not ok and content == "ERR_FILE_DOES_NOT_EXIST" then
        print("The file is missing")
    end


fs.read_dir
-----------

Returns the immediate entries in a directory as an array of tables with
``kind`` (``"directory"`` or ``"file"``) and ``name`` fields. It throws
``ERR_DIR_DOES_NOT_EXIST`` if the directory does not exist and ``ERR_NOT_A_DIR``
if the path is not a directory.

.. code::

    local entries = fs.read_dir("some/path", { sort = "numeric" })

Entries are sorted by name alphabetically by default. Set ``sort`` to
``"numeric"`` for natural sorting, which compares digit runs by numeric value
(so ``file2`` comes before ``file10``), or ``"alphabetic"`` to select the
default explicitly. An unsupported sort value throws
``ERR_INVALID_SORT_OPTION``.


fs.create_file
--------------

Creates a file, optionally writing the supplied string as its contents. If a
file already exists, the call succeeds without changing its contents. A
missing parent directory throws ``ERR_DIR_DOES_NOT_EXIST``; a target that is
not a file throws ``ERR_NOT_A_FILE``. A parent path component that is not a
directory throws ``ERR_NOT_A_DIR``.

.. code::

    fs.create_file("some/file.txt")
    fs.create_file("some/file.txt", "Hello, world!")


fs.create_dir
-------------

Creates a directory and succeeds if the directory already exists. By default,
missing parent directories are not created; pass ``{ create_parents = true }``
to create them. A missing parent throws ``ERR_DIR_DOES_NOT_EXIST``, and a path
that is not a directory throws ``ERR_NOT_A_DIR``.

.. code::

    fs.create_dir("some/dir", { create_parents = true })


Errors
------

The named error codes are plain strings returned by ``pcall``. Other filesystem
errors are reported with their operating-system error message.


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

Lua removes fields assigned ``nil``, so use ``JSON_NULL`` to encode a JSON null:

.. code::

    json_encode({ foo = JSON_NULL }) -- {"foo":null}

``json_decode`` also returns ``JSON_NULL`` for JSON null values, including
values nested in objects and arrays.


json_decode
===========

Decodes a JSON string into Lua values.

.. code::

    local payload = json_decode([[{"name":"Ada","active":true}]])

    expect_equal("Ada", payload.name)
    expect_equal(true, payload.active)

JSON null values are represented by ``JSON_NULL`` so they remain present in
Lua tables.


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

interpolate
===========

Replaces placeholders in a string with their corresponding values from the provided variables object.

.. code::

    expect_equal(
        "My name is joe, I'm 200 years old.",
        interpolate("My name is {name}, I'm {age} years old.", {
            name = "joe",
            age = 200,
        })
    )
