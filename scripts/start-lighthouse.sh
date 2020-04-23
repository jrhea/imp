rm -rf ~/.lighthouse
./../../clients/lighthouse/release/lcli new-testnet
./../../clients/lighthouse/release/lcli interop-genesis 8
./../../clients/lighthouse/release/lighthouse --debug-level trace bn --testnet-dir ~/.lighthouse/testnet --dummy-eth1 --http --enr-match