use near_sdk::AccountId;
use near_sdk_sim::{deploy, init_simulator, to_yocto};
use prize_pool::{ContractContract as PrizePoolContract, PrizePoolHeap};
fn setup() {
    let root = init_simulator(None);
    let owner = root.create_user("owner".to_string(), to_yocto("100"));
    let prizePool = deploy!(
        contract: PrizePoolContract,
        contract_id: prizepool(),
        bytes: &PRIZPOOL_WASM_BYTES,
        signer_account: root,
        init_method: new()
    );
}