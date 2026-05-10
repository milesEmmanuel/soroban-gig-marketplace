# Wave Program — GitHub Issues

Copy-paste each issue below into GitHub Issues. Set the labels as indicated.

---

## Issue 1 — Full Test Suite: Marketplace Contract

**Title:** `test: full test suite for marketplace contract`

**Labels:** `good first issue`, `testing`, `wave:medium`, `150pts`

**Description:**

The marketplace contract has basic happy-path tests. This issue tracks expanding coverage to all edge cases and failure paths.

**Tasks:**
- [ ] Test `post_job` with zero budget panics
- [ ] Test `apply_to_job` on a non-Open job panics
- [ ] Test `hire_freelancer` by non-client panics
- [ ] Test `complete_job` on non-InProgress job panics
- [ ] Test `cancel_job` on InProgress job panics
- [ ] Test `flag_dispute` by a non-party panics
- [ ] Test `resolve_dispute` with mismatched award sum panics
- [ ] Test `resolve_dispute` with zero client_award (full to freelancer)
- [ ] Test `get_fee_bps` returns correct value after initialize
- [ ] Verify event emissions for each state transition

**Acceptance Criteria:**
- All new tests pass with `cargo test -p marketplace`
- No existing tests broken
- Coverage includes at least one `#[should_panic]` test per error path

**Complexity:** Medium — **150 points**

---

## Issue 2 — Full Test Suite: Service Listing Contract

**Title:** `test: full test suite for service_listing contract`

**Labels:** `good first issue`, `testing`, `wave:medium`, `150pts`

**Description:**

Expand the service_listing contract test coverage to all edge cases.

**Tasks:**
- [ ] Test `purchase_service` on a Paused service panics
- [ ] Test `purchase_service` on a Deleted service panics
- [ ] Test `deliver_order` by non-freelancer panics
- [ ] Test `accept_delivery` on non-Delivered order panics
- [ ] Test `cancel_order` by a non-party panics
- [ ] Test `toggle_service` on a Deleted service panics
- [ ] Test `update_price` by non-owner panics
- [ ] Test `orders_completed` increments correctly across multiple orders
- [ ] Test fee=0 path (no admin transfer) in `accept_delivery`
- [ ] Verify event emissions for each state transition

**Acceptance Criteria:**
- All new tests pass with `cargo test -p service_listing`
- No existing tests broken
- At least one `#[should_panic]` test per error path

**Complexity:** Medium — **150 points**

---

## Issue 3 — Full Test Suite: Dispute Contract

**Title:** `test: full test suite for dispute contract`

**Labels:** `good first issue`, `testing`, `wave:medium`, `150pts`

**Description:**

Expand the dispute contract test coverage to all edge cases.

**Tasks:**
- [ ] Test `register_arbiter` by non-admin panics
- [ ] Test `assign_review` on non-Open dispute panics
- [ ] Test `resolve` by non-assigned arbiter panics
- [ ] Test `resolve` with mismatched award sum panics
- [ ] Test `resolve` on non-UnderReview dispute panics
- [ ] Test `dismiss` on Resolved dispute panics
- [ ] Test `dismiss` by non-admin panics
- [ ] Test full flow: open → assign → resolve with full claimant award
- [ ] Test full flow: open → dismiss (refund verified)
- [ ] Verify event emissions for each state transition

**Acceptance Criteria:**
- All new tests pass with `cargo test -p dispute`
- No existing tests broken
- At least one `#[should_panic]` test per error path

**Complexity:** Medium — **150 points**

---

## Issue 4 — On-Chain Reputation & Rating System

**Title:** `feat: on-chain reputation and rating system`

**Labels:** `enhancement`, `wave:high`, `200pts`

**Description:**

Add a reputation system so clients and freelancers can rate each other after a completed job or order. Ratings are stored on-chain and aggregated per address.

**Tasks:**
- [ ] Add `rate_job(env, rater, job_id, score: u32, comment: String)` to marketplace (score 1–5)
- [ ] Add `rate_order(env, rater, order_id, score: u32, comment: String)` to service_listing
- [ ] Store `Rating { rater, ratee, score, comment, ledger }` in persistent storage
- [ ] Store per-address aggregate: `ReputationSummary { total_score: u64, count: u64 }`
- [ ] Add `get_reputation(env, address) -> ReputationSummary` view function
- [ ] Only allow rating after Completed status; each party can rate once per job/order
- [ ] Emit `rated` event on each rating
- [ ] Add tests for rating, double-rating prevention, and aggregate calculation

**Acceptance Criteria:**
- Rating functions exist on marketplace and service_listing contracts
- Aggregate reputation queryable per address
- Double-rating by same party on same job/order panics
- All tests pass

**Complexity:** High — **200 points**

---

## Issue 5 — Milestone-Based Payments for Marketplace

**Title:** `feat: milestone-based payment releases in marketplace`

**Labels:** `enhancement`, `wave:high`, `200pts`

**Description:**

Allow clients to split a job budget into milestones. Each milestone can be approved and released independently, enabling incremental payment for large projects.

**Tasks:**
- [ ] Add `Milestone { title: String, amount: i128, released: bool }` type
- [ ] Add optional `milestones: Vec<Milestone>` to `Job` struct
- [ ] Add `add_milestone(env, client, job_id, title, amount)` — only on Open/InProgress jobs
- [ ] Add `release_milestone(env, client, job_id, milestone_index)` — transfers amount to freelancer
- [ ] Validate sum of milestone amounts does not exceed job budget
- [ ] Update `complete_job` to require all milestones released (if any exist)
- [ ] Emit `milestone_released` event per release
- [ ] Add tests for milestone creation, release, over-budget validation, and complete_job guard

**Acceptance Criteria:**
- Milestone functions work end-to-end in tests
- `complete_job` blocked until all milestones released
- Sum validation prevents over-allocation
- All tests pass

**Complexity:** High — **200 points**

---

## Issue 6 — Auto-Complete with Deadline Enforcement

**Title:** `feat: auto-complete and deadline enforcement`

**Labels:** `enhancement`, `wave:high`, `200pts`

**Description:**

Add deadline enforcement so that after `deadline_ledger` passes, anyone can trigger auto-completion (if delivered) or auto-cancellation (if not started), preventing funds from being locked indefinitely.

**Tasks:**
- [ ] Add `auto_complete(env, caller, job_id)` to marketplace: if `env.ledger().sequence() > deadline_ledger` and status is `InProgress`, release budget to freelancer and fee to admin
- [ ] Add `auto_cancel(env, caller, job_id)` to marketplace: if past deadline and status is `Open`, refund client
- [ ] Add `auto_complete(env, caller, order_id)` to service_listing: if past deadline and status is `Delivered`, accept delivery automatically
- [ ] Add `auto_cancel(env, caller, order_id)` to service_listing: if past deadline and status is `InProgress`, refund client
- [ ] Emit `auto_completed` / `auto_cancelled` events
- [ ] Add tests using `env.ledger().set_sequence_number()` to simulate deadline passing

**Acceptance Criteria:**
- Auto-complete and auto-cancel callable by anyone after deadline
- Funds correctly distributed or refunded
- Cannot auto-complete/cancel before deadline
- All tests pass

**Complexity:** High — **200 points**

---

## Issue 7 — Service Listing Tags and Skills Metadata

**Title:** `feat: tags and skills metadata for service listings`

**Labels:** `enhancement`, `wave:medium`, `150pts`

**Description:**

Add a `tags` field to `ServiceListing` so freelancers can label their services with skills (e.g., `["rust", "soroban", "defi"]`). This enables off-chain indexers to filter and search listings.

**Tasks:**
- [ ] Add `tags: Vec<String>` field to `ServiceListing` struct
- [ ] Update `create_service` to accept `tags: Vec<String>` parameter (max 10 tags, each max 32 chars)
- [ ] Add `update_tags(env, freelancer, service_id, tags: Vec<String>)` function
- [ ] Validate tag count and length; panic with descriptive message on violation
- [ ] Emit `tags_updated` event on update
- [ ] Add tests for tag creation, update, and validation errors

**Acceptance Criteria:**
- Tags stored and retrievable via `get_service`
- Validation enforced (max 10 tags, max 32 chars each)
- `update_tags` restricted to service owner
- All tests pass

**Complexity:** Medium — **150 points**

---

## Issue 8 — Platform Fee Withdrawal Function

**Title:** `feat: admin fee withdrawal function`

**Labels:** `enhancement`, `wave:medium`, `150pts`

**Description:**

Currently fees are sent directly to admin on each transaction. Add an alternative accumulation mode where fees are held in the contract and withdrawn by admin in a single call, reducing admin transaction overhead.

**Tasks:**
- [ ] Add `FEE_POOL` storage key per token to track accumulated fees in each contract
- [ ] Add `withdraw_fees(env, admin, token)` function to marketplace, service_listing, and dispute contracts
- [ ] Only admin can call `withdraw_fees`; transfers entire pool balance to admin
- [ ] Add `get_fee_pool(env, token) -> i128` view function
- [ ] Emit `fees_withdrawn` event with amount
- [ ] Add tests for fee accumulation and withdrawal

**Acceptance Criteria:**
- `withdraw_fees` transfers correct accumulated amount to admin
- `get_fee_pool` returns correct balance before and after withdrawal
- Non-admin call panics
- All tests pass

**Complexity:** Medium — **150 points**

---

## Issue 9 — Doc Comments for All Contracts

**Title:** `docs: add rustdoc comments to all public functions and types`

**Labels:** `documentation`, `good first issue`, `wave:trivial`, `100pts`

**Description:**

All public functions, structs, and enums across the three contracts lack `///` rustdoc comments. This issue tracks adding complete documentation.

**Tasks:**
- [ ] Add `///` doc comments to all public functions in `contracts/marketplace/src/lib.rs`
- [ ] Add `///` doc comments to all public functions in `contracts/service_listing/src/lib.rs`
- [ ] Add `///` doc comments to all public functions in `contracts/dispute/src/lib.rs`
- [ ] Document all public structs and their fields
- [ ] Document all enum variants
- [ ] Verify `cargo doc --workspace --no-deps` builds without warnings

**Acceptance Criteria:**
- `cargo doc --workspace --no-deps` produces zero warnings
- Every public item has a doc comment
- Comments describe parameters, panics, and return values

**Complexity:** Trivial — **100 points**

---

## Issue 10 — Testnet Deploy Script with Arbiter Registration

**Title:** `chore: testnet deployment script with arbiter registration`

**Labels:** `tooling`, `wave:medium`, `150pts`

**Description:**

Add a shell script that deploys all three contracts to Stellar Testnet using the Stellar CLI, initializes them, and registers a test arbiter in the dispute contract.

**Tasks:**
- [ ] Create `scripts/deploy_testnet.sh`
- [ ] Script generates or loads a funded testnet keypair via `stellar keys generate`
- [ ] Deploys marketplace, service_listing, and dispute contracts with `stellar contract deploy`
- [ ] Calls `initialize` on each contract with admin address and fee_bps
- [ ] Calls `register_arbiter` on dispute contract with a test arbiter address
- [ ] Saves deployed contract IDs to `scripts/deployed_ids.env`
- [ ] Add `scripts/README.md` explaining how to run the script and required env vars
- [ ] Test the script end-to-end on Testnet and document the contract IDs

**Acceptance Criteria:**
- Script runs end-to-end without errors on Stellar Testnet
- All three contracts initialized and queryable
- Arbiter registered and verifiable via `get_dispute` (or direct storage query)
- `deployed_ids.env` populated with correct contract IDs
- `scripts/README.md` documents usage

**Complexity:** Medium — **150 points**
