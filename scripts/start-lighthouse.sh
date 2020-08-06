#!/bin/bash

rm -rf ~/.lighthouse
./../../clients/lighthouse/target/release/lcli new-testnet  --deposit-contract-address 07b39F4fDE4A38bACe212b546dAc87C58DfE3fDC  --deposit-contract-deploy-block 0
./../../clients/lighthouse/target/release/lcli interop-genesis 8
RUST_LOG="libp2p_gossipsub,libp2p_swarm,libp2p_tcp" ./../../clients/lighthouse/target/release/lighthouse --debug-level trace bn --testnet-dir ~/.lighthouse/testnet --dummy-eth1 --http --enr-match 