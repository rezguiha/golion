# Local settings shared with docker/compose.yaml, bacon and cargo.
-include .env
export

lint:
	bacon clippy
format:
	cargo fmt
test:
	bacon test
# Command to use in CI as bacon is an interactive
# tool meant more for local development.
test-ci:
	cargo test --workspace --exclude web-server
server:
	bacon server

test-client:
	bacon test-client

# Runs the stack and rebuilds it when the sources change.
up:
	docker compose -f docker/compose.yaml --env-file .env up --watch
