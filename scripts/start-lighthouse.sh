

PORT=$1
rm -f lighthouse.log
rm -rf ~/.lighthouse
./../../clients/lighthouse/target/release/lcli new-testnet  --deposit-contract-address 07b39F4fDE4A38bACe212b546dAc87C58DfE3fDC  --deposit-contract-deploy-block 0
./../../clients/lighthouse/target/release/lcli interop-genesis 8
#RUST_LOG="libp2p_gossipsub,libp2p_swarm,libp2p_tcp" 
RUST_LOG=discv5 ./../../clients/lighthouse/target/debug/lighthouse \
-l \
bn \
--purge-db \
--datadir ~/.lighthouse \
--target-peers 1 \
--testnet-dir ~/.lighthouse/testnet \
--dummy-eth1 \
--enr-match \
--port $PORT 

#--boot-nodes 
#--http \
