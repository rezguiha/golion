<p align="center">
  <img src="docs/assets/golion.gif" alt="Golion" width="320">
</p>

<h1 align="center">🤖 Golion</h1>

<p align="center">
  <em>Many assets, countries, and markets combine into one optimization problem.</em>
</p>

---

Just as the lions unite to form one great robot, Golion combines the optimization components
of multiple assets — across countries and markets — into a single optimization problem, and
solves how storage, renewables, and thermal generators should participate.

## 🧩 Crates

- 🔩 `golion-common` — shared units and primitives across all crates.
- 📦 `golion-contract` — serde data models defining the server's request contract.
- ⚡ `golion-optimization` — optimization models using `good_lp` with HiGHS.
- 🌐 `golion-server` — HTTP server wiring the contract and optimization logic.

## ✨ Features

- 🌍 Multi-country, multi-market participation (wholesale + ancillary services).
- 🔋 Battery storage, renewables, and gas turbines as first-class assets.
- 🧠 One combined optimization problem across every asset, country, and market.
- 📈 Linear programming with the HiGHS solver.
- 🦀 Built in Rust (2024 edition).

## 🛠️ Prerequisites

- `cargo-watch`
- `prek`

Install tools:

```bash
cargo install cargo-watch
cargo install prek
```

## 🚀 Development workflow

From the repository root:

- 📐 `make format` — run `cargo fmt`
- 🕵️ `make lint` — run Clippy across all targets and features
- 🧪 `make test` — run documentation and unit tests

For fast iteration, consider running in two separate terminals:

- 👷 `make server` — launch the server with live reload via `cargo watch`
- 🤙 `make test-client` — run the server test loop via `cargo watch`
