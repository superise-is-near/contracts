use std::collections::{HashMap, HashSet};
use crate::prize::{FtPrize, NftPrize, PrizeToken};
use crate::prize_pool::PoolId;
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

#[derive(BorshSerialize, BorshDeserialize,Serialize,Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Record {
    id: PoolId,
    end_time: Timestamp,
    ft_prize: FtPrize,
}

#[derive(BorshSerialize, BorshDeserialize)]
pub struct Account {
    pub name: AccountId,
    pub fts: UnorderedMap<TokenAccountId, Balance>,
    // pub nfts: UnorderedSet<NftPrize>,
    pub pools: UnorderedSet<u64>,
    pub history: UnorderedMap<u64, Record>,
    pub created_pool: UnorderedSet<u64>
}
impl fmt::Display for Account {
    // fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    //     match self {
    //         RunningState::Running => write!(f, "Running"),
    //         RunningState::Paused => write!(f, "Paused"),
    //     }
    // }
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "name: {}, ", self.name.to_string());
        let ftsstring = self.fts.iter().map(|(k, v)| format!("{}: {}", k, v)).join(",");
        for (k,v) in self.fts.iter() {

        }
        write!(f, "fts: {},", ftsstring);
        let poolsstring = self.pools.iter().map(|k| format!("{}", k)).join(",");
        write!(f, "pools: {},", poolsstring);
        let hisotry = self.history.iter().map(|(k, v)| format!("{}: {}", k, to_string(&v).unwrap())).join(",");
        write!(f, "history: {}", hisotry)
    }
}

impl Account {
    pub fn new(account_id: &AccountId) -> Self {
        Account {
            name: account_id.to_string(),
            fts: UnorderedMap::new(StorageKey::AccountTokens {
                account_id: account_id.clone(),
            }),
            pools: UnorderedSet::new(StorageKey::AccountPools {
                account_id: account_id.clone(),
            }),
            history: UnorderedMap::new(StorageKey::AccountHistory {
                account_id: account_id.clone(),
            }),
            created_pool: UnorderedSet::new(StorageKey::)
        }
    }

    /// Deposit amount to the balance of given token.
    pub(crate) fn deposit(&mut self, token: &AccountId, amount: &Balance) {
        let balance = self.fts.get(&token).unwrap_or(0);
        self.fts.insert(&token, &(balance + *amount));
        println!("when deposit {}",self)
    }

    pub(crate) fn withdraw_ft(&mut self, token: &AccountId, amount: &Balance) {
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
                    account.deposit(&token_id, &amount.into())
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
        self.accounts.get(sender_id)
            .get_or_insert(&Account::new(&sender_id))
            .deposit(&token_id,&amount);
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

    pub fn view_account_balance(&self, account_id: ValidAccountId)->HashMap<AccountId,Balance> {
        log!("view_account_balance, {}",account_id);
        let map = self.accounts.get(&account_id.as_ref()).unwrap_or(&Account::new(&account_id.as_ref())).fts.iter().collect();
        // let account = self.accounts.get(&account_id.as_ref()).unwrap_or(Account::new(&account_id.as_ref()));
        // let map = account.fts.iter().collect();
        log!("map: {}",near_sdk::serde_json::to_string(&map).unwrap());
        return map
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
}
