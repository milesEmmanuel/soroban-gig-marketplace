#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, token, Address, Env, String, Symbol,
};

// ── Storage keys ─────────────────────────────────────────────────────────────

const ADMIN: Symbol = symbol_short!("ADMIN");
const DISP_CNT: Symbol = symbol_short!("DISP_CNT");

fn dispute_key(id: u64) -> (Symbol, u64) {
    (symbol_short!("DISP"), id)
}
fn arbiter_key(addr: &Address) -> (Symbol, Address) {
    (symbol_short!("ARB"), addr.clone())
}

// ── Types ─────────────────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, PartialEq)]
pub enum DisputeStatus {
    Open,
    UnderReview,
    Resolved,
    Dismissed,
}

#[contracttype]
#[derive(Clone, PartialEq)]
pub enum DisputeType {
    JobDispute,
    OrderDispute,
}

#[contracttype]
#[derive(Clone)]
pub struct Dispute {
    pub dispute_type: DisputeType,
    pub reference_id: u64,
    pub claimant: Address,
    pub respondent: Address,
    pub token: Address,
    pub disputed_amount: i128,
    pub reason: String,
    pub evidence: String,
    pub arbiter: Option<Address>,
    pub claimant_award: i128,
    pub respondent_award: i128,
    pub resolved_ledger: u32,
    pub status: DisputeStatus,
}

// ── Contract ──────────────────────────────────────────────────────────────────

#[contract]
pub struct DisputeContract;

#[contractimpl]
impl DisputeContract {
    pub fn initialize(env: Env, admin: Address) {
        if env.storage().instance().has(&ADMIN) {
            panic!("already initialized");
        }
        admin.require_auth();
        env.storage().instance().set(&ADMIN, &admin);
        env.storage().instance().set(&DISP_CNT, &0u64);
    }

    /// Admin registers a trusted arbiter.
    pub fn register_arbiter(env: Env, admin: Address, arbiter: Address) {
        admin.require_auth();
        let stored: Address = env.storage().instance().get(&ADMIN).unwrap();
        if admin != stored {
            panic!("not admin");
        }
        env.storage().persistent().set(&arbiter_key(&arbiter), &true);

        env.events().publish(
            (symbol_short!("arb_reg"), admin),
            arbiter,
        );
    }

    /// Claimant opens a dispute; disputed_amount locked from claimant.
    pub fn open_dispute(
        env: Env,
        claimant: Address,
        respondent: Address,
        token: Address,
        disputed_amount: i128,
        dispute_type: DisputeType,
        reference_id: u64,
        reason: String,
        evidence: String,
    ) -> u64 {
        claimant.require_auth();
        token::Client::new(&env, &token).transfer(
            &claimant,
            &env.current_contract_address(),
            &disputed_amount,
        );

        let id: u64 = env.storage().instance().get(&DISP_CNT).unwrap();
        let dispute = Dispute {
            dispute_type,
            reference_id,
            claimant: claimant.clone(),
            respondent: respondent.clone(),
            token: token.clone(),
            disputed_amount,
            reason,
            evidence,
            arbiter: None,
            claimant_award: 0,
            respondent_award: 0,
            resolved_ledger: 0,
            status: DisputeStatus::Open,
        };
        env.storage().persistent().set(&dispute_key(id), &dispute);
        env.storage().instance().set(&DISP_CNT, &(id + 1));

        env.events().publish(
            (symbol_short!("disp_opn"), claimant),
            (id, respondent, token, disputed_amount),
        );
        id
    }

    /// Assigned arbiter self-assigns review of an Open dispute.
    pub fn assign_review(env: Env, arbiter: Address, dispute_id: u64) {
        arbiter.require_auth();
        if !env.storage().persistent().get::<_, bool>(&arbiter_key(&arbiter)).unwrap_or(false) {
            panic!("not a registered arbiter");
        }
        let mut dispute: Dispute =
            env.storage().persistent().get(&dispute_key(dispute_id)).expect("dispute not found");
        if dispute.status != DisputeStatus::Open {
            panic!("dispute not open");
        }
        dispute.arbiter = Some(arbiter.clone());
        dispute.status = DisputeStatus::UnderReview;
        env.storage().persistent().set(&dispute_key(dispute_id), &dispute);

        env.events().publish(
            (symbol_short!("disp_rev"), arbiter),
            dispute_id,
        );
    }

    /// Assigned arbiter resolves; awards must sum to disputed_amount.
    pub fn resolve(
        env: Env,
        arbiter: Address,
        dispute_id: u64,
        claimant_award: i128,
        respondent_award: i128,
    ) {
        arbiter.require_auth();
        let mut dispute: Dispute =
            env.storage().persistent().get(&dispute_key(dispute_id)).expect("dispute not found");
        if dispute.arbiter.as_ref() != Some(&arbiter) {
            panic!("not assigned arbiter");
        }
        if dispute.status != DisputeStatus::UnderReview {
            panic!("not under review");
        }
        if claimant_award + respondent_award != dispute.disputed_amount {
            panic!("awards must sum to disputed_amount");
        }
        let tok = token::Client::new(&env, &dispute.token);
        if claimant_award > 0 {
            tok.transfer(&env.current_contract_address(), &dispute.claimant, &claimant_award);
        }
        if respondent_award > 0 {
            tok.transfer(&env.current_contract_address(), &dispute.respondent, &respondent_award);
        }
        dispute.claimant_award = claimant_award;
        dispute.respondent_award = respondent_award;
        dispute.resolved_ledger = env.ledger().sequence();
        dispute.status = DisputeStatus::Resolved;
        env.storage().persistent().set(&dispute_key(dispute_id), &dispute);

        env.events().publish(
            (symbol_short!("disp_res"), arbiter),
            (dispute_id, claimant_award, respondent_award),
        );
    }

    /// Admin dismisses an Open or UnderReview dispute; refunds claimant.
    pub fn dismiss(env: Env, admin: Address, dispute_id: u64) {
        admin.require_auth();
        let stored: Address = env.storage().instance().get(&ADMIN).unwrap();
        if admin != stored {
            panic!("not admin");
        }
        let mut dispute: Dispute =
            env.storage().persistent().get(&dispute_key(dispute_id)).expect("dispute not found");
        if dispute.status != DisputeStatus::Open && dispute.status != DisputeStatus::UnderReview {
            panic!("cannot dismiss");
        }
        token::Client::new(&env, &dispute.token).transfer(
            &env.current_contract_address(),
            &dispute.claimant,
            &dispute.disputed_amount,
        );
        dispute.status = DisputeStatus::Dismissed;
        env.storage().persistent().set(&dispute_key(dispute_id), &dispute);

        env.events().publish(
            (symbol_short!("disp_dis"), admin),
            dispute_id,
        );
    }

    // ── Views ─────────────────────────────────────────────────────────────────

    pub fn get_dispute(env: Env, dispute_id: u64) -> Dispute {
        env.storage().persistent().get(&dispute_key(dispute_id)).expect("dispute not found")
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{
        token::{Client as TokenClient, StellarAssetClient},
        Address, Env, String,
    };

    fn setup() -> (Env, Address, Address, Address, Address) {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        let claimant = Address::generate(&env);
        let respondent = Address::generate(&env);
        let token = env.register_stellar_asset_contract_v2(admin.clone()).address();
        (env, admin, claimant, respondent, token)
    }

    #[test]
    fn test_open_assign_resolve() {
        let (env, admin, claimant, respondent, token) = setup();
        let cid = env.register(DisputeContract, ());
        let dc = DisputeContractClient::new(&env, &cid);

        dc.initialize(&admin);
        let arbiter = Address::generate(&env);
        dc.register_arbiter(&admin, &arbiter);

        StellarAssetClient::new(&env, &token).mint(&claimant, &10_000);

        let disp_id = dc.open_dispute(
            &claimant,
            &respondent,
            &token,
            &10_000i128,
            &DisputeType::JobDispute,
            &0u64,
            &String::from_str(&env, "Work not done"),
            &String::from_str(&env, "Screenshot"),
        );

        dc.assign_review(&arbiter, &disp_id);
        dc.resolve(&arbiter, &disp_id, &7_000i128, &3_000i128);

        assert_eq!(TokenClient::new(&env, &token).balance(&claimant), 7_000);
        assert_eq!(TokenClient::new(&env, &token).balance(&respondent), 3_000);
        assert_eq!(dc.get_dispute(&disp_id).status, DisputeStatus::Resolved);
    }

    #[test]
    fn test_dismiss_refunds_claimant() {
        let (env, admin, claimant, respondent, token) = setup();
        let cid = env.register(DisputeContract, ());
        let dc = DisputeContractClient::new(&env, &cid);
        dc.initialize(&admin);
        StellarAssetClient::new(&env, &token).mint(&claimant, &5_000);
        let disp_id = dc.open_dispute(
            &claimant,
            &respondent,
            &token,
            &5_000i128,
            &DisputeType::OrderDispute,
            &1u64,
            &String::from_str(&env, "reason"),
            &String::from_str(&env, "evidence"),
        );
        dc.dismiss(&admin, &disp_id);
        assert_eq!(TokenClient::new(&env, &token).balance(&claimant), 5_000);
        assert_eq!(dc.get_dispute(&disp_id).status, DisputeStatus::Dismissed);
    }

    #[test]
    #[should_panic(expected = "not a registered arbiter")]
    fn test_unregistered_arbiter_cannot_assign() {
        let (env, admin, claimant, respondent, token) = setup();
        let cid = env.register(DisputeContract, ());
        let dc = DisputeContractClient::new(&env, &cid);
        dc.initialize(&admin);
        StellarAssetClient::new(&env, &token).mint(&claimant, &1_000);
        let disp_id = dc.open_dispute(
            &claimant,
            &respondent,
            &token,
            &1_000i128,
            &DisputeType::JobDispute,
            &0u64,
            &String::from_str(&env, "r"),
            &String::from_str(&env, "e"),
        );
        let fake = Address::generate(&env);
        dc.assign_review(&fake, &disp_id);
    }
}
