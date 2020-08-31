PORT=$1

CURRENT_TIME=$(date +%s)
GENESIS_TIME=$((CURRENT_TIME + 10))

rm -f prysm-local-network/$PORT/prysm.logrm -f prysm-local-network/$PORT/prysm.log
mkdir -p prysm-local-network/$PORT

./../../clients/prysm/prysm.sh beacon-chain --verbosity trace \
--datadir $PWD/prysm-local-network/$PORT \
--interop-eth1data-votes \
--interop-genesis-time $GENESIS_TIME \
--interop-num-validators 64 \
--rpc-host $(ipconfig getifaddr en0) \
--p2p-tcp-port $PORT \
--p2p-udp-port $PORT \
--deposit-contract 0x8A04d14125D0FDCDc742F4A05C051De07232EDa4 \
--p2p-local-ip $(ipconfig getifaddr en0) \
--bootstrap-node enr:-LK4QCL3slk-CztmDMt666666666666666666665rjNE50E277777777777777777777777777777777777777777777777777777777777777777777777777777777777777__________gmlkgnY0gmlwhAN_hmeJc2VjcDI1NmsxoQMef-fQtsXp-Hs6BdBgL15k9GxSO8zqQsTeDNxAgTUHmYN0Y3CCIyiDdWRwgiMo \
--log-file $PWD/prysm-local-network/$PORT/prysm.log \
--force-clear-db

#--wait-for-synced

#--rpc-host 127.0.0.1
#--force-clear-db \
# --p2p-host-ip "$(curl -s v4.ident.me)" \


