#!/bin/bash

./../../clients/lighthouse/target/release/lcli insecure-validators --count 8 --validators-dir  ~/.lighthouse/validators  --secrets-dir ~/.lighthouse/secrets
./../../clients/lighthouse/target/release/lighthouse vc --testnet-dir ~/.lighthouse/testnet 