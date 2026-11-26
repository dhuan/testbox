--[[ EXPECTED OUTPUT
📁 fs_read_to_string.lua
✅ read file successfully
END EXPECTED OUTPUT --]]

test("read file successfully", function()
    expect_equal(fs.read_to_string(os.getenv("TEST_TMP_FILE")), os.getenv("TEST_TMP_FILE_CONTENT"))
end)

