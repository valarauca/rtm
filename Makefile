.PHONY: test

test:
	RUSTFLAGS=-Ctarget-feature=+rtm cargo test --features std
	RUSTFLAGS=-Ctarget-feature=+rtm cargo test
	RUSTFLAGS=-Ctarget-feature=-rtm cargo test --features std
	RUSTFLAGS=-Ctarget-feature=-rtm cargo test

