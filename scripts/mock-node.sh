#!/bin/sh

./../target/debug/mock-node mothra --boot-nodes $(cat ~/.lighthouse/beacon/network/enr.dat) --auto-ports --datadir /tmp/.mock-node --debug-level trace
