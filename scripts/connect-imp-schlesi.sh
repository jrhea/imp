#!/bin/sh
DATA_DIR=/tmp/.imp$$ 
LISTEN_ADDRESS=$(ipconfig getifaddr en0)
TESTNETDIR=../../schlesi/light
BOOTNODES=$(cat ../../schlesi/light/boot_enr.yaml | sed 's/- //g' | sed 's/"//g' | sed 's/enr://g' | tr '\n' , | sed 's/\(.*\),/\1 /')


./../target/debug/imp --debug-level trace mothra --boot-nodes $BOOTNODES --listen-address $LISTEN_ADDRESS --port 13001 --discovery-port 12001 --datadir $DATA_DIR

rm -rf $DATA_DIR