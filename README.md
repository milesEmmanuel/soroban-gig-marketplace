# soroban-gig-marketplace

[![Stellar](https://img.shields.io/badge/Stellar-Network-blue?logo=stellar)](https://stellar.org)
[![Soroban](https://img.shields.io/badge/Soroban-SDK%20v21-blueviolet)](https://soroban.stellar.org)
[![Wave Program](https://img.shields.io/badge/Drips-Stellar%20Wave-orange)](https://www.drips.network/wave/stellar)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![CI](https://github.com/YOUR_USERNAME/soroban-gig-marketplace/actions/workflows/ci.yml/badge.svg)](https://github.com/YOUR_USERNAME/soroban-gig-marketplace/actions/workflows/ci.yml)

A production-ready, fully on-chain decentralized freelance and gig marketplace protocol built with [Soroban](https://soroban.stellar.org) smart contracts on the Stellar network.

---

## Contracts

| Contract | Path | Status | Description |
|---|---|---|---|
| **Marketplace** | `contracts/marketplace` | ✅ Production | Client-posts-job hiring model with escrow, disputes, and fee splitting |
| **Service Listing** | `contracts/service_listing` | ✅ Production | Freelancer-posts-service model with order lifecycle management |
| **Dispute** | `contracts/dispute` | ✅ Production | Standalone arbitration contract with registered arbiters |

---

## Quick Start

### Prerequisites

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Add wasm32 target
rustup target add wasm32-unknown-unknown

# Install Stellar CLI
cargo install --locked stellar-cli --features opt
```

### Clone & Build

```bash
git clone https://github.com/YOUR_USERNAME/soroban-gig-marketplace.git
cd soroban-gig-marketplace

# Build all contracts (native, for tests)
cargo build --workspace

# Build all contracts (wasm, for deployment)
cargo build --workspace --target wasm32-unknown-unknown --release

# Run all tests
cargo test --workspace
```

---

## Project Structure

```
soroban-gig-marketplace/
├── Cargo.toml                          # Workspace root
├── contracts/
│   ├── marketplace/
│   │   ├── Cargo.toml
│   │   └── src/lib.rs                  # Marketplace contract
│   ├── service_listing/
│   │   ├── Cargo.toml
│   │   └── src/lib.rs                  # Service listing contract
│   └── dispute/
│       ├── Cargo.toml
│       └── src/lib.rs                  # Dispute/arbitration contract
├── docs/
│   └── WAVE_ISSUES.md                  # GitHub issues for Wave Program
├── .github/
│   └── workflows/
│       └── ci.yml                      # CI: test + lint
├── CONTRIBUTING.md
└── LICENSE
```

---

## Use Cases

### For Clients
- Post jobs with USDC or XLM escrow locked on-chain
- Browse and hire from freelancer applications
- Raise disputes if work is unsatisfactory

### For Freelancers
- List services with fixed pricing
- Apply to client-posted jobs with custom proposals
- Deliver work and receive instant on-chain payment

### All Categories
- Software Development
- Design & Creative
- Writing & Translation
- Marketing & SEO
- Data & Analytics
- Any custom category

### Supported Tokens
| Token | Type |
|---|---|
| USDC | `TokenType::USDC` |
| XLM | `TokenType::XLM` |
| Any SAC | `TokenType::Custom` |

---

## Architecture

```
Client ──post_job──► Marketplace ──escrow──► Token Contract
                         │
                    hire_freelancer
                         │
                    complete_job ──► Freelancer (budget) + Admin (fee)
                         │
                    flag_dispute ──► Dispute Contract (optional)
```

---

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). This project participates in the [Stellar Wave Program](https://www.drips.network/wave/stellar) — open issues are tagged with point values for contributors.

---

## License

[MIT](LICENSE) © 2026
