#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, token, Address, Env, String, Symbol,
};

// ── Storage keys ────────────────────────────────────────────────────────────

const ADMIN: Symbol = symbol_short!("ADMIN");
const FEE_BPS: Symbol = symbol_short!("FEE_BPS");
const JOB_CNT: Symbol = symbol_short!("JOB_CNT");
const APP_CNT: Symbol = symbol_short!("APP_CNT");

fn job_key(id: u64) -> (Symbol, u64) {
    (symbol_short!("JOB"), id)
}
fn app_key(id: u64) -> (Symbol, u64) {
    (symbol_short!("APP"), id)
}

// ── Types ────────────────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, PartialEq)]
pub enum JobStatus {
    Open,
    InProgress,
    Completed,
    Cancelled,
    Disputed,
}

#[contracttype]
#[derive(Clone, PartialEq)]
pub enum ApplicationStatus {
    Pending,
    Accepted,
    Rejected,
}

#[contracttype]
#[derive(Clone, PartialEq)]
pub enum TokenType {
    USDC,
    XLM,
    Custom,
}

#[contracttype]
#[derive(Clone)]
pub struct Job {
    pub client: Address,
    pub freelancer: Option<Address>,
    pub token: Address,
    pub token_type: TokenType,
    pub budget: i128,
    pub fee: i128,
    pub title: String,
    pub description: String,
    pub category: String,
    pub deadline_ledger: u32,
    pub status: JobStatus,
}

#[contracttype]
#[derive(Clone)]
pub struct Application {
    pub job_id: u64,
    pub applicant: Address,
    pub proposal: String,
    pub bid_amount: i128,
    pub status: ApplicationStatus,
}

// ── Contract ─────────────────────────────────────────────────────────────────

#[contract]
pub struct MarketplaceContract;

#[contractimpl]
impl MarketplaceContract {
    /// One-time initializer. Sets admin and platform fee in basis points.
    pub fn initialize(env: Env, admin: Address, fee_bps: u32) {
        if env.storage().instance().has(&ADMIN) {
            panic!("already initialized");
        }
        admin.require_auth();
        env.storage().instance().set(&ADMIN, &admin);
        env.storage().instance().set(&FEE_BPS, &fee_bps);
        env.storage().instance().set(&JOB_CNT, &0u64);
        env.storage().instance().set(&APP_CNT, &0u64);
    }

    /// Client posts a job; budget + fee locked in contract.
    pub fn post_job(
        env: Env,
        client: Address,
        token: Address,
        token_type: TokenType,
        budget: i128,
        title: String,
        description: String,
        category: String,
        deadline_ledger: u32,
    ) -> u64 {
        client.require_auth();
        let fee_bps: u32 = env.storage().instance().get(&FEE_BPS).unwrap();
        let fee = budget * fee_bps as i128 / 10_000;
        let total = budget + fee;

        token::Client::new(&env, &token).transfer(&client, &env.current_contract_address(), &total);

        let id: u64 = env.storage().instance().get(&JOB_CNT).unwrap();
        let job = Job {
            client: client.clone(),
            freelancer: None,
            token: token.clone(),
            token_type,
            budget,
            fee,
            title,
            description,
            category,
            deadline_ledger,
            status: JobStatus::Open,
        };
        env.storage().persistent().set(&job_key(id), &job);
        env.storage().instance().set(&JOB_CNT, &(id + 1));

        env.events().publish(
            (symbol_short!("job_post"), client),
            (id, token, budget),
        );
        id
    }

    /// Freelancer applies to an open job.
    pub fn apply_to_job(
        env: Env,
        applicant: Address,
        job_id: u64,
        proposal: String,
        bid_amount: i128,
    ) -> u64 {
        applicant.require_auth();
        let job: Job = env.storage().persistent().get(&job_key(job_id)).expect("job not found");
        if job.status != JobStatus::Open {
            panic!("job not open");
        }

        let app_id: u64 = env.storage().instance().get(&APP_CNT).unwrap();
        let app = Application {
            job_id,
            applicant: applicant.clone(),
            proposal,
            bid_amount,
            status: ApplicationStatus::Pending,
        };
        env.storage().persistent().set(&app_key(app_id), &app);
        env.storage().instance().set(&APP_CNT, &(app_id + 1));

        env.events().publish(
            (symbol_short!("applied"), applicant),
            (app_id, job_id, bid_amount),
        );
        app_id
    }

    /// Client accepts an application, moving job to InProgress.
    pub fn hire_freelancer(env: Env, client: Address, job_id: u64, app_id: u64) {
        client.require_auth();
        let mut job: Job = env.storage().persistent().get(&job_key(job_id)).expect("job not found");
        if job.client != client {
            panic!("not client");
        }
        if job.status != JobStatus::Open {
            panic!("job not open");
        }
        let mut app: Application =
            env.storage().persistent().get(&app_key(app_id)).expect("app not found");
        if app.job_id != job_id {
            panic!("app/job mismatch");
        }
        app.status = ApplicationStatus::Accepted;
        job.freelancer = Some(app.applicant.clone());
        job.status = JobStatus::InProgress;

        env.storage().persistent().set(&job_key(job_id), &job);
        env.storage().persistent().set(&app_key(app_id), &app);

        env.events().publish(
            (symbol_short!("hired"), client),
            (job_id, app.applicant),
        );
    }

    /// Client marks job complete; budget released to freelancer, fee to admin.
    pub fn complete_job(env: Env, client: Address, job_id: u64) {
        client.require_auth();
        let mut job: Job = env.storage().persistent().get(&job_key(job_id)).expect("job not found");
        if job.client != client {
            panic!("not client");
        }
        if job.status != JobStatus::InProgress {
            panic!("not in progress");
        }
        let freelancer = job.freelancer.clone().expect("no freelancer");
        let admin: Address = env.storage().instance().get(&ADMIN).unwrap();
        let tok = token::Client::new(&env, &job.token);
        tok.transfer(&env.current_contract_address(), &freelancer, &job.budget);
        tok.transfer(&env.current_contract_address(), &admin, &job.fee);
        job.status = JobStatus::Completed;
        env.storage().persistent().set(&job_key(job_id), &job);

        env.events().publish(
            (symbol_short!("job_done"), client),
            (job_id, freelancer, job.budget),
        );
    }

    /// Client cancels an Open job; full refund (budget + fee) to client.
    pub fn cancel_job(env: Env, client: Address, job_id: u64) {
        client.require_auth();
        let mut job: Job = env.storage().persistent().get(&job_key(job_id)).expect("job not found");
        if job.client != client {
            panic!("not client");
        }
        if job.status != JobStatus::Open {
            panic!("can only cancel open jobs");
        }
        let refund = job.budget + job.fee;
        token::Client::new(&env, &job.token).transfer(
            &env.current_contract_address(),
            &client,
            &refund,
        );
        job.status = JobStatus::Cancelled;
        env.storage().persistent().set(&job_key(job_id), &job);

        env.events().publish(
            (symbol_short!("job_cncl"), client),
            (job_id, refund),
        );
    }

    /// Client or freelancer flags a dispute on an InProgress job.
    pub fn flag_dispute(env: Env, caller: Address, job_id: u64) {
        caller.require_auth();
        let mut job: Job = env.storage().persistent().get(&job_key(job_id)).expect("job not found");
        let freelancer = job.freelancer.clone().expect("no freelancer");
        if caller != job.client && caller != freelancer {
            panic!("not a party");
        }
        if job.status != JobStatus::InProgress {
            panic!("not in progress");
        }
        job.status = JobStatus::Disputed;
        env.storage().persistent().set(&job_key(job_id), &job);

        env.events().publish(
            (symbol_short!("disputed"), caller),
            job_id,
        );
    }

    /// Admin resolves a dispute; client_award + freelancer_award must equal budget + fee.
    pub fn resolve_dispute(
        env: Env,
        admin: Address,
        job_id: u64,
        client_award: i128,
        freelancer_award: i128,
    ) {
        admin.require_auth();
        let stored_admin: Address = env.storage().instance().get(&ADMIN).unwrap();
        if admin != stored_admin {
            panic!("not admin");
        }
        let mut job: Job = env.storage().persistent().get(&job_key(job_id)).expect("job not found");
        if job.status != JobStatus::Disputed {
            panic!("not disputed");
        }
        let total = job.budget + job.fee;
        if client_award + freelancer_award != total {
            panic!("awards must sum to budget+fee");
        }
        let freelancer = job.freelancer.clone().expect("no freelancer");
        let tok = token::Client::new(&env, &job.token);
        if client_award > 0 {
            tok.transfer(&env.current_contract_address(), &job.client, &client_award);
        }
        if freelancer_award > 0 {
            tok.transfer(&env.current_contract_address(), &freelancer, &freelancer_award);
        }
        job.status = JobStatus::Completed;
        env.storage().persistent().set(&job_key(job_id), &job);

        env.events().publish(
            (symbol_short!("resolved"), admin),
            (job_id, client_award, freelancer_award),
        );
    }

    // ── Views ────────────────────────────────────────────────────────────────

    pub fn get_job(env: Env, job_id: u64) -> Job {
        env.storage().persistent().get(&job_key(job_id)).expect("job not found")
    }

    pub fn get_application(env: Env, app_id: u64) -> Application {
        env.storage().persistent().get(&app_key(app_id)).expect("app not found")
    }

    pub fn get_fee_bps(env: Env) -> u32 {
        env.storage().instance().get(&FEE_BPS).unwrap()
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{
        testutils::{Address as _, AuthorizedFunction, AuthorizedInvocation},
        token::{Client as TokenClient, StellarAssetClient},
        Address, Env, String,
    };

    fn setup() -> (Env, Address, Address, Address, Address) {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        let client_addr = Address::generate(&env);
        let freelancer_addr = Address::generate(&env);
        let token_addr = env.register_stellar_asset_contract_v2(admin.clone()).address();
        (env, admin, client_addr, freelancer_addr, token_addr)
    }

    #[test]
    fn test_post_and_apply_and_hire_and_complete() {
        let (env, admin, client_addr, freelancer_addr, token_addr) = setup();
        let contract_id = env.register(MarketplaceContract, ());
        let mp = MarketplaceContractClient::new(&env, &contract_id);

        mp.initialize(&admin, &250u32); // 2.5% fee

        // Mint tokens to client
        StellarAssetClient::new(&env, &token_addr).mint(&client_addr, &11_000);

        let job_id = mp.post_job(
            &client_addr,
            &token_addr,
            &TokenType::USDC,
            &10_000i128,
            &String::from_str(&env, "Build a dApp"),
            &String::from_str(&env, "Full-stack Soroban dApp"),
            &String::from_str(&env, "Development"),
            &1000u32,
        );
        assert_eq!(job_id, 0);

        let app_id = mp.apply_to_job(
            &freelancer_addr,
            &job_id,
            &String::from_str(&env, "I can do this"),
            &10_000i128,
        );
        assert_eq!(app_id, 0);

        mp.hire_freelancer(&client_addr, &job_id, &app_id);
        let job = mp.get_job(&job_id);
        assert_eq!(job.status, JobStatus::InProgress);

        mp.complete_job(&client_addr, &job_id);
        let job = mp.get_job(&job_id);
        assert_eq!(job.status, JobStatus::Completed);

        // Freelancer received budget
        assert_eq!(TokenClient::new(&env, &token_addr).balance(&freelancer_addr), 10_000);
        // Admin received fee (250)
        assert_eq!(TokenClient::new(&env, &token_addr).balance(&admin), 250);
    }

    #[test]
    fn test_cancel_open_job_refunds() {
        let (env, admin, client_addr, _, token_addr) = setup();
        let contract_id = env.register(MarketplaceContract, ());
        let mp = MarketplaceContractClient::new(&env, &contract_id);
        mp.initialize(&admin, &250u32);
        StellarAssetClient::new(&env, &token_addr).mint(&client_addr, &10_250);
        let job_id = mp.post_job(
            &client_addr,
            &token_addr,
            &TokenType::XLM,
            &10_000i128,
            &String::from_str(&env, "t"),
            &String::from_str(&env, "d"),
            &String::from_str(&env, "c"),
            &500u32,
        );
        mp.cancel_job(&client_addr, &job_id);
        assert_eq!(TokenClient::new(&env, &token_addr).balance(&client_addr), 10_250);
    }

    #[test]
    fn test_dispute_and_resolve() {
        let (env, admin, client_addr, freelancer_addr, token_addr) = setup();
        let contract_id = env.register(MarketplaceContract, ());
        let mp = MarketplaceContractClient::new(&env, &contract_id);
        mp.initialize(&admin, &0u32);
        StellarAssetClient::new(&env, &token_addr).mint(&client_addr, &10_000);
        let job_id = mp.post_job(
            &client_addr,
            &token_addr,
            &TokenType::Custom,
            &10_000i128,
            &String::from_str(&env, "t"),
            &String::from_str(&env, "d"),
            &String::from_str(&env, "c"),
            &500u32,
        );
        let app_id = mp.apply_to_job(
            &freelancer_addr,
            &job_id,
            &String::from_str(&env, "p"),
            &10_000i128,
        );
        mp.hire_freelancer(&client_addr, &job_id, &app_id);
        mp.flag_dispute(&client_addr, &job_id);
        mp.resolve_dispute(&admin, &job_id, &4_000i128, &6_000i128);
        assert_eq!(TokenClient::new(&env, &token_addr).balance(&client_addr), 4_000);
        assert_eq!(TokenClient::new(&env, &token_addr).balance(&freelancer_addr), 6_000);
    }
}
