#!/bin/sh

DATA_DIR=/tmp/.imp$$ 
LISTEN_ADDRESS=$(ipconfig getifaddr en0)
LOCAL_BOOTNODE=-LK4QHy5-XoIBiyjqqmxtTaCqBUDz01chEDq5xnaMwsKhkf_AXtBNzBgxnrtVHVI20arMVofPjSUMLB5ByIOcMRuSzMBh2F0dG5ldHOIAAAAAAAAAACEZXRoMpCbXd4fAAAAAP__________gmlkgnY0gmlwhMCoAdSJc2VjcDI1NmsxoQKBpGVInMavjaM26V8DfNwnVCRHC_mJzwJMM4gjriXO8oN0Y3CCMsiDdWRwgi7g
BOOTNODES=$LOCAL_BOOTNODE
tmux new-session -d -s foo 'sh start-prysm.sh'
tmux split-window -v -t 0 'sleep 1 && sh start-prysm-validator.sh'
tmux split-window -v -t 1 "sleep 2 && ./../target/debug/imp --p2p-protocol-version gysm/libp2p --debug-level trace mothra --listen-address $LISTEN_ADDRESS --boot-nodes $BOOTNODES --port 13001 --discovery-port 12001 --datadir $DATA_DIR && sleep 2"
tmux select-layout tile
tmux rename-window 'the dude abides'
tmux attach-session -d
rm -rf $DATA_DIR
