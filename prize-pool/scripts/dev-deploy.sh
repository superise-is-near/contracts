#!/bin/bash
set -e
source ./variables.sh

# 编译&发布
bash ./build.sh && near dev-deploy out/prize-pool.wasm new '{"white_list_admin": "'$WHITELIST_ADMIN'"}' --node_url $NODE_URL
