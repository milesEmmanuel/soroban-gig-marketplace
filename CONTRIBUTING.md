# Contributing to soroban-gig-marketplace

Thank you for your interest in contributing! This project participates in the [Stellar Wave Program](https://www.drips.network/wave/stellar) — contributions earn points redeemable through Drips.

---

## Dev Setup

```bash
# 1. Install Rust stable
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup default stable

# 2. Add wasm32 target
rustup target add wasm32-unknown-unknown

# 3. Install Stellar CLI
cargo install --locked stellar-cli --features opt

# 4. Fork & clone
git clone https://github.com/YOUR_USERNAME/soroban-gig-marketplace.git
cd soroban-gig-marketplace

# 5. Build
cargo build --workspace

# 6. Test
cargo test --workspace

# 7. Lint
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
```

---

## Wave Bounty Table

Issues are labeled with complexity and point values. Points are submitted to the Stellar Wave Program at https://www.drips.network/wave/stellar.

| Complexity | Points | Examples |
|---|---|---|
| **Trivial** | 100 pts | Doc comments, minor fixes, typos |
| **Medium** | 150 pts | New features, test suites, scripts |
| **High** | 200 pts | Architecture changes, new subsystems |

See [docs/WAVE_ISSUES.md](docs/WAVE_ISSUES.md) for the full list of open bounty issues.

---

## PR Guidelines

1. **Branch naming**: `feat/short-description`, `fix/short-description`, `docs/short-description`
2. **One PR per issue** — keep changes focused
3. **Tests required** — all new logic must have corresponding tests
4. **No warnings** — CI runs `clippy -D warnings`; fix all warnings before opening a PR
5. **Format** — run `cargo fmt --all` before committing
6. **Commit messages** — use conventional commits: `feat:`, `fix:`, `docs:`, `test:`, `chore:`

### PR Description Template

```
## Summary
Brief description of what this PR does.

## Related Issue
Closes #<issue_number>

## Changes
- List of changes

## Testing
How you tested the changes.

## Wave Points
Complexity: Trivial / Medium / High
Points: 100 / 150 / 200
```

---

## Test Commands

```bash
# Run all tests
cargo test --workspace

# Run tests for a specific contract
cargo test -p marketplace
cargo test -p service_listing
cargo test -p dispute

# Run with output
cargo test --workspace -- --nocapture
```

## Lint Commands

```bash
# Format check
cargo fmt --all -- --check

# Clippy (strict)
cargo clippy --workspace --all-targets -- -D warnings

# Auto-fix formatting
cargo fmt --all
```

---

## Code of Conduct

Be respectful, inclusive, and constructive. We follow the [Contributor Covenant](https://www.contributor-covenant.org/).
