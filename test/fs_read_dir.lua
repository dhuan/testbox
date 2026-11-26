--[[ EXPECTED OUTPUT
📁 fs_read_dir.lua
✅ Read a directory successfully
END EXPECTED OUTPUT --]]

test("Read a directory successfully", function()
    expect_equal(fs.read_dir(os.getenv("TEST_TMP_DIR")), {
        {kind = "directory", name = "another_dir"},
        {kind = "file", name = "another_file"},
        {kind = "directory", name = "some_dir"},
        {kind = "file", name = "some_file"},
    })
end)
