./../../clients/prysm/prysm.sh beacon-chain --verbosity info \
--force-clear-db \
--disable-discv5 \
--interop-eth1data-votes \
--interop-genesis-time $(date +%s) \
--interop-num-validators 64 \
--rpc-host $(ipconfig getifaddr en0) \
--p2p-tcp-port 9000 \
--p2p-udp-port 9000

#--wait-for-synced

#--rpc-host 127.0.0.1
