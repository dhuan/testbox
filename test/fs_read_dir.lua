--[[ EXPECTED OUTPUT
📁 fs_read_dir.lua
✅ Read a directory successfully
✅ Sort numerically
✅ Error: directory does not exist
✅ Error: not a directory
✅ Error: invalid sort option
END EXPECTED OUTPUT --]]

test("Read a directory successfully", function()
    expect_equal(fs.read_dir(os.getenv("TEST_TMP_DIR")), {
        {kind = "directory", name = "another_dir"},
        {kind = "file", name = "another_file"},
        {kind = "directory", name = "dir_with_files"},
        {kind = "directory", name = "some_dir"},
        {kind = "file", name = "some_file"},
    })
end)

test("Sort numerically", function()
    expect_equal(fs.read_dir(os.getenv("TEST_TMP_DIR") .. "/dir_with_files", {sort = "numeric"}), {
        {kind = "file", name = "file1"},
        {kind = "file", name = "file5"},
        {kind = "file", name = "file10"},
        {kind = "file", name = "file15"},
        {kind = "file", name = "file100"},
        {kind = "file", name = "file250"},
    })
end)

test("Error: directory does not exist", function()
    expect_equal(
        table.pack(pcall(fs.read_dir, "unexisting_dir")),
        {false, "ERR_DIR_DOES_NOT_EXIST"}
    )
end)

test("Error: not a directory", function()
    expect_equal(
        table.pack(pcall(fs.read_dir, os.getenv("TEST_TMP_FILE"))),
        {false, "ERR_NOT_A_DIR"}
    )
end)

test("Error: invalid sort option", function()
    expect_equal(
        table.pack(pcall(fs.read_dir, os.getenv("TEST_TMP_DIR"), { sort = "invalid" })),
        {false, "ERR_INVALID_SORT_OPTION"}
    )
end)
