use std::borrow::Borrow;
use std::collections::{BinaryHeap, HashMap, HashSet};
use crate::prize::{NftPrize, Prize};
use crate::prize::FtPrize;
use crate::*;
use crate::{NonFungibleTokenId, FungibleTokenId, Contract, StorageKey};
use near_sdk::{assert_one_yocto, near_bindgen, AccountId, env, Timestamp};
use near_sdk::json_types::{U64, ValidAccountId};
use itertools::{Itertools, join};
use near_sdk::collections::{UnorderedMap, UnorderedSet};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::env::{block_timestamp, log};
use near_sdk::serde::{Deserialize, Serialize};
use crate::StorageKey::PrizePools;
use crate::utils::{get_block_milli_time, vec_random};
use std::cmp::Ordering;
use crate::asset::{Asset, Assets, Ft, Nft};

pub type PoolId = u64;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub enum VPool {
    TwitterPool(TwitterPool)
}

impl VPool {
    pub fn into_twitter_pool(self) -> TwitterPool {
        match self { VPool::TwitterPool(pool) => pool }
    }
}

#[derive(PartialEq)]
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub enum PoolStatus {
    PENDING,
    // create but not publish
    ONGOING,
    // after published
    FINISHED,
    DELETED,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Copy)]
#[serde(crate = "near_sdk::serde")]
pub struct PrizeDrawTime(pub MilliTimeStamp, pub PoolId);

impl Eq for PrizeDrawTime {}

impl PartialEq<Self> for PrizeDrawTime {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0) && self.1.eq(&other.1)
    }
}

impl PartialOrd<Self> for PrizeDrawTime {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PrizeDrawTime {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct Record {
    pub time: MilliTimeStamp,
    pub ft_prize: Option<FtPrize>,
    pub nft_prize: Option<NftPrize>,
    pub receiver: AccountId,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct PrizePool {
    pub id: PoolId,
    pub creator_id: AccountId,
    pub ft_prizes: Vec<FtPrize>,
    pub nft_prizes: Vec<NftPrize>,
    pub join_accounts: HashSet<AccountId>,
}

pub trait CountDownDrawPrize {
    fn draw_prize_time(&self) -> MilliTimeStamp;
}

pub trait DrawPrize {
    fn draw_prize(&self) -> HashMap<AccountId, Vec<Prize>>;
}

impl PrizePool {
    pub fn new(
        id: PoolId,
        creator_id: &String,
    ) -> Self {
        PrizePool {
            id,
            creator_id: creator_id.into(),
            ft_prizes: vec![],
            nft_prizes: vec![],
            join_accounts: HashSet::new(),
        }
    }
}

pub fn random_distribution_prizes(ft_prizes: &Vec<FtPrize>,
                                  nft_prizes: &Vec<NftPrize>,
                                  mut joiners: Vec<&AccountId>,
                                  creator: &AccountId) -> HashMap<AccountId, Vec<Prize>> {
    let mut indexs = (0..ft_prizes.len() + nft_prizes.len()).collect_vec();
    let len = indexs.len().clone();
    let mut result: HashMap<AccountId, Vec<Prize>> = HashMap::default();
    for _ in 0..len {
        let receiver = vec_random(&mut joiners).unwrap_or(creator);
        let prize_index = vec_random(&mut indexs).unwrap();
        let prize = if prize_index < ft_prizes.len() {
            Prize::FT_PRIZE(ft_prizes[prize_index].clone())
        } else {
            Prize::NFT_PRIZE(nft_prizes[prize_index - ft_prizes.len()].clone())
        };
        result.entry(receiver.clone()).or_insert(vec![]).push(prize)
    };
    return result;
}

pub struct CreatePrizePoolParam {
    name: String,
    describe: String,
    cover: String,
    fts: Vec<FtPrize>,
    nfts: Vec<NftPrize>,
    begin_time: U64,
    end_time: U64,
}

#[near_bindgen]
impl Contract {

    #[private]
    fn prize_draw(&mut self, pool_id: PoolId) {
        let mut pool = self.internal_get_twitter_pool(&pool_id);
        // 1. check time
        let time_now = get_block_milli_time();
        assert!(pool.end_time <= time_now, "pool end_time ({}) is before block_timestamp({})", pool.end_time, time_now);
        // 2. internal transfer prize to user
        let user_prize_map = pool.draw_prize();
        user_prize_map.iter()
            .for_each(|(account_id, prizes)| {
                let mut account = self.internal_get_account(&account_id);
                prizes.iter().for_each(|prize| {
                    let mut record = Record{
                        time: get_block_milli_time(),
                        ft_prize: None,
                        nft_prize: None,
                        receiver: account_id.clone()
                    };
                    match prize {
                        Prize::NFT_PRIZE(nft_prize) => {
                            account.assets.deposit_nft(&nft_prize.nft);
                            record.nft_prize = Some(nft_prize.clone());
                        }
                        Prize::FT_PRIZE(ft_prize) => {
                            account.assets.deposit_ft(&ft_prize.ft);
                            record.ft_prize = Some(ft_prize.clone());
                        }
                    }
                    pool.records.push(record);
                });
                self.internal_save_account(&account_id, account);
            });

        pool.status = PoolStatus::FINISHED;
        self.internal_save_twitter_pool(pool);
        // self.twitter_prize_pools.insert(&pool_id,&pool);
    }

    // pub fn view_prize_pool(&self, pool_id: u64) -> PrizePool {
    //     self.prize_pools.get(&pool_id.into()).expect("nonexistent pool id")
    // }

    pub fn view_prize_pool_queue(&self) -> Vec<PrizeDrawTime> {
        // return self.pool_queue.into_iter().collect_vec();
        return self.pool_queue.iter().map(|e| e.clone()).collect_vec();
    }

    // 访问是否有开奖的奖池
    pub fn touch_pools(&mut self) {
        log!("block time is {}",get_block_milli_time());
        while !self.pool_queue.is_empty() && self.pool_queue.peek().unwrap().0 <= get_block_milli_time() {
            let pool = self.pool_queue.pop().unwrap();
            log!("pool {} start prize_draw at block_time: {}",pool.1, get_block_milli_time());
            // 只有存在的奖池才会继续
            match self.twitter_prize_pools.get(&pool.1) {
                None => {}
                Some(_) => { self.prize_draw(pool.1) }
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod test_twitter {
    use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
    use near_sdk::log;
    use crate::*;
    use crate::asset::Ft;
    use crate::prize::{FtPrize, FtPrizeCreateParam, Prize};
    use crate::TwitterPool;
    use crate::tests::setup_contract;
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk_sim::lazy_static_include::syn::export::str;
    use crate::twitter_giveaway::TwitterPoolCreateParam;

    #[test]
    fn test_create_param() {
        let param = TwitterPoolCreateParam {
            name: None,
            describe: None,
            cover: None,
            end_time: None,
            white_list: None,
            requirements: None,
            ft_prizes: Some(vec![FtPrizeCreateParam {
                ft: Ft {
                    contract_id: "wrap.testnet".to_string(),
                    balance: U128::from(1000000000000000000000000),
                }
            }]),
            nft_prizes: None,
            join_accounts: None,
            twitter_link: None,
        };
        let (mut context, mut contract) = setup_contract();
        contract.internal_deposit_ft(accounts(0).as_ref(), &"wrap.testnet".to_string(), &U128::from(1000000000000000000000000));
        let id = contract.create_twitter_pool(param);
        let pool = contract.view_twitter_prize_pool(id);
        let pool_des = near_sdk::serde_json::to_string(&pool).unwrap();
        println!("{:?}", pool_des);
    }

    #[test]
    fn test_create() {
        const CREATE_PARAM_RAW: &str = r#"{
"ft_prizes": [
      {
        "ft": {
          "contract_id": "wrap.testnet",
          "balance": "1000000000000000000000000"
        }
      }
    ],
    "nft_prizes": []
}"#;

        // tests::setup_contract()
        let (mut context, mut contract) = setup_contract();
        contract.internal_deposit_ft(accounts(0).as_ref(), &"wrap.testnet".to_string(), &U128::from(1000000000000000000000000));

        let param = near_sdk::serde_json::from_str(CREATE_PARAM_RAW).unwrap();

        let pool = contract.create_twitter_pool(param);
        println!("{:?}", pool)
    }
}