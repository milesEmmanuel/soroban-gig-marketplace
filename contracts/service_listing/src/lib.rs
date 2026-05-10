#![no_std]

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, token, Address, Env, String, Symbol,
};

// ── Storage keys ─────────────────────────────────────────────────────────────

const ADMIN: Symbol = symbol_short!("ADMIN");
const FEE_BPS: Symbol = symbol_short!("FEE_BPS");
const SVC_CNT: Symbol = symbol_short!("SVC_CNT");
const ORD_CNT: Symbol = symbol_short!("ORD_CNT");

fn svc_key(id: u64) -> (Symbol, u64) {
    (symbol_short!("SVC"), id)
}
fn ord_key(id: u64) -> (Symbol, u64) {
    (symbol_short!("ORD"), id)
}

// ── Types ─────────────────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, PartialEq)]
pub enum ServiceStatus {
    Active,
    Paused,
    Deleted,
}

#[contracttype]
#[derive(Clone, PartialEq)]
pub enum OrderStatus {
    Pending,
    InProgress,
    Delivered,
    Completed,
    Cancelled,
    Disputed,
}

#[contracttype]
#[derive(Clone)]
pub struct ServiceListing {
    pub freelancer: Address,
    pub token: Address,
    pub price: i128,
    pub title: String,
    pub description: String,
    pub category: String,
    pub delivery_ledgers: u32,
    pub orders_completed: u64,
    pub status: ServiceStatus,
}

#[contracttype]
#[derive(Clone)]
pub struct Order {
    pub service_id: u64,
    pub client: Address,
    pub freelancer: Address,
    pub token: Address,
    pub amount: i128,
    pub fee: i128,
    pub requirements: String,
    pub deadline_ledger: u32,
    pub status: OrderStatus,
}

// ── Contract ──────────────────────────────────────────────────────────────────

#[contract]
pub struct ServiceListingContract;

#[contractimpl]
impl ServiceListingContract {
    pub fn initialize(env: Env, admin: Address, fee_bps: u32) {
        if env.storage().instance().has(&ADMIN) {
            panic!("already initialized");
        }
        admin.require_auth();
        env.storage().instance().set(&ADMIN, &admin);
        env.storage().instance().set(&FEE_BPS, &fee_bps);
        env.storage().instance().set(&SVC_CNT, &0u64);
        env.storage().instance().set(&ORD_CNT, &0u64);
    }

    /// Freelancer creates a service listing.
    pub fn create_service(
        env: Env,
        freelancer: Address,
        token: Address,
        price: i128,
        title: String,
        description: String,
        category: String,
        delivery_ledgers: u32,
    ) -> u64 {
        freelancer.require_auth();
        let id: u64 = env.storage().instance().get(&SVC_CNT).unwrap();
        let svc = ServiceListing {
            freelancer: freelancer.clone(),
            token: token.clone(),
            price,
            title,
            description,
            category,
            delivery_ledgers,
            orders_completed: 0,
            status: ServiceStatus::Active,
        };
        env.storage().persistent().set(&svc_key(id), &svc);
        env.storage().instance().set(&SVC_CNT, &(id + 1));

        env.events().publish(
            (symbol_short!("svc_new"), freelancer),
            (id, token, price),
        );
        id
    }

    /// Client purchases a service; payment locked in contract.
    pub fn purchase_service(
        env: Env,
        client: Address,
        service_id: u64,
        requirements: String,
    ) -> u64 {
        client.require_auth();
        let svc: ServiceListing =
            env.storage().persistent().get(&svc_key(service_id)).expect("service not found");
        if svc.status != ServiceStatus::Active {
            panic!("service not active");
        }
        let fee_bps: u32 = env.storage().instance().get(&FEE_BPS).unwrap();
        let fee = svc.price * fee_bps as i128 / 10_000;
        let total = svc.price + fee;

        token::Client::new(&env, &svc.token).transfer(
            &client,
            &env.current_contract_address(),
            &total,
        );

        let deadline = env.ledger().sequence() + svc.delivery_ledgers;
        let ord_id: u64 = env.storage().instance().get(&ORD_CNT).unwrap();
        let order = Order {
            service_id,
            client: client.clone(),
            freelancer: svc.freelancer.clone(),
            token: svc.token.clone(),
            amount: svc.price,
            fee,
            requirements,
            deadline_ledger: deadline,
            status: OrderStatus::InProgress,
        };
        env.storage().persistent().set(&ord_key(ord_id), &order);
        env.storage().instance().set(&ORD_CNT, &(ord_id + 1));

        env.events().publish(
            (symbol_short!("ord_new"), client),
            (ord_id, service_id, svc.price),
        );
        ord_id
    }

    /// Freelancer marks order as delivered.
    pub fn deliver_order(env: Env, freelancer: Address, order_id: u64) {
        freelancer.require_auth();
        let mut order: Order =
            env.storage().persistent().get(&ord_key(order_id)).expect("order not found");
        if order.freelancer != freelancer {
            panic!("not freelancer");
        }
        if order.status != OrderStatus::InProgress {
            panic!("not in progress");
        }
        order.status = OrderStatus::Delivered;
        env.storage().persistent().set(&ord_key(order_id), &order);

        env.events().publish(
            (symbol_short!("ord_dlvr"), freelancer),
            order_id,
        );
    }

    /// Client accepts delivery; payment released to freelancer + fee to admin.
    pub fn accept_delivery(env: Env, client: Address, order_id: u64) {
        client.require_auth();
        let mut order: Order =
            env.storage().persistent().get(&ord_key(order_id)).expect("order not found");
        if order.client != client {
            panic!("not client");
        }
        if order.status != OrderStatus::Delivered {
            panic!("not delivered");
        }
        let admin: Address = env.storage().instance().get(&ADMIN).unwrap();
        let tok = token::Client::new(&env, &order.token);
        tok.transfer(&env.current_contract_address(), &order.freelancer, &order.amount);
        if order.fee > 0 {
            tok.transfer(&env.current_contract_address(), &admin, &order.fee);
        }
        order.status = OrderStatus::Completed;
        env.storage().persistent().set(&ord_key(order_id), &order);

        // Increment orders_completed on the service
        let mut svc: ServiceListing =
            env.storage().persistent().get(&svc_key(order.service_id)).expect("service not found");
        svc.orders_completed += 1;
        env.storage().persistent().set(&svc_key(order.service_id), &svc);

        env.events().publish(
            (symbol_short!("ord_done"), client),
            (order_id, order.freelancer, order.amount),
        );
    }

    /// Client or freelancer cancels; full refund to client.
    pub fn cancel_order(env: Env, caller: Address, order_id: u64) {
        caller.require_auth();
        let mut order: Order =
            env.storage().persistent().get(&ord_key(order_id)).expect("order not found");
        if caller != order.client && caller != order.freelancer {
            panic!("not a party");
        }
        if order.status != OrderStatus::InProgress && order.status != OrderStatus::Pending {
            panic!("cannot cancel");
        }
        let refund = order.amount + order.fee;
        token::Client::new(&env, &order.token).transfer(
            &env.current_contract_address(),
            &order.client,
            &refund,
        );
        order.status = OrderStatus::Cancelled;
        env.storage().persistent().set(&ord_key(order_id), &order);

        env.events().publish(
            (symbol_short!("ord_cncl"), caller),
            (order_id, refund),
        );
    }

    /// Freelancer toggles service between Active and Paused.
    pub fn toggle_service(env: Env, freelancer: Address, service_id: u64) {
        freelancer.require_auth();
        let mut svc: ServiceListing =
            env.storage().persistent().get(&svc_key(service_id)).expect("service not found");
        if svc.freelancer != freelancer {
            panic!("not owner");
        }
        svc.status = match svc.status {
            ServiceStatus::Active => ServiceStatus::Paused,
            ServiceStatus::Paused => ServiceStatus::Active,
            ServiceStatus::Deleted => panic!("service deleted"),
        };
        env.storage().persistent().set(&svc_key(service_id), &svc);

        env.events().publish(
            (symbol_short!("svc_tgl"), freelancer),
            service_id,
        );
    }

    /// Freelancer updates service price.
    pub fn update_price(env: Env, freelancer: Address, service_id: u64, new_price: i128) {
        freelancer.require_auth();
        let mut svc: ServiceListing =
            env.storage().persistent().get(&svc_key(service_id)).expect("service not found");
        if svc.freelancer != freelancer {
            panic!("not owner");
        }
        svc.price = new_price;
        env.storage().persistent().set(&svc_key(service_id), &svc);

        env.events().publish(
            (symbol_short!("svc_upd"), freelancer),
            (service_id, new_price),
        );
    }

    // ── Views ─────────────────────────────────────────────────────────────────

    pub fn get_service(env: Env, service_id: u64) -> ServiceListing {
        env.storage().persistent().get(&svc_key(service_id)).expect("service not found")
    }

    pub fn get_order(env: Env, order_id: u64) -> Order {
        env.storage().persistent().get(&ord_key(order_id)).expect("order not found")
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
        let freelancer = Address::generate(&env);
        let client = Address::generate(&env);
        let token = env.register_stellar_asset_contract_v2(admin.clone()).address();
        (env, admin, freelancer, client, token)
    }

    #[test]
    fn test_create_purchase_deliver_accept() {
        let (env, admin, freelancer, client, token) = setup();
        let cid = env.register(ServiceListingContract, ());
        let sl = ServiceListingContractClient::new(&env, &cid);

        sl.initialize(&admin, &250u32);
        StellarAssetClient::new(&env, &token).mint(&client, &10_250);

        let svc_id = sl.create_service(
            &freelancer,
            &token,
            &10_000i128,
            &String::from_str(&env, "Logo Design"),
            &String::from_str(&env, "Professional logo"),
            &String::from_str(&env, "Design"),
            &100u32,
        );

        let ord_id = sl.purchase_service(
            &client,
            &svc_id,
            &String::from_str(&env, "Make it blue"),
        );

        sl.deliver_order(&freelancer, &ord_id);
        sl.accept_delivery(&client, &ord_id);

        assert_eq!(TokenClient::new(&env, &token).balance(&freelancer), 10_000);
        assert_eq!(TokenClient::new(&env, &token).balance(&admin), 250);
        assert_eq!(sl.get_service(&svc_id).orders_completed, 1);
    }

    #[test]
    fn test_cancel_order_refunds() {
        let (env, admin, freelancer, client, token) = setup();
        let cid = env.register(ServiceListingContract, ());
        let sl = ServiceListingContractClient::new(&env, &cid);
        sl.initialize(&admin, &0u32);
        StellarAssetClient::new(&env, &token).mint(&client, &5_000);
        let svc_id = sl.create_service(
            &freelancer,
            &token,
            &5_000i128,
            &String::from_str(&env, "t"),
            &String::from_str(&env, "d"),
            &String::from_str(&env, "c"),
            &50u32,
        );
        let ord_id = sl.purchase_service(&client, &svc_id, &String::from_str(&env, "req"));
        sl.cancel_order(&client, &ord_id);
        assert_eq!(TokenClient::new(&env, &token).balance(&client), 5_000);
    }

    #[test]
    fn test_toggle_and_update_price() {
        let (env, admin, freelancer, _client, token) = setup();
        let cid = env.register(ServiceListingContract, ());
        let sl = ServiceListingContractClient::new(&env, &cid);
        sl.initialize(&admin, &0u32);
        let svc_id = sl.create_service(
            &freelancer,
            &token,
            &1_000i128,
            &String::from_str(&env, "t"),
            &String::from_str(&env, "d"),
            &String::from_str(&env, "c"),
            &50u32,
        );
        sl.toggle_service(&freelancer, &svc_id);
        assert_eq!(sl.get_service(&svc_id).status, ServiceStatus::Paused);
        sl.update_price(&freelancer, &svc_id, &2_000i128);
        assert_eq!(sl.get_service(&svc_id).price, 2_000);
    }
}
