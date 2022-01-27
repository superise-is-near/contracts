use std::any::Any;
use std::collections::HashMap;
use std::convert::TryFrom;

use near_sdk::json_types::{ValidAccountId, U128};
use near_sdk::serde_json::{Value, from_value};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::AccountId;
use near_sdk_sim::{
    call, deploy, init_simulator, to_yocto, view, ContractAccount, ExecutionResult, UserAccount,
};

use ref_exchange::{ContractContract as Exchange, PoolInfo, ContractMetadata};
use test_token::ContractContract as TestToken;
use prize_pool::{ContractContract as PrizePoolContract, PrizePoolHeap};


near_sdk_sim::lazy_static_include::lazy_static_include_bytes! {
    PRIZPOOL_WASM_BYTES => "../out/main.wasm",
}

fn dai() -> AccountId {
    "dai".to_string()
}

fn eth() -> AccountId {
    "eth".to_string()
}

fn swap() -> AccountId {
    "swap".to_string()
}

fn prizepool() -> AccountId{"prize-pool".to_string()}
pub const PRIZE_POOL_ACCOUNT_ID: AccountId = "prize_pool".to_string();



pub fn prize_pool_contract()->ContractAccount<PrizePoolContract> {
    return deploy!(
        contract: PrizePoolContract,
        contract_id: prizepool(),
        bytes: &PRIZPOOL_WASM_BYTES,
        signer_account: root,
        init_method: new()
    );
}