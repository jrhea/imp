#!/bin/bash

# Usage: sh crawl-network.sh schlesi|topaz num_crawlers snapshot|timehsitory|none

trap post_process EXIT

function post_process() {
    sleep 10
    if [ "$OUTPUT_MODE" != "none" ]; then
        echo "Post processing starting..."
        rm -f $DATA_DIR/enrs.txt
        # group by node-id, seq_no, taking the highest seq no in each group and saving the enr
        if [ -z "$FORK_DIGEST" ]; then
            tail -n+2 $DATA_DIR/crawler* | sed 's/\".*\"//g' |  cut -d',' -f3,14,16 | sort -t ',' -k1,1 -k2,2nr -s -u | sort -t ',' -u -k1,1| cut -d',' -f3 |sed -e "s/^enr://" > $DATA_DIR/enrs.txt
        else
            tail -n+2 $DATA_DIR/crawler* | grep $FORK_DIGEST | sed 's/\".*\"//g' |  cut -d',' -f3,14,16 | sort -t ',' -k1,1 -k2,2nr -s -u | sort -t ',' -u -k1,1| cut -d',' -f3 |sed -e "s/^enr://" > $DATA_DIR/enrs.txt
        fi
        echo "Post processing complete"
        echo "exit"
    fi
    kill 0
}

NETWORK=$1
NUM_CRAWLERS=$2
OUTPUT_MODE=$3

IP_ADDRESS=
if [ "$(uname)" == "Darwin" ]; then
    IP_ADDRESS=$(ipconfig getifaddr en0)      
elif [ "$(expr substr $(uname -s) 1 5)" == "Linux" ]; then
    IP_ADDRESS=$(hostname -i)
fi

FORK_DIGEST=
BOOTSTRAP_BOOTNODES=
if [ $NETWORK = "witti" ]; then
    FORK_DIGEST=f6775d07
    BOOTSTRAP_BOOTNODES=$(curl -s https://raw.githubusercontent.com/goerli/witti/master/lighthouse/boot_enr.yaml | tr -d '"' | sed -e "s/^- enr://" | tr "\n" "," | sed -e "s/,$//g")
elif [ $NETWORK = "topaz" ]; then
    FORK_DIGEST=f071c66c
    BOOTSTRAP_BOOTNODES=-Ku4QAGwOT9StqmwI5LHaIymIO4ooFKfNkEjWa0f1P8OsElgBh2Ijb-GrD_-b9W4kcPFcwmHQEy5RncqXNqdpVo1heoBh2F0dG5ldHOIAAAAAAAAAACEZXRoMpAAAAAAAAAAAP__________gmlkgnY0gmlwhBLf22SJc2VjcDI1NmsxoQJxCnE6v_x2ekgY_uoE1rtwzvGy40mq9eD66XfHPBWgIIN1ZHCCD6A
else
    echo network $NETWORK "not supported"
    exit 1
fi

TIMESTAMP=$(date +%s)
DATA_DIR=$HOME/.$NETWORK
BACKUP_DIR=$HOME/.imp/$TIMESTAMP
if [ $HOME = "/" ]; then
    if [[ -z "${PWD//*\/scripts*/}" ]]; then
        DATA_DIR=$PWD/../.$NETWORK
        BACKUP_DIR=$PWD/../.imp/$TIMESTAMP
    else
        DATA_DIR=$PWD/.$NETWORK
        BACKUP_DIR=$PWD/.imp/$TIMESTAMP
    fi
fi
mkdir -p $DATA_DIR
mkdir -p $BACKUP_DIR

FILE_BOOTNODES=
BOOTNODES=
if [ -f $DATA_DIR/enrs.txt ]; then 
    echo "Additional bootnodes found in file"
    FILE_BOOTNODES=$(cat $DATA_DIR/enrs.txt | tr "\n" "," | sed -e "s/,$//g")
    BOOTNODES=$BOOTSTRAP_BOOTNODES,$FILE_BOOTNODES
else
    BOOTNODES=$BOOTSTRAP_BOOTNODES
fi

if [ "$OUTPUT_MODE" != "none" ]; then
    echo "Backing up $DATA_DIR to $BACKUP_DIR"
    cp -r $DATA_DIR $BACKUP_DIR/
    rm -f $DATA_DIR/crawler*
fi
PORT=12000
for i in $(seq 1 $NUM_CRAWLERS); do
    echo cat $DATA_DIR/crawler$PORT.csv
    #RUST_LOG=libp2p_discv5=debug
    ./../target/debug/imp --p2p-protocol-version imp/libp2p --debug-level debug crawler --output-mode $OUTPUT_MODE --datadir $DATA_DIR --listen-address $IP_ADDRESS --port $PORT --fork-digest "$FORK_DIGEST" --boot-nodes $BOOTNODES &
    let PORT++;
done

wait 
