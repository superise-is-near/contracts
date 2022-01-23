use crate::*;
use std::collections::{HashMap, HashSet};
use std::iter::FromIterator;
use std::process::id;
use itertools::{Itertools, join};
use near_sdk::{assert_one_yocto, env, near_bindgen};
use near_sdk::json_types::ValidAccountId;
use crate::{Account, AccountId, Assets, Contract, CountDownDrawPrize, DrawPrize, MilliTimeStamp, PoolId, PrizeDrawTime, PrizePool};
use crate::prize::{FtPrize, FtPrizeCreateParam, NftPrize, NftPrizeCreateParam, Prize};
use crate::prize_pool::{random_distribution_prizes};
use crate::StorageKey::TwitterPools;
use crate::ContractContract;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use crate::asset::Ft;


type TwitterAccount = String;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct TwitterPoolDisplay {
    pub id: u64,
    pub name: String,
    pub describe: String,
    pub cover: String,
    pub finish: bool,
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
            finish: pool.finish,
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

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct TwitterPool {
    pub name: String,
    pub describe: String,
    pub cover: String,
    pub prize_pool: PrizePool,
    pub finish: bool,
    pub end_time: MilliTimeStamp,
    pub white_list: HashSet<AccountId>,
    pub requirements: String,
    pub twitter_near_bind: HashMap<AccountId, TwitterAccount>,
    pub twitter_link: String,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct TwitterPoolCreateParam {
    pub name: String,
    pub describe: String,
    pub cover: String,
    pub end_time: MilliTimeStamp,
    pub white_list: Option<Vec<AccountId>>,
    pub requirements: Option<String>,
    pub ft_prizes: Option<Vec<FtPrizeCreateParam>>,
    pub nft_prizes: Option<Vec<NftPrizeCreateParam>>,
    pub join_accounts: Option<Vec<AccountId>>,
    pub twitter_link: String,
}

impl TwitterPool {
    pub fn new_by_near_call(param: &TwitterPoolCreateParam, creator_id: &AccountId, id_gen: fn() -> u64) -> Self {
        Self {
            name: param.name.clone(),
            describe: param.describe.clone(),
            cover: param.cover.clone(),
            prize_pool: PrizePool {
                id: id_gen(),
                creator_id: creator_id.clone(),
                ft_prizes: param.ft_prizes.as_ref().unwrap_or(&vec![]).iter().map(|e| FtPrize { ft: Ft{ contract_id: e.ft.contract_id.clone(), balance: e.ft.balance.0 }, prize_id: id_gen() }).collect_vec(),
                nft_prizes: param.nft_prizes.as_ref().unwrap_or(&vec![]).iter().map(|e| NftPrize { nft: e.nft.clone(), prize_id: id_gen() }).collect_vec(),
                join_accounts: HashSet::from_iter(param.join_accounts.as_ref().unwrap_or(&vec![]).iter().map(|e| e.clone())),
            },
            finish: false,
            end_time: 0,
            white_list: param.white_list.as_ref().unwrap_or(&vec![]).iter().map(|e| e.clone()).collect(),
            requirements: param.requirements.as_ref().unwrap_or(&"".to_string()).clone(),
            twitter_near_bind: Default::default(),
            twitter_link: param.twitter_link.clone(),
        }
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
    pub fn new_twitter_pool_by_create_param(&mut self, param: &TwitterPoolCreateParam) -> TwitterPool {
        TwitterPool {
            name: param.name.clone(),
            describe: param.describe.clone(),
            cover: param.cover.clone(),
            prize_pool: PrizePool {
                id: self.next_id(),
                creator_id: env::predecessor_account_id(),
                ft_prizes: param.ft_prizes.as_ref().unwrap_or(&vec![]).iter().map(|e| FtPrize {
                    ft: Ft { contract_id: e.ft.contract_id.clone(), balance: e.ft.balance.0 },
                    prize_id: self.next_id(),
                }).collect_vec(),
                nft_prizes: param.nft_prizes.as_ref().unwrap_or(&vec![]).iter().map(|e| NftPrize { nft: e.nft.clone(), prize_id: self.next_id() }).collect_vec(),
                join_accounts: HashSet::from_iter(param.join_accounts.as_ref().unwrap_or(&vec![]).iter().map(|e| e.clone())),
            },
            finish: false,
            end_time: param.end_time,
            white_list: param.white_list.as_ref().unwrap_or(&vec![]).iter().map(|e| e.clone()).collect(),
            requirements: param.requirements.as_ref().unwrap_or(&"".to_string()).clone(),
            twitter_near_bind: Default::default(),
            twitter_link: param.twitter_link.clone(),
        }
    }

    #[payable]
    pub fn create_twitter_pool(&mut self, param: TwitterPoolCreateParam) -> TwitterPool {
        assert_one_yocto();
        let creator_id = env::predecessor_account_id();
        // todo user should register first
        let mut account = self.accounts.get(&creator_id).unwrap_or(Account::new(&creator_id));

        param.ft_prizes.as_ref().unwrap_or(&vec![]).iter().for_each(|x| account.assets.withdraw_contract_amount(&x.ft.contract_id,&x.ft.balance.0));
        param.nft_prizes.as_ref().unwrap_or(&vec![]).iter().for_each(|x| account.assets.withdraw_nft(&x.nft));
        // let pool = TwitterPool::new_by_near_call(&param,&creator_id,(self.next_id)(self));
        let pool = self.new_twitter_pool_by_create_param(&param);
        self.twitter_prize_pools.insert(&pool.prize_pool.id, &pool);
        self.pool_queue.push((&pool).into());
        return pool;
    }

    pub fn join_twitter_pool(&mut self, pool_id: u64) {
        let mut pool = self.twitter_prize_pools.get(&pool_id).expect(&format!("no such pool,id:{}", pool_id));
        let joiner = env::predecessor_account_id();
        assert!(pool.white_list.contains(&joiner), "you are not in whitelist");
        pool.prize_pool.join_accounts.insert(joiner);
        self.twitter_prize_pools.insert(&pool.prize_pool.id, &pool);
    }

    //todo unjoin twitter pool
    pub fn unjoin_twitter_pool(&mut self, pool_id: PoolId) {}

    pub fn view_twitter_prize_pool(&self, pool_id: PoolId) -> TwitterPool {
        return self.twitter_prize_pools.get(&pool_id).expect("inexistent pool id");
    }

    pub fn update_whitelist(&mut self) {}

    pub fn add_user_into_whitelist(&mut self, param: TwitterPoolWhiteListParam) {
        let mut pool = self.twitter_prize_pools.get(&param.pool_id).expect("inexistent pool id");
        let signer = env::predecessor_account_id();
        // check authority
        assert!(signer == pool.prize_pool.creator_id || signer == self.white_list_admin, "no authority change whitelist");
        assert!(pool.twitter_near_bind.values().find(|&e| e.eq(&param.twitter_account)).is_none(),
                format!("this twitter account {} has been used!", param.twitter_account));
        pool.white_list.insert(param.account.into());
        self.twitter_prize_pools.insert(&pool.prize_pool.id, &pool);
    }

    pub fn view_twitter_prize_pool_list(&self) -> Vec<TwitterPoolDisplay> {
        return self.twitter_prize_pools.values().map_into().collect_vec();
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
    use crate::prize::{FtCreateParam, FtPrizeCreateParam};
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