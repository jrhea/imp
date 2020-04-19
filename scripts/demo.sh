#!/bin/sh

tmux new-session -d -s foo 'sh start-imp.sh'
tmux split-window -v -t 0 'sleep 2 && sh start-imp.sh'
tmux select-layout tile
tmux rename-window 'the dude abides'
tmux attach-session -d
