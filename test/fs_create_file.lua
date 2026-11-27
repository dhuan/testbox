--[[ EXPECTED OUTPUT
📁 fs_create_file.lua
✅ create empty file
✅ create file with content
✅ Error: directory does not exist
END EXPECTED OUTPUT --]]

function now()
    return exec("date +%s%N").stdout
end

test("create empty file", function()
    local file_path = os.getenv("TESTBOX_TEST_TMP_DIR") .. "/" .. now()
    fs.create_file(file_path)
    expect_equal(fs.read_to_string(file_path), "")
end)

test("create file with content", function()
    local file_path = os.getenv("TESTBOX_TEST_TMP_DIR") .. "/" .. now()
    fs.create_file(file_path, "Testing...")
    expect_equal(fs.read_to_string(file_path), "Testing...")
end)

test("Error: directory does not exist", function()
    expect_equal(
        table.pack(pcall(fs.create_file, "this/directory/must/not/exist.txt")),
        {false, "ERR_DIR_DOES_NOT_EXIST"}
    )
end)
