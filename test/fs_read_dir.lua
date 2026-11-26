--[[ EXPECTED OUTPUT
📁 fs_read_dir.lua
✅ Read a directory successfully
✅ Error: directory does not exist
END EXPECTED OUTPUT --]]

test("Read a directory successfully", function()
    expect_equal(fs.read_dir(os.getenv("TEST_TMP_DIR")), {
        {kind = "directory", name = "another_dir"},
        {kind = "file", name = "another_file"},
        {kind = "directory", name = "some_dir"},
        {kind = "file", name = "some_file"},
    })
end)

test("Error: directory does not exist", function()
    expect_equal(
        table.pack(pcall(fs.read_dir, "unexisting_dir")),
        {false, "ERR_DIR_DOES_NOT_EXIST"}
    )
end)
