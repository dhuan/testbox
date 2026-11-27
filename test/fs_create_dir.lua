--[[ EXPECTED OUTPUT
📁 fs_create_dir.lua
✅ create directory
✅ create parents
✅ Error: directory does not exist
✅ Error: not a directory
✅ Error: parent component is not a directory
END EXPECTED OUTPUT --]]

local function unique_dir_path()
    return os.getenv("TESTBOX_TEST_TMP_DIR") .. "/fs-create-dir-" .. random_chars(12)
end

test("create directory", function()
    local path = unique_dir_path()
    fs.create_dir(path)

    expect_equal(#fs.read_dir(path), 0)

    -- Creating an existing directory succeeds.
    fs.create_dir(path)
end)

test("create parents", function()
    local parent_path = unique_dir_path() .. "/parent"
    local path = parent_path .. "/child"
    fs.create_dir(path, { create_parents = true })

    local entries = fs.read_dir(path)
    expect_equal(#entries, 0)
    local parent_entries = fs.read_dir(parent_path)
    expect_equal(parent_entries[1].name, "child")
    expect_equal(parent_entries[1].kind, "directory")
end)

test("Error: directory does not exist", function()
    local path = unique_dir_path() .. "/child"
    expect_equal(
        table.pack(pcall(fs.create_dir, path)),
        { false, "ERR_DIR_DOES_NOT_EXIST" }
    )
end)

test("Error: not a directory", function()
    expect_equal(
        table.pack(pcall(fs.create_dir, os.getenv("TEST_TMP_FILE"))),
        { false, "ERR_NOT_A_DIR" }
    )
end)

test("Error: parent component is not a directory", function()
    local path = os.getenv("TEST_TMP_FILE") .. "/child"
    expect_equal(
        table.pack(pcall(fs.create_dir, path, { create_parents = true })),
        { false, "ERR_NOT_A_DIR" }
    )
end)
