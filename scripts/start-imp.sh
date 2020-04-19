DATA_DIR=/tmp/.imp$$ 
./../target/debug/imp --testnet-dir $HOME/.lighthouse/testnet --debug-level trace mothra --boot-nodes $(cat ~/.lighthouse/beacon/network/enr.dat) --auto-ports --datadir $DATA_DIR
rm -rf $DATA_DIR
