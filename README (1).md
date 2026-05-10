# 🛒 Soroban Gig Marketplace

A production-ready, open-source decentralized freelance and gig marketplace protocol built on [Stellar Soroban](https://soroban.stellar.org). Supports both client-posted jobs and freelancer service listings, with built-in escrow, dispute resolution, and payments in USDC or XLM.

[![Stellar](https://img.shields.io/badge/Stellar-Soroban-7B2FBE?style=flat-square&logo=stellar)](https://soroban.stellar.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg?style=flat-square)](LICENSE)
[![Wave Program](https://img.shields.io/badge/Drips-Stellar%20Wave-brightgreen?style=flat-square)](https://www.drips.network/wave/stellar)
[![Contributions Welcome](https://img.shields.io/badge/contributions-welcome-orange.svg?style=flat-square)](CONTRIBUTING.md)

---

## 📦 Contracts

| Contract | Description | Status |
|---|---|---|
| [Marketplace](./contracts/marketplace) | Client posts jobs, freelancers apply and get hired | ✅ Stable |
| [Service Listing](./contracts/service_listing) | Freelancers post services, clients purchase directly | ✅ Stable |
| [Dispute](./contracts/dispute) | Arbiter-based dispute resolution for jobs and orders | ✅ Stable |

---

## 🚀 Quick Start

### Prerequisites

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup target add wasm32-unknown-unknown

# Install Stellar CLI
cargo install --locked stellar-cli --features opt
```

### Build & Test

```bash
git clone https://github.com/YOUR_ORG/soroban-gig-marketplace.git
cd soroban-gig-marketplace
cargo build --target wasm32-unknown-unknown --release
cargo test
```

---

## 📂 Project Structure

```
soroban-gig-marketplace/
├── contracts/
│   ├── marketplace/        # Job board: post, apply, hire, complete
│   ├── service_listing/    # Gig store: list services, purchase, deliver
│   └── dispute/            # Arbitration: open, review, resolve disputes
├── docs/                   # Architecture guides & Wave issues
├── scripts/                # Deploy scripts
└── .github/
    └── workflows/          # CI/CD pipelines
```

---

## 🌍 Use Cases

- **Clients** — Post jobs with budgets locked in escrow; hire from applicants; release payment on completion
- **Freelancers** — List services at fixed prices; receive orders; get paid automatically on acceptance
- **All Categories** — Tech, design, writing, marketing, African gig economy, skilled trades, and more
- **Tokens** — Pay in USDC (stable) or XLM (native Stellar) — any Stellar asset supported

---

## 🤝 Contributing

This project is part of the **Stellar Wave Program** on [Drips](https://www.drips.network/wave/stellar). Contributors earn rewards by resolving tagged issues during Wave sprints.

See [CONTRIBUTING.md](CONTRIBUTING.md) for setup and PR guidelines.

---

## 📄 License

MIT — see [LICENSE](LICENSE) for details.
