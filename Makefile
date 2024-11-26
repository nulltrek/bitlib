.PHONY: test
test:
	cargo fmt
	cargo test $(name)

level?=info

.PHONY: test-log
test-log:
	cargo fmt
	RUST_LOG=$(level) cargo test $(name)

.PHONY: test-backtrace
test-backtrace:
	cargo fmt
	RUST_BACKTRACE=1 cargo test

.PHONY: doc
doc:
	cargo doc --document-private-items

.PHONY: build
build:
	cargo fmt
	cargo build
