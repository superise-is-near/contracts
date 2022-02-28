use near_sdk::AccountId;
use near_sdk_sim::{
    call, deploy, init_simulator, to_yocto, view, ContractAccount, ExecutionResult, UserAccount,
};
use prize_pool::{ContractContract as PrizePoolContract};
use test_token::{ContractContract as TestTokenContract};
use near_sdk::json_types::{ValidAccountId, U128};
use std::convert::TryFrom;
use crate::common::constant::PRIZE_POOL_ACCOUNT_ID;
use crate::common::utils::to_va;



near_sdk_sim::lazy_static_include::lazy_static_include_bytes! {
    PRIZPOOL_WASM_BYTES => "./out/prize-pool.wasm",
    TEST_TOKEN_WASM_BYTES => "../res/test_token.wasm",
}


enum Contracts {
    SUPERISE,
}

type ContractName = String;
trait GetContractName {
    fn get_contract_name(&self)->AccountId;
}

pub fn  deploy_prize_pool_contract(signer_account: &UserAccount, admin: Option<AccountId>)->ContractAccount<PrizePoolContract> {
    let contract = deploy!(
        contract: PrizePoolContract,
        contract_id: PRIZE_POOL_ACCOUNT_ID(),
        bytes: &PRIZPOOL_WASM_BYTES,
        signer_account: signer_account,
        init_method: new(ValidAccountId::try_from(admin.unwrap_or("123".to_string())).unwrap())
    );
    contract
}

pub fn deploy_test_token_contract(
    signer_account: &UserAccount,
    token_id: AccountId,
    accounts_to_register: Vec<AccountId>
)->ContractAccount<TestTokenContract> {
    let t = deploy!(
        contract: TestTokenContract,
        contract_id: token_id,
        bytes: &TEST_TOKEN_WASM_BYTES,
        signer_account: signer_account
    );
    call!(signer_account, t.new()).assert_success();
    call!(
        signer_account,
        t.mint(to_va(signer_account.account_id.clone()), to_yocto("1000").into())
    )
        .assert_success();
    for account_id in accounts_to_register {
        call!(
            signer_account,
            t.storage_deposit(Some(to_va(account_id)), None),
            deposit = to_yocto("1")
        )
            .assert_success();
    }
    t
}