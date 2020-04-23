#!/bin/sh
DATA_DIR1=/tmp/.imp1$$ 
DATA_DIR2=/tmp/.imp2$$ 
TESTNETDIR=$HOME/.lighthouse/testnet
BOOTNODES=$(cat ~/.lighthouse/beacon/network/enr.dat)

tmux new-session -d -s foo 'sh start-lighthouse.sh'
tmux split-window -v -t 0 'sleep 1 && sh start-lighthouse-validator.sh'
tmux split-window -v -t 1 "sleep 2 && ./../target/debug/imp --testnet-dir $TESTNETDIR --debug-level trace mothra --boot-nodes $BOOTNODES --auto-ports --datadir $DATA_DIR1"
tmux split-window -v -t 1 "sleep 3 && ./../target/debug/imp --testnet-dir $TESTNETDIR --debug-level trace mothra --boot-nodes $BOOTNODES --auto-ports --datadir $DATA_DIR2"
tmux select-layout tile
tmux rename-window 'the dude abides'
tmux attach-session -d
rm -rf $DATA_DIR1
rm -rf $DATA_DIR2