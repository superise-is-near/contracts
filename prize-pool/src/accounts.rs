use std::collections::{HashMap, HashSet};
use crate::prize::{FtPrize, NftPrize, PrizeToken};
use crate::prize_pool::{PoolId, PrizePoolDisplay};
use crate::utils::TokenAccountId;
use crate::utils::{ext_self, GAS_FOR_FT_TRANSFER, GAS_FOR_RESOLVE_TRANSFER};
use crate::{Contract, StorageKey};
use crate::*;
use near_contract_standards::fungible_token::core_impl::ext_fungible_token;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::serde_json::to_string;
use asset::*;

use near_sdk::collections::{TreeMap, UnorderedMap, UnorderedSet};
use near_sdk::json_types::{ValidAccountId, U128};
use near_sdk::{assert_one_yocto, env, near_bindgen, AccountId, Balance, Promise, PromiseResult, Timestamp, log};
use itertools::Itertools;
use std::panic::catch_unwind;
use near_sdk::env::log;
use crate::asset::{ContractId, NftId};

#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct Account {
    pub name: AccountId,
    pub assets: Assets,
    pub pools: HashSet<PoolId>
}

impl Account {
    pub fn new(account_id: &AccountId) -> Self {
        Account {
            name: account_id.to_string(),
            assets: Assets::default(),
            pools: HashSet::default()
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

#[near_bindgen]
impl Contract {
    pub fn exchange_callback_post_withdraw(
        &mut self,
        token_id: AccountId,
        sender_id: AccountId,
        amount: U128,
    ) {
        assert_eq!(
            env::promise_results_count(),
            1,
            "{}",
            "callback_withdraw_invalid"
        );
        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(_) => {}
            PromiseResult::Failed => {
                // This reverts the changes from withdraw function.
                // If account doesn't exit, deposits to the owner's account as lostfound.
                if let Some(mut account) = self.accounts.get(&sender_id) {
                    account.assets.deposit_ft(&token_id, &amount.into()) 
                }
            }
        }
    }

    pub fn internal_deposit(
        &mut self,
        sender_id: &AccountId,
        token_id: &AccountId,
        amount: Balance,
    ) {
        let mut account = self.accounts.get(&sender_id)
            .unwrap_or(Account::new(&sender_id));
        account.assets.deposit_ft(&token_id, &amount);
        self.accounts.insert(&sender_id,&account);
    }

    pub fn internal_deposit_nft(
        &mut self,
        owner_id: &AccountId,
        contract_id: &ContractId,
        token_id: &NftId,
    ) {
        let mut account = self.accounts.get(&owner_id)
            .unwrap_or(Account::new(&owner_id));
        account.assets.deposit_nft(&contract_id, &token_id);
        self.accounts.insert(&owner_id,&account);
    }

    /// Sends given amount to given user and if it fails, returns it back to user's balance.
    /// Tokens must already be subtracted from internal balance.
    pub(crate) fn internal_send_tokens(
        &self,
        sender_id: &AccountId,
        token_id: &AccountId,
        amount: Balance,
    ) -> Promise {
        ext_fungible_token::ft_transfer(
            sender_id.clone(),
            U128(amount),
            None,
            &token_id,
            1,
            GAS_FOR_FT_TRANSFER,
        )
        .then(ext_self::exchange_callback_post_withdraw(
            token_id.clone(),
            sender_id.clone(),
            U128(amount),
            &env::current_account_id(),
            0,
            GAS_FOR_RESOLVE_TRANSFER,
        ))
    }

    pub fn view_account_balance(&self, account_id: ValidAccountId)->Assets {
        return self.accounts.get(&account_id.as_ref())
        .expect(format!("no such account: {}",account_id))
        .assets;
    }


    #[payable]
    pub fn withdraw_ft(&mut self, token_id: ValidAccountId, amount: U128) -> Promise {
        // assert_one_yocto();
        // let token_id: AccountId = token_id.into();
        // let amount: u128 = amount.into();
        // assert!(amount > 0, "{}", "Illegal withdraw amount");
        // let sender_id = env::predecessor_account_id();
        // let mut account = self.accounts.get(&sender_id).expect("no such account");
        // account.assets.withdraw_ft(&token_id, &amount);
        // log!("withdraw_ft, token_id {}",token_id);
        self.internal_send_tokens(&sender_id, &token_id, amount)

        //1. 拿到account
        let mut account = self.accounts.get(&env::predecessor_account_id()).expect("no such user");
        //2. 调account的withdraw
        account.assets.withdraw_contract_amount(token_id.into(),&amount.0);

        //3. 把资产转给用户



    }

    pub fn view_user_pool(&self, account_id: ValidAccountId)->Vec<PrizePoolDisplay> {
        let account = self.accounts.get(account_id.as_ref()).unwrap_or(Account::new(account_id.as_ref()));
        return account.pools.iter()
            .filter_map(|pool_id|self.prize_pools.get(&pool_id))
            .map(|e|e.into()).collect_vec()
    }
}
