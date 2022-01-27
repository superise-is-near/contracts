use near_sdk::AccountId;
use near_sdk_sim::{ContractAccount, deploy, init_simulator, to_yocto};
use prize_pool::{Contract, ContractContract as PrizePoolContract, PrizePoolHeap};
use crate::common::utils::*;
pub mod common;

fn setup_prize_pool()->ContractAccount<PrizePoolContract> {
    let root = init_simulator(None);
    let owner = root.create_user("owner".to_string(), to_yocto("100"));
    let prizePool = deploy!(
        contract: PrizePoolContract,
        contract_id: PRIZE_POOL_ACCOUNT_ID,
        bytes: &PRIZPOOL_WASM_BYTES,
        signer_account: root,
        init_method: new()
    );;
}

#[test]
fn test_withdraw() {

}