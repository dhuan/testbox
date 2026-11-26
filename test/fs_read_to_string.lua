--[[ EXPECTED OUTPUT
📁 fs_read_to_string.lua
✅ read file successfully
✅ error: file does not exist
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
