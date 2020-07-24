#!/bin/bash
cd ../../clients/lighthouse/scripts/local_testnet/

./clean.sh
./setup.sh
./beacon_node.sh trace