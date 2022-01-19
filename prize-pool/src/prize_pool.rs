use std::borrow::Borrow;
use std::collections::{BinaryHeap, HashMap, HashSet};
use crate::prize::{NftPrize, Prize};
use crate::prize::FtPrize;
use crate::*;
use crate::{NonFungibleTokenId, FungibleTokenId, Contract, StorageKey};
use crate::prize::{PrizeToken};
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


enum PoolEnum {
    TwitterPool(TwitterPool)
}


#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct PrizeDrawTime(MilliTimeStamp,PoolId);
impl Eq for PrizeDrawTime {}

impl PartialEq<Self> for PrizeDrawTime{
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)&&self.1.eq(&other.1)
    }
}

impl PartialOrd<Self> for PrizeDrawTime{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PrizeDrawTime {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}


#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct PrizePoolDisplay {
    pub id: PoolId,
    pub creator_id: AccountId,
    pub name: String,
    pub describe: String,
    pub cover: String,
    pub end_time: MilliTimeStamp,
    pub joiner_sum: usize,
    pub ticket_price: Ft,
    pub finish: bool,
}

impl From<PrizePool> for PrizePoolDisplay {
    fn from(prize_pool: PrizePool) -> Self {
        return PrizePoolDisplay {
            id: prize_pool.id,
            creator_id: prize_pool.creator_id,
            name: prize_pool.name,
            describe: prize_pool.describe,
            cover: prize_pool.cover,
            end_time: prize_pool.end_time,
            joiner_sum: prize_pool.join_accounts.len(),
            ticket_price: Ft { contract_id: prize_pool.ticket_token_id, balance: prize_pool.ticket_price.0 },
            finish: prize_pool.finish,
        };
    }
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Record {
    time: MilliTimeStamp,
    ft_prize: FtPrize,
    receiver: AccountId,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct PrizePool {
    pub id: PoolId,
    pub creator_id: AccountId,
    pub ft_prizes: Vec<FtPrize>,
    pub nft_prizes: Vec<NftPrize>,
    pub join_accounts: HashSet<AccountId>,
}

pub trait CountDownDrawPrize {
    fn draw_prize_time(&self)->MilliTimeStamp;

}

pub trait DrawPrize {
   fn draw_prize(&self)-> HashMap<AccountId, Vec<Prize>>;
}

pub trait RandomDistributePrize: DistributePrize {
    fn random_distribute_prize(&self) -> HashMap<AccountId, Vec<Prize>>;
}

impl PrizePool {
    pub fn new(
        id: PoolId,
        creator_id: &String,
        name: String,
        describe: String,
        cover: String,
        ticket_prize: U128,
        ticket_token_id: String,
        end_time: MilliTimeStamp,
    ) -> Self {
        PrizePool {
            id,
            creator_id: creator_id.into(),
            name,
            describe,
            cover,
            ft_prizes: vec![],
            nft_prizes: vec![],
            ticket_price: ticket_prize.into(),
            ticket_token_id,
            join_accounts: HashSet::new(),
            end_time,
            finish: false,
            records: vec![],
        }
    }
}

pub fn random_distribution_prizes(ft_prizes: &Vec<FtPrize>,
                                  nft_prizes: &Vec<NftPrize>,
                                  mut joiners: Vec<&AccountId>,
                                  creator: &AccountId) -> HashMap<AccountId, Vec<Prize>> {
    let mut indexs = (0..ft_prizes.len() + nft_prizes.len()).collect_vec();
    let len = indexs.len();
    let mut result: HashMap<ccountId, Vec<Prize>> = HashMap::default();
    for _ in len {
        let receiver = vec_random(&mut joiners).unwrap_or(creator);
        let prize_index = vec_random(&mut indexs).unwrap();
        let prize = if prize_index < ft_prizes.len() { Prize::FT_PRIZE(ft_prizes[prize_index].clone()) } else { Prize::NFT_PRIZE(nft_prizes[prize_index].clone()) };
        result.entry(key).or_insert(vec![]).push(prize)
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
    pub fn random_distribution_prizes(&mut self,
                                      ft_prizes: &Vec<FtPrize>,
                                      nft_prizes: &Vec<NftPrize>,
                                      mut joiners: Vec<&AccountId>,
                                      creator: &AccountId) {
        let mut indexs = (0..ft_prizes.len()).chain((0..nft_prizes).map(|e| e + ft_prizes.len())).collect_vec();
        let len = indexs.len();
        for _ in len {
            let receiver = vec_random(&mut joiners).unwrap_or(creator);
            let mut account = self.accounts.get(receiver).unwrap_or(Account::new(receiver));
            let prize_index = vec_random(&mut indexs).unwrap();
            if prize_index < ft_prizes.len() { account.assets.deposit_ft(&ft_prizes[prize_index].ft.contract_id, &ft_prizes[prize_index].ft.balance) } else { account.assets.deposit_nft(&nft_prizes[prize_index - ft_prizes.len()].nft.contract_id, &nft_prizes[prize_index - ft_prizes.len()].nft.nft_id) }
            self.accounts.insert(&account.name, &account);
        }
    }

    #[payable]
    pub fn create_prize_pool(&mut self,
                             name: String,
                             describe: String,
                             cover: String,
                             ticket_prize: U128,
                             ticket_token_id: String,
                             end_time: u64,
                             fts: Option<Vec<FtPrize>>,
                             nfts: Option<Vec<NftPrize>>) -> PrizePool {
        assert_one_yocto();
        assert!(ticket_prize.0 >= 0, "ticket_prize:{} should >=0", ticket_prize.0);
        // 检测账户余额是否充足
        let account_id = env::predecessor_account_id();
        let mut account = self.accounts.get(&account_id).unwrap_or(Account::new(&account_id));
        let mut ft_prizes = fts.unwrap_or(vec![]);
        let mut nft_prizes = nfts.unwrap_or(vec![]);
        for ft_prize in &mut ft_prizes {
            account.assets.withdraw_ft(&ft_prize.ft.contract_id, &ft_prize.ft.balance);
            ft_prize.prize_id = self.next_id();
        }
        for nft_prize in &mut nft_prizes {
            account.assets.withdraw_nft(&nft_prize.nft.contract_id, &nft_prize.nft.nft_id);
            nft_prize.prize_id = self.next_id();
        }
        // update account
        self.accounts.insert(&account_id, &account);

        // 创建奖池
        let id = self.next_id();
        let pool = self.prize_pools.insert(
            &id,
            &PrizePool {
                id,
                creator_id: account_id,
                name,
                describe,
                cover,
                ticket_price: ticket_prize,
                ticket_token_id,
                join_accounts: HashSet::default(),
                end_time: end_time.into(),
                finish: false,
                ft_prizes,
                nft_prizes,
                records: vec![],
            }).expect("create prize pool error");

        // 更新用户的拥有记录
        let mut account = self.accounts.get(&account_id).unwrap_or(Account::new(&account_id));
        account.pools.insert(id);
        self.accounts.insert(&account_id, &account);
        self.pool_queue.push(PrizePoolHeap(end_time, id));
        return pool;
    }

    #[private]
    fn prize_draw(&mut self, pool_id: PoolId) {
        let mut pool = self.prize_pools.get(&pool_id).expect("pool id didn't exist");
        // 1. 检测时间是否合法
        let time_now = get_block_milli_time();
        assert!(pool.end_time <= time_now, "pool end_time ({}) is before block_timestamp({})", pool.end_time, time_now);
        // 2. 分发奖品

        let mut joiners = pool.join_accounts.iter().collect_vec();
        for ft_prize in &pool.ft_prizes {
            let receiver = vec_random(&mut joiners).unwrap_or(&pool.creator_id);
            let mut account = self.accounts.get(receiver).unwrap_or(Account::new(receiver));
            account.deposit_ft(&ft_prize.token_id, &ft_prize.amount.0);
            // 更新账户
            self.accounts.insert(receiver, &account);
            // 保存记录
            pool.records.push(Record { time: get_block_milli_time(), receiver: receiver.clone(), ft_prize: ft_prize.clone() })
        }
        // 3. 设置状态,添加记录
        pool.finish = true;
        self.prize_pools.insert(&pool_id, &pool);
        // 4. 把参与者的门票发给奖池创建者。
        let mut creator_account = self.accounts.get(&pool.creator_id).unwrap_or(Account::new(&pool.creator_id));
        let ticket_token_id_balance = creator_account.fts.get(&pool.ticket_token_id).unwrap_or(0);
        let ticket_amount = (pool.join_accounts.len() as u128) * pool.ticket_price.0;
        creator_account.fts.insert(&pool.ticket_token_id, &(ticket_token_id_balance + ticket_amount));
        self.accounts.insert(&pool.creator_id, &creator_account);
    }

    pub fn delete_prize_pool(&mut self, pool_id: u64) {
        // 1. 找到Prize pool
        let pool = self.prize_pools.get(&pool_id).expect("prize pool not exist");
        // 2. 如果pool 没有finish，返回资产
        if !pool.finish {
            // 2.1 返回创建者资产
            let mut pool_create_account = self.accounts.get(&pool.creator_id).unwrap_or(Account::new(&pool.creator_id));
            for ft in &pool.ft_prizes {
                pool_create_account.deposit_ft(&ft.token_id, &ft.amount.0);
            }
            self.accounts.insert(&pool.creator_id, &pool_create_account);

            // 2.2 返回参与者资产
            for joiner in &pool.join_accounts {
                let mut account = self.accounts.get(&joiner).unwrap_or(Account::new(&joiner));
                account.deposit_ft(&pool.ticket_token_id, &pool.ticket_price.0);
            }
        }
        // 3. 删除pool, 任务队列中会check pool_id是否存在。
        self.prize_pools.remove(&pool_id);
    }

    pub fn view_prize_pool(&self, pool_id: u64) -> PrizePool {
        self.prize_pools.get(&pool_id.into()).expect("nonexistent pool id")
    }

    pub fn view_prize_pool_list(&self) -> Vec<PrizePoolDisplay> {
        let sort_ord = |a: &PrizePoolDisplay, b: &PrizePoolDisplay| {
            if a.finish == b.finish { a.end_time.cmp(&b.end_time) } else if a.finish { Ordering::Greater } else { Ordering::Less }
        };
        return self.prize_pools.values()
            .map(|v| v.into())
            .sorted_by(sort_ord)
            .collect_vec();
    }

    pub fn join_pool(&mut self, pool_id: u64) {
        let joiner_id = env::predecessor_account_id();
        let mut pool = self.prize_pools.get(&pool_id.into()).expect(&format!("no such pool,id:{}", pool_id).to_string());
        assert!(!pool.finish, "Can't join a finished prize pool");
        // 检测是否已经参加过了。
        assert!(!pool.join_accounts.contains(&joiner_id), "{} has joined this prize pool", joiner_id);

        let mut account = self.accounts
            .get(&joiner_id)
            .unwrap_or(Account::new(&joiner_id));
        let balance = account.fts.get(&pool.ticket_token_id).unwrap_or(0);
        assert!(balance >= pool.ticket_price.0, "{} is less than ticket price", pool.ticket_token_id);
        account.fts.insert(&pool.ticket_token_id, &(balance - pool.ticket_price.0));
        account.pools.insert(pool_id);
        pool.join_accounts.insert(String::from(&joiner_id));
        self.prize_pools.insert(&pool_id, &pool);
        self.accounts.insert(&joiner_id, &account);
    }

    pub fn unjoin_pool(&mut self, pool_id: U64) {
        let mut pool = self.prize_pools.get(&pool_id.into()).expect(&format!("no such pool,id:{}", pool_id.0).to_string());
        assert!(!pool.finish, "Can't join a finished prize pool");
        let joiner_id = env::predecessor_account_id();
        assert!(!pool.join_accounts.contains(&joiner_id), "{} didn't join this prize pool", joiner_id);
        let mut account = self.accounts
            .get(&joiner_id)
            .unwrap_or(Account::new(&joiner_id));
        let balance = account.fts.get(&pool.ticket_token_id).unwrap_or(0);
        account.fts.insert(&pool.ticket_token_id, &(balance + pool.ticket_price.0));
        pool.join_accounts.remove(&joiner_id);
        account.pools.remove(&pool_id.0);
    }

    pub fn view_prize_pool_queue(&self) -> Vec<PrizePoolHeap> {
        return self.pool_queue.iter().map(|e| PrizePoolHeap(e.0, e.1)).collect_vec();
    }

    // 访问是否有开奖的奖池
    pub fn touch_pools(&mut self) {
        log!("block time is {}",get_block_milli_time());
        while !self.pool_queue.is_empty() && self.pool_queue.peek().unwrap().0 <= get_block_milli_time() {
            let pool = self.pool_queue.pop().unwrap();
            log!("pool {} start prize_draw at block_time: {}",pool.1, get_block_milli_time());
            // 只有存在的奖池才会继续
            match self.prize_pools.get(&pool.1) {
                None => {}
                Some(prize_pool) => { self.prize_draw(prize_pool.id) }
            }
        }
    }
}

