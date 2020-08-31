./../../clients/prysm/prysm.sh validator \
--keymanager=interop \
--keymanageropts='{"keys":64}' \
--datadir $PWD/prysm-data \
--disable-accounts-v2 \
--beacon-rpc-provider 192.168.1.212:4000 \
--force-clear-db 