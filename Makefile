lint:
	cargo clippy --all-targets --all-features -- -D warnings
format:
	cargo fmt
test:
	cargo test --doc

server:
	cargo watch -q -c -w crates/ -x run

test-client:
	cargo watch -q -c -w crates/golion-server/tests/ -x 'test --package golion-server --test quick_dev -- --nocapture'
