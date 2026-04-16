extract_expected_output () {
    awk '
BEGIN { PRINT = 0 }
{ if ($0 == "--[[ EXPECTED OUTPUT") { PRINT = 1; next; } }
{ if ($0 == "END EXPECTED OUTPUT --]]") { exit 0 } }
{ if (PRINT == 1) { print } }
    '
}

OUTPUT_STDERR=$(mktemp)

ls test/*.lua | while read TEST_FILE
do
    echo $TEST_FILE

    OUTPUT=$(./target/debug/testbox "${TEST_FILE}" 2> "${OUTPUT_STDERR}")

    EXPECTED_OUTPUT=$(cat "${TEST_FILE}" | extract_expected_output)

    if [ "${OUTPUT}" != "${EXPECTED_OUTPUT}" ]
    then
        printf "\n---\nExpected output:\n%s\n---\nActual output:\n%s\nSTDERR:\n%s\n" "${EXPECTED_OUTPUT}" "${OUTPUT}" "$(cat $OUTPUT_STDERR)"

        exit 1
    fi

    echo OK
done
