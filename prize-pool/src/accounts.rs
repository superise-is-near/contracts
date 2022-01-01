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

use near_sdk::collections::{TreeMap, UnorderedMap, UnorderedSet};
use near_sdk::json_types::{ValidAccountId, U128};
use near_sdk::{assert_one_yocto, env, near_bindgen, AccountId, Balance, Promise, PromiseResult, Timestamp, log};
use itertools::Itertools;
use std::panic::catch_unwind;
use near_sdk::env::log;
use crate::asset::{ContractId, NftId};

#[derive(BorshSerialize, BorshDeserialize)]
pub struct Account {
    pub name: AccountId,

    pub fts: UnorderedMap<TokenAccountId, Balance>,
    pub nfts: UnorderedMap<AccountId, HashSet<NftId>>,
    pub pools: UnorderedSet<u64>,
}
impl fmt::Display for Account {
    // fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    //     match self {
    //         RunningState::Running => write!(f, "Running"),
    //         RunningState::Paused => write!(f, "Paused"),
    //     }
    // }
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        let ftsstring = self.fts.iter().map(|(k, v)| format!("{}: {}", k, v)).join(",");
        let poolsstring = self.pools.iter().map(|k| format!("{}", k)).join(",");
        write!(f, "{{ name: {{ {} }}, fts: {{ {} }}, pools: {{ {} }} }}",self.name.to_string(),ftsstring,poolsstring)
    }
}

impl Account {
    pub fn new(account_id: &AccountId) -> Self {
        Account {
            name: account_id.to_string(),
            fts: UnorderedMap::new(StorageKey::AccountFts {
                account_id: account_id.clone(),
            }),
            nfts: UnorderedMap::new(StorageKey::AccountNfts {account_id: account_id.clone()}),
            pools: UnorderedSet::new(StorageKey::AccountPools {
                account_id: account_id.clone(),
            }),
        }
    }

    /// Deposit amount to the balance of given token.
    pub(crate) fn deposit_ft(&mut self, token: &AccountId, amount: &Balance) {
        let balance = self.fts.get(&token).unwrap_or(0);
        self.fts.insert(&token, &(balance + *amount));
        println!("when deposit {}",self)
    }

    pub(crate) fn deposit_nft(&mut self, contract_id: &ContractId, nft_id: &NftId) {
        let mut nft_set = self.nfts.get(&contract_id).unwrap_or(HashSet::new());
        nft_set.insert(nft_id.clone());
        self.nfts.insert(&contract_id,&nft_set);
    }

    pub(crate) fn withdraw_ft(&mut self, token: &AccountId, amount: &Balance) {
        let balance = self.fts.get(token).expect("unregistered token");
        assert!(balance >= *amount, "token balance not enough");
        self.fts.insert(&token, &(balance - (*amount)));
    }

    pub(crate) fn withdraw_nft(&mut self, token: &AccountId, amount: &Balance) {
        let balance = self.fts.get(token).expect("unregistered token");
        assert!(balance >= *amount, "token balance not enough");
        self.fts.insert(&token, &(balance - (*amount)));
    }
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
                    account.deposit_ft(&token_id, &amount.into())
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
            account.deposit_ft(&token_id, &amount);
        self.accounts.insert(&sender_id,&account);
    }

    pub fn internal_deposit_nft(
        &mut self,
        owner_id: &AccountId,
        contract_id: &ContractId,
        token_id: &NftId,
    ) {
        let mut account = self.accounts.get(&owner_id)
            .unwrap_or(Account::new(&sender_id));
        account.deposit_nft(&contract_id, &token_id);
        self.accounts.insert(&sender_id,&account);
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

    pub fn view_account_balance(&self, account_id: ValidAccountId)->HashMap<AccountId,U128> {
        log!("view_account_balance, {}",account_id);
        let map;
        if self.accounts.contains_key(&account_id.as_ref()) {
            map = self.accounts.get(&account_id.as_ref())
                .unwrap()
                .fts
                .iter()
                .map(|(k,v)|(k,U128(v)))
                .collect();
        } else {
            map = HashMap::new();
        }
        log!("map: {}",near_sdk::serde_json::to_string(&map).unwrap());
        return map;
    }


    #[payable]
    pub fn withdraw_ft(&mut self, token_id: ValidAccountId, amount: U128) -> Promise {
        assert_one_yocto();
        let token_id: AccountId = token_id.into();
        let amount: u128 = amount.into();
        assert!(amount > 0, "{}", "Illegal withdraw amount");
        let sender_id = env::predecessor_account_id();
        let mut account = self.accounts.get(&sender_id).expect("no such account");
        account.withdraw_ft(&token_id, &amount);
        log!("withdraw_ft, token_id {}",token_id);
        self.internal_send_tokens(&sender_id, &token_id, amount)
    }

    pub fn view_user_pool(&self, account_id: ValidAccountId)->Vec<PrizePoolDisplay> {
        let account = self.accounts.get(account_id.as_ref()).unwrap_or(Account::new(account_id.as_ref()));
        return account.pools.iter()
            .filter_map(|pool_id|self.prize_pools.get(&pool_id))
            .map(|e|e.into()).collect_vec()
    }
}
