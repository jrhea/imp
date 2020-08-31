#!/bin/bash
# sh connect-network.sh altona

NETWORK=$1
ENRS_FILE=$HOME/.$NETWORK/enrs.csv 
BOOTNODES=-LK4QEOMHtTU7PtHBbkKJtxsXIS396CV6kstB_9aBoo4iBRDPvkDj1DIoceAlG7_DCs9g-wAyU6SN0PMtuU98aFDhX4Bh2F0dG5ldHOIAAAAAAAAAACEZXRoMpCi7FS9AAAAAP__________gmlkgnY0gmlwhH8AAAGJc2VjcDI1NmsxoQKxn9xuvQlKHh-UT-WNquUijgAzEW2v7l3Fy_SP-U62HYN0Y3CCMsiDdWRwgjLI
DATA_DIR=$HOME/.imp/datadir
LISTEN_ADDRESS=$(ipconfig getifaddr en0)
mkdir -p $DATA_DIR
RUST_BACKTRACE=full ./../target/debug/imp --debug-level trace mothra --debug-level trace --boot-nodes $BOOTNODES --listen-address $LISTEN_ADDRESS --port 13001 --discovery-port 12001 --datadir $DATA_DIR
#RUST_BACKTRACE=full ./../target/debug/imp --enr-file $ENRS_FILE --debug-level trace mothra --debug-level trace --listen-address $LISTEN_ADDRESS --port 13001 --discovery-port 12001 --datadir $DATA_DIR
rm -rf $DATA_DIR


