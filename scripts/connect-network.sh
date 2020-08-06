#!/bin/bash
# sh connect-network.sh altona

NETWORK=$1
ENRS_FILE=$HOME/.$NETWORK/enrs.csv 
DATA_DIR=$HOME/.imp/datadir
LISTEN_ADDRESS=$(ipconfig getifaddr en0)
mkdir -p $DATA_DIR
RUST_BACKTRACE=full ./../target/debug/imp --enr-file $ENRS_FILE --debug-level trace mothra --debug-level trace --listen-address $LISTEN_ADDRESS --port 13001 --discovery-port 12001 --datadir $DATA_DIR
rm -rf $DATA_DIR


