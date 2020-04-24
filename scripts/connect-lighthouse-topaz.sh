#RUST_LOG=libp2p_discv5=debug \ 
./../../clients/lighthouse/release/lighthouse --debug-level trace bn  --datadir ~/.lighthouse --testnet-dir ./../../clients/eth2-testnets/prysm/Topaz\(v0.11.1\) --port 13002 --discovery-port 12002 --listen-address $(ipconfig getifaddr en0) --enr-match
