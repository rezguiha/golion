lint:
	bacon clippy
format:
	cargo fmt
test:
	bacon test

server:
	bacon server

test-client:
	bacon test-client
