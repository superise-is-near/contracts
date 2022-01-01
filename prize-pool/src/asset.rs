use std::collections::{HashMap, HashSet};
use std::io::Error;
use near_sdk::{AccountId, Balance};

pub type ContractId=String;
pub type NftId=String;
pub enum Asset{
    Ft(Ft),Nft(Nft)
}
pub struct Ft{
    pub contract_id: ContractId,
    pub balance: Balance
}

pub struct Nft{
    pub contract_id: ContractId,
    pub nft_id: NftId
}

pub struct Assets{
    pub fts: HashMap<ContractId, Balance>,
    pub nfts: HashMap<ContractId,HashSet<NftId>>
}

impl Default for Assets {
    fn default() -> Self {
        return Assets{
            fts: Default::default(),
            nfts: Default::default()
        }
    }
}
impl Assets {

    pub fn deposit_ft(&mut self, account_id: &AccountId, amount: &Balance) {
        let x = self.fts.get(&account_id).unwrap_or(&0);
        self.fts.insert(account_id.clone(),*x+amount);
    }

    pub fn deposit_nft(&mut self, contract_id: &ContractId, nft_id: &NftId) {
        if !self.nfts.contains_key(&contract_id) {
            self.nfts.insert(contract_id.clone(),HashSet::default());
        }
        self.nfts.get(&contract_id).unwrap().insert(nft_id.clone());
    }

    pub fn withdraw_ft(&mut self, ft: &Ft)-> Result< (),&'static str>{
        let balance = self.fts.get(&ft.contract_id).unwrap_or(&0);
        if balance < *ft.balance {
            return Result::Err("xxx")
        }
        self.fts.insert(ft.contract_id.clone(),balance-(*ft.balance));
        return Result::Ok(())
    }

    pub fn withdraw_nft(&mut self, nft: &Nft)-> Result<(),&'static str>{
        if self.nfts.contains_key(&nft.contract_id)&& self.nfts.get(&nft.contract_id).unwrap().contains(&nft.nft_id) {
            self.nfts.get(&nft.contract_id).unwrap().remove(&nft.nft_id);
            return Result::Ok(());
        }
        return Result::Err("");
    }
}