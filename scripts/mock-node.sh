#!/bin/sh

tmux new-session -d -s foo './../sim/mock-node/target/debug/mock-node mothra --datadir ~/.imp --debug-level trace'
tmux split-window -v -t 0 'sleep 2 && ./../sim/mock-node/target/debug/mock-node mothra --boot-nodes $(cat ~/.imp/network/enr.dat) --port 9001 --datadir /tmp/.imp --debug-level trace'
tmux select-layout tile
tmux rename-window 'the dude abides'
tmux attach-session -d
