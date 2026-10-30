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

## 🦁 What makes Golion different

Golion models how a portfolio is traded and settled, not only how each asset dispatches.

- 🏢 **BRP and BSP perimeters.** Assets are grouped the way the market sees them. A
  balance responsible party (BRP) perimeter trades the net position of its assets on
  wholesale markets and settles imbalance as a whole, so its assets can cover for each
  other. A balancing service provider (BSP) perimeter bids reserve for the assets
  certified together, and the optimizer splits each award across them. An asset belongs
  to at most one BRP but can serve several reserves. Market rules are enforced at the
  perimeter level, where bids are placed: reserve bids are whole 1 MW increments and FCR
  bids are symmetric for the perimeter, while each asset's share stays free.
- 👮‍♂️ **Hard physics, penalized positions.** Physical limits are strict constraints:
  power, state of charge, reserve headroom and energy, and cycling. Market positions are
  penalized instead: a commitment the assets cannot deliver becomes a priced imbalance or
  reserve shortage, reported in the output, rather than an infeasible problem.
- 🫐 **Multi-product wholesale bidding.** A BRP can trade several products of the same
  market at once and co-optimize them on one net position, for example 15 min, 30 min,
  and 1 h products on continuous intraday.
- 🔒 **Deliverable reserves.** Every reserve keeps the power headroom and the energy for a
  full activation over its activation window, on top of the planned dispatch. Expected
  activations count toward the rolling 24 h cycle limit.

## 🧩 Crates

- 🔩 `golion-domain` — shared units and primitives across all crates.
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

- `bacon`
- `prek`

Install tools:

```bash
cargo install --locked bacon
cargo install prek
```

## 🚀 Development workflow

From the repository root:

- 📐 `make format` — run `cargo fmt`
- 🕵️ `make lint` — run Clippy across all targets and features
- 🧪 `make test` — run documentation and unit tests

For fast iteration, consider running in two separate terminals:

- 👷 `make server` — launch the server with live reload via `bacon`
- 🤙 `make test-client` — run the server test loop via `bacon`
