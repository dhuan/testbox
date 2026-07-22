build:
	cargo build --profile release

test:
	sh test/test.sh

.PHONY: test
