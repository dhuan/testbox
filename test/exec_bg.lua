--[[ EXPECTED OUTPUT
📁 exec_bg.lua
✅ exec_bg: wait for text
✅ exec_bg: saving output
END EXPECTED OUTPUT --]]

local function read_file(path)
    local file = io.open(path, "r")

    if file == nil then
        return nil
    end

    local content = file:read("*a")
    file:close()

    return content
end

local function wait_for_file_match(path, pattern)
    for _ = 1, 50 do
        local content = read_file(path)

        if content ~= nil and string.match(content, pattern) ~= nil then
            return content
        end

        os.execute("sleep 0.1")
    end

    return read_file(path)
end

test("exec_bg: wait for text", function()
    local exec_result = exec_bg([[
sleep 1
echo one 

sleep 1
echo two 

sleep 1
echo three 
]], {
        wait = function(stdout, stderr)
            return string.match(stdout, "two") ~= nil
        end
    })

    expect_equal(exec_result.stdout, "one\ntwo\n")
end)

test("exec_bg: saving output", function()
    local output_file = "/tmp/testbox-exec-bg-output-" .. random_chars(12) .. ".log"
    local exec_result = exec_bg([[
echo one
sleep 0.1
echo two
sleep 0.1
echo error >&2
echo three
]], {
        save_output_as = output_file,
        wait = function(stdout, stderr)
            return string.match(stdout, "two") ~= nil
        end
    })

    expect_equal(exec_result.stdout, "one\ntwo\n")

    local saved_output = wait_for_file_match(output_file, "three")

    expect_equal(string.match(saved_output, "one") ~= nil, true)
    expect_equal(string.match(saved_output, "two") ~= nil, true)
    expect_equal(string.match(saved_output, "error") ~= nil, true)
    expect_equal(string.match(saved_output, "three") ~= nil, true)
end)
