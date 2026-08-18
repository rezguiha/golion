lint:
	bacon clippy
format:
	cargo fmt
test:
	bacon test
# Command to use in CI as bacon is an interactive
# tool meant more for local development.
test-ci:
	cargo test --workspace --exclude golion-server
server:
	bacon server

test-client:
	bacon test-client
