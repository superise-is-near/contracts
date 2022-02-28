use near_sdk::AccountId;
use near_sdk::json_types::U128;
use near_sdk_sim::{call, ContractAccount, deploy, init_simulator, to_yocto, view};
use prize_pool::{Contract, ContractContract as PrizePoolContract, PrizePoolHeap };
use prize_pool::asset::Ft;
use prize_pool::asset::AssetsDTO;
use crate::common::contracts::{deploy_prize_pool_contract, deploy_test_token_contract};
use crate::common::utils::*;
pub mod common;


#[test]
fn test_withdraw() {
    let root = init_simulator(None);
    let user = root.create_user("user".to_string(), to_yocto("100"));
    let prize_pool_contract = deploy_prize_pool_contract(&root,None);
    let wrap_near = deploy_test_token_contract(
        &root,
        "wrapnear".to_string(),
        vec![user.account_id.clone(),prize_pool_contract.account_id().clone()] );
    println!("one yocto: {}",to_yocto("1"));
    call!(
        root,
        wrap_near.ft_transfer(to_va(user.account_id.clone()), to_yocto("10").into(), None ),
        deposit = 1
    ).assert_success();
    call!(
        user,
        wrap_near.ft_transfer_call(to_va(prize_pool_contract.account_id()), to_yocto("10").into(), None, "".to_string()),
        deposit = 1
    ).assert_success();
    let assets = view!(prize_pool_contract.view_account_assets(to_va(user.account_id.clone()))).unwrap_json::<AssetsDTO>();
    assert_eq!(assets.ft_assets[0], Ft{ contract_id: "wrapnear".to_string(), balance: U128::from(to_yocto("10"))}, "balance should equal");
}