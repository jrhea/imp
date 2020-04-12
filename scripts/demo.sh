#!/bin/sh

tmux new-session -d -s foo './../target/debug/imp mothra --datadir ~/.imp --debug-level trace'
tmux split-window -v -t 0 'sleep 2 && ./../target/debug/imp mothra --boot-nodes $(cat ~/.imp/network/enr.dat) --port 9001 --datadir /tmp/.imp --debug-level trace'
tmux select-layout tile
tmux rename-window 'the dude abides'
tmux attach-session -d
