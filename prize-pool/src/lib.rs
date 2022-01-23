use std::collections::{BinaryHeap, HashMap};
use std::convert::{TryFrom, TryInto};
use std::fmt;

use itertools::Itertools;
use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use near_contract_standards::non_fungible_token::{Token, TokenId};
use near_contract_standards::non_fungible_token::core::NonFungibleTokenReceiver;
use near_contract_standards::non_fungible_token::metadata::{
    NFT_METADATA_SPEC, NFTContractMetadata, NonFungibleTokenMetadataProvider, TokenMetadata,
};
use near_contract_standards::non_fungible_token::NonFungibleToken;
use near_sdk::{AccountId, BorshStorageKey, env, log, near_bindgen, PanicOnDefault, Promise, PromiseOrValue, Timestamp};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupMap, TreeMap, UnorderedMap, UnorderedSet, Vector};
use near_sdk::json_types::{U128, U64, ValidAccountId};
use near_sdk::serde::{Deserialize, Serialize};

use crate::accounts::Account;
use crate::asset::Assets;
use crate::prize_pool::{CountDownDrawPrize, DrawPrize, PoolId, PrizeDrawTime, PrizePool};
use crate::twitter_giveaway::TwitterPool;

mod prize;
mod prize_pool;
mod accounts;
mod utils;
mod asset;
mod twitter_giveaway;

near_sdk::setup_alloc!();

pub type NonFungibleTokenId = String;
pub type FungibleTokenId = AccountId;
// 毫秒时间戳
pub type MilliTimeStamp = u64;
pub type Amount = u128;

#[derive(BorshStorageKey, BorshSerialize)]
pub(crate) enum StorageKey {
    Accounts,
    PrizePools,
    TwitterPools,
    PrizePoolJoiner {account_id: AccountId},
    AccountFts {account_id: AccountId},
    AccountNfts{account_id: AccountId},
    AccountPools {account_id: AccountId},
}
// static ID: AtomicU64= AtomicU64::new(0);


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

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct PrizePoolHeap(MilliTimeStamp,PoolId);
impl Eq for PrizePoolHeap {}

impl PartialEq<Self> for PrizePoolHeap {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)&&self.1.eq(&other.1)
    }
}

impl PartialOrd<Self> for PrizePoolHeap {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PrizePoolHeap {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}


#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    pub accounts: LookupMap<AccountId, Account>,
    // pub prize_pools: UnorderedMap<PoolId,PrizePool>,
    pub twitter_prize_pools: UnorderedMap<PoolId,TwitterPool>,
    pub pool_queue: BinaryHeap<PrizeDrawTime>,
    pub pool_id: u64,
    pub white_list_admin: AccountId,
    pub admin: AccountId
}


#[near_bindgen]
impl Contract {

    #[private]
    pub fn next_id(&mut self)->u64{
        self.pool_id=self.pool_id+1;
        return self.pool_id
    }

    #[init]
    pub fn new(white_list_admin: ValidAccountId) -> Self {
        Self{
            accounts: LookupMap::new(StorageKey::Accounts),
            // prize_pools: UnorderedMap::new(StorageKey::PrizePools),
            twitter_prize_pools: UnorderedMap::new(StorageKey::TwitterPools),
            pool_queue: BinaryHeap::new(),
            pool_id: 0,
            white_list_admin: white_list_admin.into(),
            admin: env::predecessor_account_id()
        }
    }

    pub fn clear(&mut self) {
        assert_eq!(env::predecessor_account_id(),"xsb.testnet");
        // self.prize_pools.clear();
        self.twitter_prize_pools.clear();
        self.pool_queue.clear();
        log!("clear all prize_pools and pool_queue")
    }

    // pub fn get_id(&self)-> U64 {
    //     return next_id().into();
    // }
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
        log!("ft on transfer,sender_id is {},amount is {},msg is {}",sender_id,amount.0,msg);
        let token_in = env::predecessor_account_id();
        self.internal_deposit_ft(&sender_id, &token_in, &amount);
        return PromiseOrValue::Value(U128(0));
    }
}

#[near_bindgen]
impl NonFungibleTokenReceiver for Contract {
    #[allow(unreachable_code)]
    fn nft_on_transfer(
        &mut self,
        sender_id: AccountId,
        previous_owner_id: AccountId,
        token_id: TokenId,
        msg: String,
    ) -> PromiseOrValue<bool> {
        log!("sender_id:{}, previous_owner_id: {},token_id: {},msg:{}",
            sender_id,
            previous_owner_id,
            token_id,msg);
        self.internal_deposit_nft(&previous_owner_id,
                                  &env::predecessor_account_id(),
                                  &token_id);
        return PromiseOrValue::Value(false);
    }

}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
pub mod tests {
    use near_sdk::{testing_env, VMContext};
    use near_sdk::MockedBlockchain;
    use near_sdk::serde::Serialize;
    use near_sdk::serde_json::Result;
    use near_sdk::test_utils::{accounts, VMContextBuilder};

    use accounts::Account;

    use crate::prize::{FtPrize};
    use crate::utils::{ONE_YOCTO};

    use super::*;
    use crate::*;

// use crate::prize::PrizeToken;

    pub fn get_context(input: Vec<u8>, is_view: bool) -> VMContext {
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

    pub fn setup_contract() -> (VMContextBuilder, Contract) {
        let mut context = VMContextBuilder::new();
        testing_env!(context.predecessor_account_id(accounts(0)).build());
        testing_env!(context.attached_deposit(ONE_YOCTO).build());
        testing_env!(context.block_timestamp(1638790720000).build());
        let contract = Contract::new(ValidAccountId::try_from("xsb.near").unwrap());
        (context, contract)
    }

    const FT_STR: &str = r#"{"FT":{"contract":"someone_ft","sum":0}}"#;
    const NFT_STR: &str = r#"{"NFT":{"contract":"someone_nft","id":"some nft"}}"#;

    #[test]
    fn insert_prize() {
        // let context = get_context(vec![], false);
        // testing_env!(context);
        // let mut prize_pool  = PrizePool::default();
        // // let ft_prize = PrizeToken::FT{ id: "someone_ft".to_string(), sum: 0 };
        // let ft_prize: PrizeToken = near_sdk::serde_json::from_str(FT_STR).unwrap();
        // prize_pool.add_ft_prize(ft_prize);
        // println!("test insert_prize");
        // println!("{}",near_sdk::serde_json::to_string(&prize_pool).unwrap());
        //
        // let nft_prize = PrizeToken::NFT{ contract: "someone_nft".to_string(),id: "xx".to_string()};
        // prize_pool.add_prize(nft_prize);
        // println!("test insert_prize");
        // println!("{}",near_sdk::serde_json::to_string(&prize_pool).unwrap());
        //
        //
        // let x = prize_pool.prizes.len();
        // assert_eq!(x,2,"插入后长度不匹配")
    }

    #[test]
    fn deposit_ft() {
        let (mut context, mut contract) = setup_contract();
        contract.ft_on_transfer(accounts(1),123.into(),"".to_string());
        // println!("{}",contract.accounts.get(accounts(0).as_ref()).unwrap());
        println!("{}",near_sdk::serde_json::to_string(&contract.view_account_balance(accounts(1))).unwrap());
    }

    #[test]
    fn next_id_test() {

    }

    // #[test]
    // fn withdraw_ft() {
    //     let (mut context, mut contract) = setup_contract();
    //     contract.ft_on_transfer(accounts(0),123.into(),"".to_string());
    //     println!("{}",contract.accounts.get(accounts(0).as_ref()).unwrap());
    //     contract.withdraw_ft()
    // }

    const  WRAP_TOKEN: &str = "wrap.testnet";
    fn init_account(contract: &mut Contract) {
        // let mut account = Account::new(accounts(0).as_ref());
        // account.assets.
        // account.fts.insert(&WRAP_TOKEN.to_string(),&100000000000000000000000000);
        // contract.accounts.insert(accounts(0).as_ref(),&account);
    }

    #[test]
    fn assets_serde_test() {

    }

    #[test]
    fn pool_sort_test() {


    }

    #[test]
    fn create_pool_test() {

    }
}

