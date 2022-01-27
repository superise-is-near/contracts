use crate::TrackerId;
use crate::MethodName;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::io::Error;
use near_sdk::{AccountId, Balance, log};
use crate::{ContractId, NftId};


pub enum Asset {
    Ft(Ft),
    Nft(Nft),
}

#[derive(Debug)]
pub struct Ft {
    pub contract_id: ContractId,
    pub balance: Balance,
}

#[derive(Debug)]
pub struct Nft {
    pub contract_id: ContractId,
    pub nft_id: NftId,
}

#[derive(Default)]
pub struct Assets {
    pub fts: HashMap<ContractId, Balance>,
    pub nfts: HashMap<ContractId, HashSet<NftId>>,
}
impl Assets {
    pub fn deposit_ft(&mut self, account_id: &AccountId, amount: &Balance) {
        let x = self.fts.get(account_id).unwrap_or(&0);
        self.fts.insert(account_id.clone(), *x + amount);

        log!("{}" , )
    }

    pub fn deposit_nft(&mut self, contract_id: &ContractId, nft_id: &NftId) {
        if !self.nfts.contains_key(contract_id) {
            self.nfts.insert(contract_id.clone(), HashSet::default());
        }
        self.nfts.get(contract_id).unwrap().insert(nft_id.clone());
    }

    pub fn withdraw_ft(&mut self, ft: &Ft) {
        let balance = self.fts.get(&ft.contract_id).unwrap_or(&0);
        if *balance < ft.balance {
            panic!("Fail to withdraw ft {:?}, account balance is {}",ft, balance);
        }
        self.fts.insert(ft.contract_id.clone(), *balance - ft.balance);
    }

    pub fn withdraw_nft(&mut self, nft: &Nft) {
        if self.nfts.contains_key(&nft.contract_id) && self.nfts.get(&nft.contract_id).unwrap().contains(&nft.nft_id) {
            self.nfts.get(&nft.contract_id).unwrap().remove(&nft.nft_id);
        }
        panic!("Fail to withdraw nft {:?}, no such nft", nft);
    }
}


struct Operator {
    contract: ContractId,
    method: MethodName,
}

struct Operation {
    from: AccountId,
    to: AccountId,
    assets: Assets,
    msg: String
}

struct AssetsOperate{
    operator: Operator,
    operations: Vec<Operation>,
    tracker_id: TrackerId
}