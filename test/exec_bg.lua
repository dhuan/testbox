--[[ EXPECTED OUTPUT
📁 exec_bg.lua
  ✅ exec_bg: wait for text
END EXPECTED OUTPUT --]]

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
