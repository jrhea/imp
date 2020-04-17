rm -rf ~/.lighthouse
./release/lcli new-testnet
./release/lcli interop-genesis 8
./release/lighthouse --debug-level trace bn --testnet-dir ~/.lighthouse/testnet --dummy-eth1 --http --enr-match