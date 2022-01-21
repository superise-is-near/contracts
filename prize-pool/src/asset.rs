use std::borrow::BorrowMut;
use crate::*;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::io::Error;
use near_sdk::{AccountId, Balance};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};

pub type ContractId = String;
pub type NftId = String;

pub enum Asset {
    Ft(Ft),
    Nft(Nft),
}

#[derive(BorshDeserialize, BorshSerialize,Debug,Serialize,Deserialize,Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Ft {
    pub contract_id: String,
    pub balance: Balance,
}

#[derive(BorshDeserialize, BorshSerialize,Debug,Serialize,Deserialize,Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Nft {
    pub contract_id: String,
    pub nft_id: NftId,
}

#[derive(BorshSerialize, BorshDeserialize,Debug,Clone,Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Assets {
    pub fts: HashMap<ContractId, Balance>,
    pub nfts: HashMap<ContractId, HashSet<NftId>>,
}

impl Default for Assets {
    fn default() -> Self {
        return Assets {
            fts: Default::default(),
            nfts: Default::default(),
        };
    }
}

impl Assets {

    pub fn deposit_asset(&mut self, asset: &Asset) {
        match asset {
            Asset::Ft(ft)=>self.deposit_ft(ft),
            Asset::Nft(nft) => self.deposit_nft(nft)
        }
    }

    pub fn deposit_ft(&mut self, ft: &Ft) {
        self.deposit_contract_amount(&ft.contract_id,&ft.balance);
    }

    pub fn deposit_contract_amount(&mut self, contract_id: &ContractId,  amount: &Amount) {
        let x = self.fts.get(contract_id).unwrap_or(&0);
        self.fts.insert(contract_id.clone(), *x + *amount);
    }

    pub fn deposit_nft(&mut self, nft: &Nft) {
        self.deposit_contract_nft_id(&nft.contract_id,&nft.nft_id);
    }

    pub fn deposit_contract_nft_id(&mut self, contract_id: &ContractId, nft_id: &NftId) {
        self.nfts.entry(contract_id.clone())
            .or_insert(Default::default())
            .insert(nft_id.clone());
    }

    pub fn withdraw_ft(&mut self, ft: &Ft) {
        let balance = self.fts.get(&ft.contract_id).unwrap_or(&0);
        if *balance < ft.balance {
            panic!("Fail to withdraw ft {{contract_id: {}, amount: {}}}, account balance is {}",ft.contract_id,ft.balance, balance);
        }
        self.fts.insert(ft.contract_id.clone(), *balance - ft.balance);
    }
    pub fn withdraw_contract_amount(&mut self, contract_id: &ContractId, amount: &Amount) {
        let balance = self.fts.get(contract_id).unwrap_or(&0);
        if *balance < *amount {
            panic!("Fail to withdraw ft {{contract_id: {}, amount: {}}}, account balance is {}",contract_id,amount, balance);
        }
        self.fts.insert(contract_id.clone(), *balance - *amount);
    }

    pub fn withdraw_nft(&mut self, nft: &Nft) {
        self.withdraw_contract_nft_id(&nft.contract_id,&nft.nft_id);
    }
    pub fn withdraw_contract_nft_id(&mut self, contract_id: &ContractId, nft_id: &NftId) {
        let mut nfts = self.nfts.get_mut(contract_id).expect("nft not exist");
        assert!(nfts.remove(nft_id),"nft not exist");
    }
}