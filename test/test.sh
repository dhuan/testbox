extract_expected_output () {
    awk '
BEGIN { PRINT = 0 }
{ if ($0 == "--[[ EXPECTED OUTPUT") { PRINT = 1; next; } }
{ if ($0 == "END EXPECTED OUTPUT --]]") { exit 0 } }
{ if (PRINT == 1) { print } }
    '
}

extract_test_options () {
    awk '
/^-- OPTIONS:/ {
    sub(/-- OPTIONS: /, "", $0)

    print;
}
'
}

extract_expected_exit_code () {
    awk '
/^-- EXIT CODE:/ {
    sub(/-- EXIT CODE: /, "", $0)

    print;
}
'
}

if ! which mock 2> /dev/null
then
    echo "Missing program: mock"

    exit 1
fi

OUTPUT_STDERR=$(mktemp)

TEST_FILES="$(ls test/*.lua)"
if [ -n "${TEST_FILE}" ]
then
    TEST_FILES="$(printf "%s" "${TEST_FILES}" | grep "${TEST_FILE}" | awk '1')"
fi

if [ -z "${TEST_FILES}" ]
then
    echo "No test files found. Aborting."

    exit 0
fi

export TEST_TMP_FILE=$(mktemp)
export TEST_TMP_DIR=$(mktemp -d)
export TEST_TMP_FILE_CONTENT="Hello, world."
printf "%s" "${TEST_TMP_FILE_CONTENT}" > "${TEST_TMP_FILE}"

printf "%s\n" "${TEST_FILES}" | while read -r TEST_FILE
do
    echo $TEST_FILE

    TEST_OPTIONS="$(cat "${TEST_FILE}" | extract_test_options)"
    EXPECTED_EXIT_CODE="$(cat "${TEST_FILE}" | extract_expected_exit_code)"
    if [ -z "${EXPECTED_EXIT_CODE}" ]
    then
        EXPECTED_EXIT_CODE=0
    fi

    COMMAND="./target/debug/testbox $TEST_OPTIONS "${TEST_FILE}""

    OUTPUT=$(eval $COMMAND 2> "${OUTPUT_STDERR}")
    EXIT_CODE=$?

    EXPECTED_OUTPUT=$(cat "${TEST_FILE}" | extract_expected_output)

    if [ "${OUTPUT}" != "${EXPECTED_OUTPUT}" ] || [ "${EXIT_CODE}" != "${EXPECTED_EXIT_CODE}" ]
    then
        printf "\n---\nExpected exit code:\n%s\n---\nActual exit code:\n%s\n---\nExpected output:\n%s\n---\nActual output:\n%s\nSTDERR:\n%s\n" "${EXPECTED_EXIT_CODE}" "${EXIT_CODE}" "${EXPECTED_OUTPUT}" "${OUTPUT}" "$(cat $OUTPUT_STDERR)"

        exit 1
    fi

    echo OK
done
