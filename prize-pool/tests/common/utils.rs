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

near_sdk_sim::lazy_static_include::lazy_static_include_bytes! {
    TEST_TOKEN_WASM_BYTES => "../res/test_token.wasm",
    PREV_EXCHANGE_WASM_BYTES => "../res/ref_exchange_102.wasm",
    EXCHANGE_WASM_BYTES => "../res/ref_exchange_release.wasm",
    PRIZPOOL_WASM_BYTES => "../out/main.wasm",
}





