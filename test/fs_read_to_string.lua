--[[ EXPECTED OUTPUT
📁 fs_read_to_string.lua
✅ read file successfully
✅ error: file does not exist
✅ error: not a file
END EXPECTED OUTPUT --]]

test("read file successfully", function()
    expect_equal(fs.read_to_string(os.getenv("TEST_TMP_FILE")), os.getenv("TEST_TMP_FILE_CONTENT"))
end)

test("error: file does not exist", function()
    expect_equal(
        table.pack(pcall(fs.read_to_string, "unexisting_file")),
        {false, "ERR_FILE_DOES_NOT_EXIST"}
    )
end)

test("error: not a file", function()
    expect_equal(
        table.pack(pcall(fs.read_to_string, os.getenv("TEST_TMP_DIR"))),
        {false, "ERR_NOT_A_FILE"}
    )
end)
