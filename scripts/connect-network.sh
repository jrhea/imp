#!/bin/bash
# sh connect-network.sh witti

NETWORK=$1
DATA_DIR=$HOME/.imp/$NETWORK
LISTEN_ADDRESS=$(ipconfig getifaddr en0)
TESTNETDIR=../../$NETWORK/lighthouse
BOOTNODES=$(cat ~/.$NETWORK/enrs.txt | tr "\n" "," | sed -e "s/,$//g")


./../target/debug/imp --debug-level info mothra --debug-level info --boot-nodes $BOOTNODES --listen-address $LISTEN_ADDRESS --port 13001 --discovery-port 12001 --datadir $DATA_DIR

rm -rf $DATA_DIR