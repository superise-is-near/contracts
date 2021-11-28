mod prize;
mod prize_pool;
mod accounts;
mod utils;

use std::collections::HashMap;
use near_contract_standards::non_fungible_token::metadata::{
    NFTContractMetadata, NonFungibleTokenMetadataProvider, TokenMetadata, NFT_METADATA_SPEC,
};
use near_contract_standards::non_fungible_token::{Token, TokenId};
use near_contract_standards::non_fungible_token::NonFungibleToken;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupMap, Vector, UnorderedMap, UnorderedSet, TreeMap};
use near_sdk::json_types::{U128, U64, ValidAccountId};
use near_sdk::{env, near_bindgen, AccountId, BorshStorageKey, PanicOnDefault, Promise, PromiseOrValue, log};
use crate::prize::{Prize};
use std::convert::{TryFrom,TryInto};
use std::fmt;
use itertools::Itertools;
use crate::prize_pool::PrizePool;
use near_sdk::borsh::maybestd::sync::atomic::{Ordering, AtomicUsize};
use std::sync::atomic::AtomicU64;
use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use crate::accounts::Account;
use near_sdk::serde::{Deserialize, Serialize};

near_sdk::setup_alloc!();

pub type CreateId = AccountId;
pub type NonFungibleTokenId = String;
pub type FungibleTokenId = AccountId;

pub const WRAP_NEAR: &str = if env!("NEAR_ENV").eq("testnet") {"wrap.testnet"} else { "wrap.near" };

#[derive(BorshStorageKey, BorshSerialize)]
pub(crate) enum StorageKey {
    Accounts,
    PrizePools,
    OwnPool,
    OwnPools {account_id: AccountId},
    AccountTokens {account_id: AccountId},
    AccountPools {account_id: AccountId},
    AccountHistory {account_id: AccountId},
    AccountCreatedPools {account_id: AccountId},
}
pub fn next_id()->u64 {
    static ID: AtomicU64= AtomicU64::new(0);
    ID.fetch_add(1,Ordering::SeqCst);
    ID.load(Ordering::Relaxed)
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Eq, PartialEq, Clone)]
#[serde(crate = "near_sdk::serde")]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
pub enum RunningState {
    Running, Paused
}

impl fmt::Display for RunningState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RunningState::Running => write!(f, "Running"),
            RunningState::Paused => write!(f, "Paused"),
        }
    }
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    pub accounts: LookupMap<AccountId, Account>,
    pub prize_pools: UnorderedMap<u64,PrizePool>,
    // who own pools
    // pub own_pools: LookupMap<AccountId, UnorderedSet<u64>>
}

#[near_bindgen]
impl Contract {

    #[init]
    pub fn new() -> Self {
        Self{
            accounts: LookupMap::new(StorageKey::Accounts),
            prize_pools: UnorderedMap::new(StorageKey::PrizePools),
        }
    }

    pub fn clear(&mut self) {
        self.accounts.clear();
    }

    pub fn get_id(&self)-> U64 {
        return next_id().into();
    }
}

#[near_bindgen]
impl FungibleTokenReceiver for Contract {
    #[allow(unreachable_code)]
    fn ft_on_transfer(
        &mut self,
        sender_id: ValidAccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        log!("ft on transfer");
        let token_in = env::predecessor_account_id();
        self.internal_deposit(sender_id.as_ref(), &token_in, amount.into());
        PromiseOrValue::Value(U128(0))
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests {
    use crate::prize::{PrizeToken};
    use super::*;
    use near_sdk::MockedBlockchain;
    use near_sdk::{testing_env, VMContext};
    use near_sdk::serde::Serialize;
    use near_sdk::serde_json::Result;
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use accounts::Account;
    // use crate::prize::PrizeToken;

    fn get_context(input: Vec<u8>, is_view: bool) -> VMContext {
        VMContext {
            current_account_id: "alice_near".to_string(),
            signer_account_id: "bob_near".to_string(),
            signer_account_pk: vec![0, 1, 2],
            predecessor_account_id: "carol_near".to_string(),
            input,
            block_index: 0,
            block_timestamp: 0,
            account_balance: 0,
            account_locked_balance: 0,
            storage_usage: 0,
            attached_deposit: 0,
            prepaid_gas: 10u64.pow(18),
            random_seed: vec![0, 1, 2],
            is_view,
            output_data_receivers: vec![],
            epoch_height: 0,
        }
    }

    fn setup_contract() -> (VMContextBuilder, Contract) {
        let mut context = VMContextBuilder::new();
        testing_env!(context.predecessor_account_id(accounts(1)).build());
        let contract = Contract::new();
        (context, contract)
    }

    // const FT_STR: &str = r#"{"FT":{"contract":"someone_ft","sum":0}}"#;
    // const NFT_STR: &str = r#"{"NFT":{"contract":"someone_nft","id":"some nft"}}"#;
    // #[test]
    // fn insert_prize() {
    //     let context = get_context(vec![], false);
    //     testing_env!(context);
    //     let mut prize_pool  = PrizePool::default();
    //     // let ft_prize = PrizeToken::FT{ id: "someone_ft".to_string(), sum: 0 };
    //     let ft_prize: PrizeToken = near_sdk::serde_json::from_str(FT_STR).unwrap();
    //     prize_pool.add_ft_prize(ft_prize);
    //     println!("test insert_prize");
    //     println!("{}",near_sdk::serde_json::to_string(&prize_pool).unwrap());

    //     let nft_prize = PrizeToken::NFT{ contract: "someone_nft".to_string(),id: "xx".to_string()};
    //     prize_pool.add_prize(nft_prize);
    //     println!("test insert_prize");
    //     println!("{}",near_sdk::serde_json::to_string(&prize_pool).unwrap());


    //     let x = prize_pool.prizes.len();
    //     assert_eq!(x,2,"插入后长度不匹配")
    // }

    #[test]
    fn deposit_ft() {
        let (mut context, mut contract) = setup_contract();
        contract.ft_on_transfer(accounts(0),123.into(),"".to_string());
        // println!("{}",contract.accounts.get(accounts(0).as_ref()).unwrap());
        println!("{}",near_sdk::serde_json::to_string(&contract.view_account_balance(accounts(0))).unwrap());
    }

    #[test]
    fn test() {
        let env = env!("RUST_ENV");
        println!("{}", env);
    }

    // #[test]
    // fn withdraw_ft() {
    //     let (mut context, mut contract) = setup_contract();
    //     contract.ft_on_transfer(accounts(0),123.into(),"".to_string());
    //     println!("{}",contract.accounts.get(accounts(0).as_ref()).unwrap());
    //     contract.withdraw_ft()
    // }

    fn view_prizes() {
        let context = get_context(vec![], false);
    }

    fn delete_prize() {
        let context = get_context(vec![], false);
    }
}

