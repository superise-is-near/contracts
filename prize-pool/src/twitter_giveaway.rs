use crate::*;
use std::collections::{HashMap, HashSet};
use std::iter::FromIterator;
use std::process::id;
use itertools::{Itertools, join};
use near_sdk::{assert_one_yocto, env, near_bindgen};
use near_sdk::json_types::ValidAccountId;
use crate::{Account, AccountId, Assets, Contract, CountDownDrawPrize, DrawPrize, MilliTimeStamp, PoolId, PrizeDrawTime, PrizePool};
use crate::prize::{FtPrize, FtPrizeCreateParam, NftPrize, NftPrizeCreateParam, Prize};
use crate::prize_pool::{PoolStatus, random_distribution_prizes};
use crate::StorageKey::TwitterPools;
use crate::ContractContract;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::serde::de::Unexpected::Str;
use crate::asset::Ft;
use crate::utils::get_block_milli_time;


type TwitterAccount = String;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct TwitterPoolDisplay {
    pub id: u64,
    pub name: String,
    pub describe: String,
    pub cover: String,
    pub status: PoolStatus,
    pub end_time: MilliTimeStamp,
    pub twitter_link: String,
}


impl From<TwitterPool> for TwitterPoolDisplay {
    fn from(pool: TwitterPool) -> Self {
        TwitterPoolDisplay {
            id: pool.prize_pool.id,
            name: pool.name,
            describe: pool.describe,
            cover: pool.cover,
            status: pool.status,
            end_time: pool.end_time,
            twitter_link: pool.twitter_link,
        }
    }
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct TwitterPoolDetail {
    pub name: String,
    pub describe: String,
    pub cover: String,
    pub prize_pool: PrizePool,
    pub finish: bool,
    pub end_time: MilliTimeStamp,
    pub white_list: Vec<AccountId>,
    pub requirement: Vec<String>,
    pub twitter_link: String,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct TwitterPoolWhiteListParam {
    pub pool_id: PoolId,
    pub account: ValidAccountId,
    pub twitter_account: TwitterAccount,
}

impl From<TwitterPool> for VPool {
    fn from(pool: TwitterPool) -> Self {
        Self::TwitterPool(pool)
    }
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct TwitterPool {
    pub name: String,
    pub describe: String,
    pub cover: String,
    pub prize_pool: PrizePool,
    pub status: PoolStatus,
    pub end_time: MilliTimeStamp,
    pub create_time: MilliTimeStamp,
    pub update_time: MilliTimeStamp,
    pub white_list: HashSet<AccountId>,
    pub requirements: String,
    pub twitter_near_bind: HashMap<TwitterAccount, AccountId>,
    pub twitter_link: String,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct TwitterPoolVO {
    pub name: String,
    pub describe: String,
    pub cover: String,
    pub prize_pool: PrizePool,
    pub finish: bool,
    pub end_time: MilliTimeStamp,
    pub create_time: MilliTimeStamp,
    pub white_list: HashSet<AccountId>,
    pub requirements: String,
    pub twitter_near_bind: HashMap<TwitterAccount, AccountId>,
    pub twitter_link: String,
}


#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct TwitterPoolCreateParam {
    pub name: Option<String>,
    pub describe: Option<String>,
    pub cover: Option<String>,
    pub end_time: Option<MilliTimeStamp>,
    pub white_list: Option<Vec<AccountId>>,
    pub requirements: Option<String>,
    pub ft_prizes: Option<Vec<FtPrizeCreateParam>>,
    pub nft_prizes: Option<Vec<NftPrizeCreateParam>>,
    pub join_accounts: Option<Vec<AccountId>>,
    pub twitter_link: Option<String>,
}

impl TwitterPool {
    pub fn publish(&mut self) {
        assert_ne!(self.end_time, UNINITIALIZED_TIME_STAMP, "end_time haven't init");
        self.status = PoolStatus::ONGOING;
    }
}

impl DrawPrize for TwitterPool {
    fn draw_prize(&self) -> HashMap<AccountId, Vec<Prize>> {
        return random_distribution_prizes(&self.prize_pool.ft_prizes,
                                          &self.prize_pool.nft_prizes,
                                          self.prize_pool.join_accounts.iter().collect_vec(),
                                          &self.prize_pool.creator_id);
    }
}

impl From<&TwitterPool> for PrizeDrawTime {
    fn from(pool: &TwitterPool) -> Self {
        return PrizeDrawTime(pool.end_time.clone(), pool.prize_pool.id.clone());
    }
}

#[near_bindgen]
impl Contract {

    pub fn publish_pool(&mut self, pool_id: PoolId) {
        self.internal_creator_use_twitter_pool(
            &pool_id,
            |pool|{
               pool.status = PoolStatus::ONGOING;
            });
    }

    #[private]
    pub(crate) fn internal_creator_use_twitter_pool<F>(&mut self, pool_id: &PoolId, mut f: F)
    where F: FnMut(&mut TwitterPool) {
        let mut pool = self.internal_get_twitter_pool(pool_id);
        assert_eq!(pool.prize_pool.creator_id,env::predecessor_account_id(),"only creator can use pool!");
        f(&mut pool);
        self.internal_save_twitter_pool(pool)
    }

    #[private]
    pub fn internal_get_twitter_pool(&self, id: &PoolId) -> TwitterPool {
        let pool = self.twitter_prize_pools.get(id).expect("pool not exist");
        match pool {
            VPool::TwitterPool(prize_pool) => { prize_pool }
            _ => panic!("error type")
        }
    }

    #[private]
    pub fn internal_save_twitter_pool(&mut self, twitter_pool: TwitterPool) {
        self.twitter_prize_pools.insert(&twitter_pool.prize_pool.id.clone(), &twitter_pool.into());
    }

    #[private]
    fn new_twitter_pool_by_create_param(&mut self, param: &TwitterPoolCreateParam) -> TwitterPool {
        TwitterPool {
            name: param.name.as_ref().unwrap_or(&"".to_string()).clone(),
            describe: param.describe.as_ref().unwrap_or(&"".to_string()).clone(),
            cover: param.cover.as_ref().unwrap_or(&"".to_string()).clone(),
            prize_pool: PrizePool {
                id: self.next_id(),
                creator_id: env::predecessor_account_id(),
                ft_prizes: param.ft_prizes.as_ref().unwrap_or(&vec![]).iter().map(|e| FtPrize {
                    ft: Ft { contract_id: e.ft.contract_id.clone(), balance: e.ft.balance },
                    prize_id: self.next_id(),
                }).collect_vec(),
                nft_prizes: param.nft_prizes.as_ref().unwrap_or(&vec![]).iter().map(|e| NftPrize { nft: e.nft.clone(), prize_id: self.next_id() }).collect_vec(),
                join_accounts: HashSet::from_iter(param.join_accounts.as_ref().unwrap_or(&vec![]).iter().map(|e| e.clone())),
            },
            status: PoolStatus::PENDING,
            end_time: param.end_time.unwrap_or(UNINITIALIZED_TIME_STAMP),
            create_time: get_block_milli_time(),
            update_time: get_block_milli_time(),
            white_list: param.white_list.as_ref().unwrap_or(&vec![]).iter().map(|e| e.clone()).collect(),
            requirements: param.requirements.as_ref().unwrap_or(&"".to_string()).clone(),
            twitter_near_bind: Default::default(),
            twitter_link: param.twitter_link.as_ref().unwrap_or(&"".to_string()).clone(),
        }
    }

    #[private]
    fn update_twitter_pool_by_create_param(&mut self, param: &TwitterPoolCreateParam, pool_id: &PoolId) {
        let mut pool = self.internal_get_twitter_pool(&pool_id);
        if param.name.is_some() { pool.name = param.name.as_ref().unwrap().clone(); }
        if param.describe.is_some() { pool.describe = param.describe.as_ref().unwrap().clone(); }
        if param.cover.is_some() { pool.cover = param.cover.as_ref().unwrap().clone(); }
        if param.ft_prizes.is_some() {
            pool.prize_pool.ft_prizes = param.ft_prizes.as_ref().unwrap_or(&vec![]).iter().map(|e| FtPrize {
                ft: Ft { contract_id: e.ft.contract_id.clone(), balance: e.ft.balance },
                prize_id: self.next_id(),
            }).collect_vec();
        }
        if param.nft_prizes.is_some() {
            pool.prize_pool.nft_prizes = param.nft_prizes.as_ref().unwrap_or(&vec![]).iter().map(|e| NftPrize { nft: e.nft.clone(), prize_id: self.next_id() }).collect_vec();
        }

        if param.end_time.is_some() {pool.end_time = param.end_time.as_ref().unwrap().clone();}
        if param.white_list.is_some() {pool.white_list = param.white_list.as_ref().unwrap_or(&vec![]).iter().map(|e|e.clone()).collect()}
        // if param.requirements.is_some() {pool.requirements = param.requirements.as_ref().unwrap_or(&"".to_string()).clone()}
        if param.twitter_link.is_some() {pool.twitter_link = param.twitter_link.as_ref().unwrap().clone();}
        pool.update_time = get_block_milli_time();
        self.internal_save_twitter_pool(pool);
    }

    #[payable]
    pub fn create_twitter_pool(&mut self, param: TwitterPoolCreateParam) -> PoolId {
        assert_one_yocto();
        let creator_id = env::predecessor_account_id();
        // todo user should register first
        self.internal_use_account(
            &creator_id,
            |account| {
                param.ft_prizes.as_ref().unwrap_or(&vec![]).iter().for_each(|x| account.assets.withdraw_contract_amount(&x.ft.contract_id, &x.ft.balance.0));
                param.nft_prizes.as_ref().unwrap_or(&vec![]).iter().for_each(|x| account.assets.withdraw_nft(&x.nft));
            });

        // let pool = TwitterPool::new_by_near_call(&param,&creator_id,(self.next_id)(self));
        let pool = self.new_twitter_pool_by_create_param(&param);
        let pool_id = pool.prize_pool.id.clone();
        self.pool_queue.push((&pool).into());
        self.internal_save_twitter_pool(pool);
        // self.twitter_prize_pools.insert(&pool.prize_pool.id, &pool.into());
        return pool_id;
    }

    #[payable]
    pub fn update_twitter_pool(&mut self, param: TwitterPoolCreateParam, pool_id: PoolId)-> PoolId {
        assert_one_yocto();
        let updater = env::predecessor_account_id();
        let pool = self.internal_get_twitter_pool(&pool_id);
        assert_eq!(updater, pool.prize_pool.creator_id, "only creator can update!");

        let mut account = self.internal_get_account(&updater);

        pool.prize_pool.ft_prizes.iter().map(|e|&e.ft).for_each(|ft|account.assets.deposit_ft(ft));
        pool.prize_pool.nft_prizes.iter().map(|e|&e.nft).for_each(|nft|account.assets.deposit_nft(nft));

        param.ft_prizes.as_ref().unwrap_or(&vec![]).iter().for_each(|x| account.assets.withdraw_contract_amount(&x.ft.contract_id, &x.ft.balance.0));
        param.nft_prizes.as_ref().unwrap_or(&vec![]).iter().for_each(|x| account.assets.withdraw_nft(&x.nft));

        self.update_twitter_pool_by_create_param(&param,&pool_id);
        return pool_id;
    }

    pub fn join_twitter_pool(&mut self, pool_id: u64) {
        let mut pool = self.internal_get_twitter_pool(&pool_id);//self.twitter_prize_pools.get(&pool_id).expect(&format!("no such pool,id:{}", pool_id));
        assert_eq!(pool.status, PoolStatus::ONGOING, "pool can only join in ongoing status");
        let joiner = env::predecessor_account_id();
        assert!(pool.white_list.contains(&joiner), "you are not in whitelist");
        pool.prize_pool.join_accounts.insert(joiner);
        // self.twitter_prize_pools.insert(&pool.prize_pool.id, &pool);
        self.internal_save_twitter_pool(pool);
    }

    //todo unjoin twitter pool
    // pub fn unjoin_twitter_pool(&mut self, pool_id: PoolId) {}

    pub fn view_twitter_prize_pool(&self, pool_id: PoolId) -> TwitterPool {
        return self.internal_get_twitter_pool(&pool_id);
    }

    pub fn add_user_into_whitelist(&mut self, param: TwitterPoolWhiteListParam) {
        let mut pool = self.internal_get_twitter_pool(&param.pool_id);
        let signer = env::predecessor_account_id();
        // check authority
        assert!(signer == pool.prize_pool.creator_id || signer == self.white_list_admin, "no authority change whitelist");
        assert!(!pool.twitter_near_bind.contains_key(&param.twitter_account),
                format!("this twitter account {} has been used!", param.twitter_account));

        // todo don't save now for test easier;
        // pool.twitter_near_bind.insert(param.twitter_account,param.account.clone().into());
        pool.white_list.insert(param.account.into());
        self.internal_save_twitter_pool(pool);
    }

    pub fn view_twitter_prize_pool_list(&self) -> Vec<TwitterPoolDisplay> {
        return self.twitter_prize_pools.values().map(VPool::into_twitter_pool).map_into().collect_vec();
        // return self.twitter_prize_pools.get(&pool_id).expect("inexistent pool id");
    }
}


#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod test_twitter {
    use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
    use near_sdk::log;
    use crate::*;
    use crate::asset::Ft;
    use crate::TwitterPool;
    use crate::tests::setup_contract;
    use crate::twitter_giveaway::TwitterPoolCreateParam;

    #[test]
    fn test_create() {
        const CREATE_PARAM_RAW: &str = r#"{
    "name": "1",
    "requirements": "[]",
    "twitter_link": "123",
    "white_list": [],
    "cover": "https://justplayproducts.com/wp-content/uploads/2020/06/78550_78551-Ryans-Mystery-Playdate-Mini-Mystery-Boxes-Call-Out-2-scaled-470x470.jpg",
    "describe": "1",
    "end_time": 1642919340000,
    "ft_prizes": [
      {
        "ft": {
          "contract_id": "NEAR",
          "balance": "00000000000000000000000000"
        }
      }
    ],
    "join_accounts": null,
    "nft_prizes": []
}"#;

        // tests::setup_contract()
        let (mut context, mut contract) = setup_contract();
        let param = near_sdk::serde_json::from_str(CREATE_PARAM_RAW).unwrap();

        let pool = contract.create_twitter_pool(param);
        println!("{:?}", pool)
    }
}