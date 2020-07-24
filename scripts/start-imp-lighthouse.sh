#!/bin/sh
DATA_DIR1=/tmp/.imp1$$ 
DATA_DIR2=/tmp/.imp2$$ 
TESTNETDIR=$HOME/.lighthouse/local-testnet
BOOTNODES=$(cat ~/.lighthouse/local-testnet/beacon/beacon/network/enr.dat)

#tmux new-session -d -s foo 'sh start-lighthouse.sh'
#tmux split-window -v -t 0 'sleep 1 && sh start-lighthouse-validator.sh'
#tmux split-window -v -t 1 "sleep 4 && ./../target/debug/imp --testnet-dir $TESTNETDIR --debug-level trace mothra --boot-nodes $BOOTNODES --auto-ports --datadir $DATA_DIR1"
#tmux split-window -v -t 1 "sleep 5 && ./../target/debug/imp --testnet-dir $TESTNETDIR --debug-level trace mothra --boot-nodes $BOOTNODES --auto-ports --datadir $DATA_DIR2"
#tmux select-layout tile
#tmux rename-window 'the dude abides'
#tmux attach-session -d
./../target/debug/imp --testnet-dir $TESTNETDIR --debug-level trace mothra --boot-nodes $BOOTNODES --port 13001 --discovery-port 12001 --datadir $DATA_DIR1
rm -rf $DATA_DIR1
rm -rf $DATA_DIR2