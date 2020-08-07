CURRENT_TIME=$(date +%s)
GENESIS_TIME=$((CURRENT_TIME + 90))

./../../clients/prysm/prysm.sh beacon-chain --verbosity info \
--force-clear-db \
--disable-discv5 \
--interop-eth1data-votes \
--interop-genesis-time $GENESIS_TIME \
--interop-num-validators 64 \
--rpc-host $(ipconfig getifaddr en0) \
--p2p-tcp-port 9000 \
--p2p-udp-port 9000 \
--deposit-contract 0x8A04d14125D0FDCDc742F4A05C051De07232EDa4

#--wait-for-synced

#--rpc-host 127.0.0.1
