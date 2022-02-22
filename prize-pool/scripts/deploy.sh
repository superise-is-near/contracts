#!/bin/bash
set -e

source ./variables.sh

#create contract account
near create-account $PRIZE_POOL_CONTRACT_ID --masterAccount=$CONTRACT_ACCOUNT_ID --initialBalance=10

#delete contract account
#near delete $CONTRACT_ID $ACCOUNT_ID

# 编译&发布
bash ./build.sh && near deploy $PRIZE_POOL_CONTRACT_ID out/prize-pool.wasm new '{"white_list_admin": "'$WHITELIST_ADMIN'"}' --node_url $NODE_URL
