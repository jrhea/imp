CURRENT_TIME=$(date +%s)
GENESIS_TIME=$((CURRENT_TIME + 90))

./../../clients/prysm/prysm.sh beacon-chain --verbosity trace \
--disable-discv5 \
--interop-eth1data-votes \
--interop-genesis-time $GENESIS_TIME \
--interop-num-validators 64 \
--rpc-host $(ipconfig getifaddr en0) \
--p2p-tcp-port 13000 \
--p2p-udp-port 12000 \
--deposit-contract 0x8A04d14125D0FDCDc742F4A05C051De07232EDa4

#--wait-for-synced

#--rpc-host 127.0.0.1
#--force-clear-db \