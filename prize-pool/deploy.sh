#!/bin/bash
export ACCOUNT_ID=superise.testnet
export BOB_ACCOUNT_ID=bob.$ACCOUNT_ID
ONE_YOCTO=0.000000000000000000000001
export WHITELIST_ADMIN=whitelist_admin.$ACCOUNT_ID

export CONTRACT_ID=prizepool.$ACCOUNT_ID
export NODE_URL=https://public-rpc.blockpi.io/http/near-testnet

#创建合约账户
#near create-account $CONTRACT_ID --masterAccount=$ACCOUNT_ID --initialBalance=10

#删除合约账户
#near delete prizepool.superise.testnet superise.testnet

# reference: https://github.com/near/core-contracts/tree/master/state-cleanup
# 1. deploy state_cleanup wasm
#near deploy $CONTRACT_ID out/state_cleanup.wasm --node_url $NODE_URL &&
## 2. cleanup state
#python3 state-clean.py prizepool.superise.testnet xsb.testnet &&
## 3. redeploy my contract
#bash ./build.sh && near deploy $CONTRACT_ID out/prize-pool.wasm new '{"white_list_admin": "'$WHITELIST_ADMIN'"}' --node_url $NODE_URL

# 删除重新发布一条龙
#near call prizepool.superise.testnet clear --accountId xsb.testnet --gas 60000000000000 &&
#near delete prizepool.superise.testnet xsb.testnet &&
#near create-account $CONTRACT_ID --masterAccount=$ACCOUNT_ID --initialBalance=10 &&
#bash ./build.sh && near deploy $CONTRACT_ID out/prize-pool.wasm  new '{"white_list_admin": "'$WHITELIST_ADMIN'"}'

# 编译&发布
bash ./build.sh && near deploy $CONTRACT_ID out/prize-pool.wasm --node_url $NODE_URL # new '{}'