use std::collections::{BinaryHeap, HashMap};
use std::convert::{TryFrom, TryInto};
use std::fmt;
use std::sync::atomic::AtomicU64;

use itertools::Itertools;
use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use near_contract_standards::non_fungible_token::{Token, TokenId};
use near_contract_standards::non_fungible_token::metadata::{
    NFT_METADATA_SPEC, NFTContractMetadata, NonFungibleTokenMetadataProvider, TokenMetadata,
};
use near_contract_standards::non_fungible_token::NonFungibleToken;
use near_sdk::{AccountId, BorshStorageKey, env, log, near_bindgen, PanicOnDefault, Promise, PromiseOrValue, Timestamp};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::borsh::maybestd::sync::atomic::{AtomicUsize, Ordering};
use near_sdk::collections::{LazyOption, LookupMap, TreeMap, UnorderedMap, UnorderedSet, Vector};
use near_sdk::json_types::{U128, U64, ValidAccountId};
use near_sdk::serde::{Deserialize, Serialize};

use crate::accounts::Account;
use crate::prize::Prize;
use crate::prize_pool::{PoolId, PrizePool};

mod prize;
mod prize_pool;
mod accounts;
mod utils;

near_sdk::setup_alloc!();

pub type NonFungibleTokenId = String;
pub type FungibleTokenId = AccountId;
// 毫秒时间戳
pub type MilliTimeStamp = u64;

#[derive(BorshStorageKey, BorshSerialize)]
pub(crate) enum StorageKey {
    Accounts,
    PrizePools,
    AccountTokens {account_id: AccountId},
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
    pub prize_pools: UnorderedMap<PoolId,PrizePool>,
    pub pool_queue: BinaryHeap<PrizePoolHeap>, // who own pools
    pub pool_id: u64
    // pub own_pools: LookupMap<AccountId, UnorderedSet<u64>>
}


#[near_bindgen]
impl Contract {

    #[private]
    pub fn next_id(&mut self)->u64{
        self.pool_id=self.pool_id+1;
        return self.pool_id
    }

    #[init]
    pub fn new() -> Self {
        Self{
            accounts: LookupMap::new(StorageKey::Accounts),
            prize_pools: UnorderedMap::new(StorageKey::PrizePools),
            pool_queue: BinaryHeap::new(),
            pool_id: 0
        }
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
        self.internal_deposit(sender_id.as_ref(), &token_in, amount.into());
        PromiseOrValue::Value(U128(0))
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests {
    use near_sdk::{testing_env, VMContext};
    use near_sdk::MockedBlockchain;
    use near_sdk::serde::Serialize;
    use near_sdk::serde_json::Result;
    use near_sdk::test_utils::{accounts, VMContextBuilder};

    use accounts::Account;

    use crate::prize::{FtPrize, PrizeToken};
    use crate::utils::{ONE_YOCTO};

    use super::*;

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
        testing_env!(context.predecessor_account_id(accounts(0)).build());
        testing_env!(context.attached_deposit(ONE_YOCTO).build());
        testing_env!(context.block_timestamp(1638790720000).build());
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
        let mut account = Account::new(accounts(0).as_ref());
        account.fts.insert(&WRAP_TOKEN.to_string(),&100000000000000000000000000);
        contract.accounts.insert(accounts(0).as_ref(),&account);
    }

    #[test]
    fn pool_sort_test() {
        let (mut context, mut contract) = setup_contract();
        let factory = |id: PoolId,time: MilliTimeStamp,finish: bool|{
            let mut pool = PrizePool::new(id, &"".to_string(), "".to_string(), "".to_string(),
                                      "".to_string(), U128::from(123), "".to_string(),
                                      time,
                                      vec![], vec![]);
            pool.finish = finish;
            return pool;
        };
        contract.prize_pools.insert(&1, &factory(1,5,true));
        contract.prize_pools.insert(&2, &factory(2,6,false));
        contract.prize_pools.insert(&3, &factory(3,7, false));
        contract.prize_pools.insert(&4, &factory(4,5, false));
        let vec1 = contract.view_prize_pool_list().iter().map(|e| e.id).collect_vec();
        println!("{:?}", vec1);

    }

    #[test]
    fn create_pool_test() {
        let cover = "https://image.baidu.com/search/detail?ct=503316480&z=undefined&tn=baiduimagedetail&ipn=d&word=%E7%9B%B2%E7%9B%92&step_word=&ie=utf-8&in=&cl=2&lm=-1&st=undefined&hd=undefined&latest=undefined&copyright=undefined&cs=984361998,1860976251&os=3817620326,3085336574&simid=3457770807,390738890&pn=0&rn=1&di=187440&ln=1891&fr=&fmq=1638858788964_R&fm=&ic=undefined&s=undefined&se=&sme=&tab=0&width=undefined&height=undefined&face=undefined&is=0,0&istype=0&ist=&jit=&bdtype=0&spn=0&pi=0&gsm=0&objurl=https%3A%2F%2Fpics6.baidu.com%2Ffeed%2F50da81cb39dbb6fd9a5ecf17fcf5541e962b37d6.jpeg%3Ftoken%3Ddea499573d7eaa207f0e0869ce32382a&rpstart=0&rpnum=0&adpicid=0&nojc=undefined&dyTabStr=MCwzLDIsMSw2LDUsNCw3LDgsOQ%3D%3D";
        let (mut context, mut contract) = setup_contract();
        init_account(&mut contract);
        let ticket_prize: U128 = U128("1000000000000000000000000".parse().unwrap());
        let fts = vec![FtPrize{ token_id: WRAP_TOKEN.to_string(), amount: U128(1000000000000000000000000u128) }];
        let pool_id = contract.create_prize_pool("name".into(), "desc".into(),
                                             cover.into(),
                                             ticket_prize,
                                             "wrap.testnet".to_string(),
                                                 1638790620000,
                                             Some(fts),
                                             None);
        assert_eq!(contract.prize_pools.len(), 1, "奖池集合长度不是1");
        assert_eq!(contract.pool_queue.len(), 1, "奖池任务队列长度不是1");
        println!("init account state: {}", near_sdk::serde_json::to_string(&contract.view_account_balance(accounts(0))).unwrap());
        contract.join_pool(pool_id);
        contract.touch_pools();
        println!("pool after prize withdraw: {}", near_sdk::serde_json::to_string(&contract.view_prize_pool(pool_id)).unwrap());
        println!("account after prize withdraw: {}", near_sdk::serde_json::to_string(&contract.view_account_balance(accounts(0))).unwrap());
        println!("pool list after prize withdraw: {}", near_sdk::serde_json::to_string(&contract.view_prize_pool_list()).unwrap());
    }
}

