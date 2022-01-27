use std::collections::{HashMap, HashSet};
use crate::prize::{FtPrize, NftPrize};
use crate::prize_pool::{PoolId};
use crate::utils::{ONE_YOCTO, TokenAccountId};
use crate::utils::{GAS_FOR_FT_TRANSFER, GAS_FOR_RESOLVE_TRANSFER};
use crate::{Contract, StorageKey};
use crate::*;
use near_contract_standards::fungible_token::core_impl::ext_fungible_token;
use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::serde_json::{json, to_string};
use asset::*;

use near_sdk::collections::{TreeMap, UnorderedMap, UnorderedSet};
use near_sdk::json_types::{ValidAccountId, U128};
use near_sdk::{ext_contract, assert_one_yocto, env, near_bindgen, AccountId, Balance, Promise, PromiseResult, Timestamp, log};
use itertools::Itertools;
use std::panic::catch_unwind;
use near_sdk::env::log;
use crate::asset::{ContractId, NftId};

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct Account {
    pub name: AccountId,
    pub assets: Assets,
    pub pools: HashSet<PoolId>,
}

impl Account {
    pub fn new(account_id: &AccountId) -> Self {
        Account {
            name: account_id.to_string(),
            assets: Assets::default(),
            pools: HashSet::default(),
        }
    }

    // /// Deposit amount to the balance of given token.
    // pub(crate) fn deposit_ft(&mut self, token: &AccountId, amount: &Balance) {
    //     self.assets.deposit_ft(token,amount);
    // }

    // pub(crate) fn deposit_nft(&mut self, contract_id: &ContractId, nft_id: &NftId) {
    //     self.assets.deposit_nft(contract_id,nft_id);
    // }

    // pub(crate) fn withdraw_ft(&mut self, token: &AccountId, amount: &Balance) {
    //     let balance = self.fts.get(token).expect("unregistered token");
    //     assert!(balance >= *amount, "token balance not enough");
    //     self.fts.insert(&token, &(balance - (*amount)));
    // }

    // pub(crate) fn withdraw_nft(&mut self, token: &AccountId, amount: &Balance) {
    //     let balance = self.fts.get(token).expect("unregistered token");
    //     assert!(balance >= *amount, "token balance not enough");
    //     self.fts.insert(&token, &(balance - (*amount)));
    // }
}

#[ext_contract(ext_self)]
pub trait ContractMethod {
    fn withdraw_ft_callback(
        &mut self,
        sender_id: AccountId,
        token_id: AccountId,
        amount: U128,
    );
    fn withdraw_nft_callback(
        &mut self,
        sender_id: AccountId,
        contract_id: AccountId,
        nft_id: AccountId,
    );
}

#[near_bindgen]
impl Contract {
    #[private]
    pub fn withdraw_nft_callback(
        &mut self,
        sender_id: AccountId,
        contract_id: AccountId,
        nft_id: AccountId,
    ) {
        assert_eq!(
            env::promise_results_count(),
            1,
            "withdraw_nft_callback_invalid"
        );
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(_) => {}
            PromiseResult::Failed => {
                if let Some(mut account) = self.accounts.get(&sender_id) {
                    account.assets.deposit_contract_nft_id(&contract_id, &nft_id);
                    self.accounts.insert(&sender_id, &account);
                }
            }
        }
    }

    #[private]
    pub fn withdraw_ft_callback(
        &mut self,
        token_id: AccountId,
        sender_id: AccountId,
        amount: U128,
    ) {
        assert_eq!(
            env::promise_results_count(),
            1,
            "withdraw_ft_callback_invalid"
        );
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(_) => {}
            PromiseResult::Failed => {
                // This reverts the changes from withdraw function.
                // If account doesn't exit, deposits to the owner's account as lostfound.
                if let Some(mut account) = self.accounts.get(&sender_id) {
                    account.assets.deposit_contract_amount(&token_id, &amount.into());
                    self.accounts.insert(&sender_id, &account);
                }
            }
        }
    }

    // transfer ft to other account
    #[private]
    pub(crate) fn external_send_ft(
        &self,
        received_id: &AccountId,
        token_id: &AccountId,
        amount: &U128,
    ) -> Promise {
        ext_fungible_token::ft_transfer(
            received_id.clone(),
            amount.clone(),
            None,
            &token_id,
            1,
            GAS_FOR_FT_TRANSFER,
        )
            .then(ext_self::withdraw_ft_callback(
                received_id.clone(),
                token_id.clone(),
                amount.clone(),
                &env::current_account_id(),
                0,
                GAS_FOR_RESOLVE_TRANSFER,
            ))
    }

    // transfer ft to other account
    #[private]
    pub(crate) fn external_send_nft(
        &self,
        received_id: &AccountId,
        contract_id: &AccountId,
        nft_id: &NftId,
    ) -> Promise {
        return Promise::new(contract_id.clone())
            .function_call(
                b"nft_transfer".to_vec(),
                json!({
                    "receiver_id": received_id.clone(),
                    "token_id": nft_id.clone()
                }).to_string().into_bytes(),
                ONE_YOCTO,
                GAS_FOR_FT_TRANSFER)
            .then(ext_self::withdraw_nft_callback(
                received_id.clone(),
                contract_id.clone(),
                nft_id.clone(),
                &env::current_account_id(),
                0,
                GAS_FOR_RESOLVE_TRANSFER,
            ));
        // receiver_id: string,
        // token_id: string,
        // approval_id: number|null,
        // memo: string|null,
    }

    pub fn view_account_balance(&self, account_id: ValidAccountId) -> HashMap<ContractId, U128> {
        return self.accounts.get(&account_id.as_ref())
            .expect("no such account")
            .assets
            .fts
            .iter()
            .map(|(contract_id, balance)| (contract_id.clone(), U128::from(balance.clone())))
            .collect();
        // .map(|(contract_id,balance)|{})
    }
    pub fn view_account_assets(&self, account_id: ValidAccountId) -> AssetsDTO {
        return self.accounts.get(&account_id.as_ref())
            .expect(&format!("no such account: {}", account_id))
            .assets.into();
    }

    pub fn internal_deposit_ft(&mut self, account_id: &ValidAccountId, contract_id: &ContractId, amount: &U128) {
        let mut account = self.accounts.get(account_id.as_ref()).unwrap_or(Account::new(account_id.as_ref()));
        account.assets.deposit_contract_amount(&contract_id, &amount.0);
        self.accounts.insert(&account_id.as_ref(), &account);
    }

    pub fn internal_deposit_nft(&mut self, account_id: &AccountId, contract_id: &ContractId, nft_id: &NftId) {
        let mut account = self.accounts.get(&account_id).unwrap_or(Account::new(&account_id));
        account.assets.deposit_contract_nft_id(&contract_id, &nft_id);
        self.accounts.insert(&account_id, &account);
    }


    // //todo 实现withdraw
    #[payable]
    pub fn withdraw_ft(&mut self, token_id: ValidAccountId, amount: U128) -> Promise {
        assert_one_yocto();

        //1. 拿到account
        let mut account = self.accounts.get(&env::predecessor_account_id()).expect("no such user");
        //2. 内部withdraw
        account.assets.withdraw_contract_amount(token_id.as_ref(), &amount.0);
        self.accounts.insert(&account.name, &account);
        //3. 外部合约transfer
        self.external_send_ft(&env::predecessor_account_id(), token_id.as_ref(), &amount)
    }

    // //todo 实现withdraw
    #[payable]
    pub fn withdraw_nft(&mut self, contract_id: ValidAccountId, nft_id: NftId) -> Promise {
        assert_one_yocto();

        //1. 拿到account
        let mut account = self.accounts.get(&env::predecessor_account_id()).expect("no such user");
        //2. 内部withdraw
        account.assets.withdraw_contract_nft_id(contract_id.as_ref(), &nft_id);
        self.accounts.insert(&account.name, &account);
        //3. 调外部合约transfer nft
        self.external_send_nft(&env::predecessor_account_id(), contract_id.as_ref(), &nft_id)
    }
}
