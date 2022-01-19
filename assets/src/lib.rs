use near_sdk::AccountId;
use near_sdk::collections::LookupMap;
use crate::asset::Asset;
use near_sdk::Promise;

mod asset;
mod apply;
mod actions;

near_sdk::setup_alloc!();

pub type ContractId = String;
pub type NftId = String;
pub type MethodName = String;
pub type TrackerId = String;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct AssetContract {
    usable: LookupMap<AccountId, Asset>,
    using: LookupMap<AccountId, Asset>
}


#[near_bindgen]
impl AssetContract {
    pub fn applyAsset() {

    }

}
