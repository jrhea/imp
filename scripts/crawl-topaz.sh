
DATA_DIR=/tmp/.imp$$ 
BOOTNODES=-Ku4QAGwOT9StqmwI5LHaIymIO4ooFKfNkEjWa0f1P8OsElgBh2Ijb-GrD_-b9W4kcPFcwmHQEy5RncqXNqdpVo1heoBh2F0dG5ldHOIAAAAAAAAAACEZXRoMpAAAAAAAAAAAP__________gmlkgnY0gmlwhBLf22SJc2VjcDI1NmsxoQJxCnE6v_x2ekgY_uoE1rtwzvGy40mq9eD66XfHPBWgIIN1ZHCCD6A
#RUST_LOG=libp2p_discv5=debug
./../target/debug/imp --p2p-protocol-version imp/libp2p --debug-level debug crawler --listen-address $(ipconfig getifaddr en0) --port 12001 --boot-nodes $BOOTNODES
rm -rf $DATA_DIR
