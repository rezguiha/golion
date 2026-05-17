# Golion

Golion is a multi-market,multi-country,multi-asset optimizer.
It has three crates:

- `golion-domain`: shared domain models.
- `golion-optimization`: optimization models using `good_lp` with HiGHS.
- `golion-server`: HTTP server wiring domain and optimization logic.

## Prerequisites

- `cargo-watch`
- `prek`

Install tools:

```bash
cargo install cargo-watch
cargo install prek
```

## Development workflow

From the repository root:

- `make format` - run `cargo fmt`
- `make lint` - run Clippy across all targets and features
- `make test` - run documentation tests

For fast iteration, consider running in too separate terminals:

- `make server` - launch the server with live reload via `cargo watch`
- `make test-client` - run the server test loop via `cargo watch`
