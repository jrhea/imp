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
            cat $DATA_DIR/enrs.txt | tr "\n" "," | sed -e "s/,$//g" > $DATA_DIR/enrs.csv
        else
            tail -n+2 $DATA_DIR/crawler* | grep $FORK_DIGEST | sed 's/\".*\"//g' |  cut -d',' -f3,14,16 | sort -t ',' -k1,1 -k2,2nr -s -u | sort -t ',' -u -k1,1| cut -d',' -f3 |sed -e "s/^enr://" > $DATA_DIR/enrs.txt
            cat $DATA_DIR/enrs.txt | tr "\n" "," | sed -e "s/,$//g" > $DATA_DIR/enrs.csv
            tail -n+2 $DATA_DIR/crawler* | grep $FORK_DIGEST | sed 's/\".*\"//g' | grep -v "\[\]"|  cut -d',' -f3,14,15,16 | sort -t ',' -k1,1 -k2,2nr -s -u | sort -t ',' -u -k1,1| cut -d',' -f4 | sed -e "s/^enr://" > $DATA_DIR/validating_enrs.txt
             cat $DATA_DIR/validating_enrs.txt | tr "\n" "," | sed -e "s/,$//g" > $DATA_DIR/validating_enrs.csv
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
elif [ $NETWORK = "altona" ]; then
    FORK_DIGEST=fdca39b0
    BOOTSTRAP_BOOTNODES=$(curl -s https://raw.githubusercontent.com/eth2-clients/eth2-testnets/master/shared/altona/bootstrap_nodes.txt | grep "enr:" | sed -e "s/^enr://" | tr "\n" "," |sed -e "s/,$//g")
elif [ $NETWORK = "onyx" ]; then
    FORK_DIGEST=a65b4897
    BOOTSTRAP_BOOTNODES=-LK4QNtJfsgcW7OsSWmx0viM1EfhtteFr_AEmQbKBDiO731DWFhpckZmCD0lX_QKwIO5HkkUcxhQ_8PSG1SsoLQIEJEeh2F0dG5ldHOIYAICAAAAAASEZXRoMpCmW0iXAAAAAP__________gmlkgnY0gmlwhEj8IuqJc2VjcDI1NmsxoQKMxUzwsbHy_0xq0jK8PCc3zKudGv2N0EE9B7f0ObbJ4oN0Y3CCMsiDdWRwgi7g,-LS4QHj0e2Kw5z8Ha-GtNbaxdHd7FieB0ER3sm0L59AwGQt4TBZPNnOEN-78a5S5JJWl3xTta0dwfQR37zKC_-je_8CCBbqHYXR0bmV0c4gGKFAAQKREFYRldGgykKZbSJcAAAAA__________-CaWSCdjSCaXCEfEdtC4lzZWNwMjU2azGhAlRtklD9MhHYWowLMGQX1bkvFRVhlXWQAXlAoXaISma2g3RjcIIyyIN1ZHCCLuA,-LK4QHt4MMEQRHBWHAG1PmkremYEaWi0L1GzzZTL9eEza1L-G5gBJlow92B5GVzEJeAxMw6kbFxRJTdYTwh3xvZCoVNwh2F0dG5ldHOIAEEUGAADECCEZXRoMpD9yjmwAAABIf__________gmlkgnY0gmlwhFNVv3SJc2VjcDI1NmsxoQP23W3m9AVsrd68UEhKL5Bwpkq47fDDOVgDoAfc3zM60YN0Y3CCIyiDdWRwgiMo
elif [ $NETWORK = "medalla" ]; then
    FORK_DIGEST=e7a75d5a
    BOOTSTRAP_BOOTNODES=$(curl -s https://raw.githubusercontent.com/goerli/medalla/master/medalla/bootnodes.txt | grep "enr:" | sed -e "s/^enr:/,/" | tr -d "\n" | tr -d "\r" |sed -e "s/^,//g")
elif [ $NETWORK = "prysm-attack" ]; then
    FORK_DIGEST=c354a54a
    BOOTSTRAP_BOOTNODES=$(curl -s https://raw.githubusercontent.com/ethereum/public-attacknets/master/attacknets/beta-0/prysm-attack-0/lighthouse/boot_enr.yaml | grep "enr:" | sed -e "s/^- enr://" | tr "\n" "," |sed -e "s/,$//g")
elif [ $NETWORK = "lighthouse-attack" ]; then
    FORK_DIGEST=80e1769b
    BOOTSTRAP_BOOTNODES=$(curl -s https://raw.githubusercontent.com/ethereum/public-attacknets/master/attacknets/beta-0/lighthouse-attack-0/lighthouse/boot_enr.yaml | grep "enr:" | sed -e "s/^- enr://" | tr "\n" "," |sed -e "s/,$//g")
elif [ $NETWORK = "teku-attack" ]; then
    FORK_DIGEST=157d3034
    BOOTSTRAP_BOOTNODES=$(curl -s https://raw.githubusercontent.com/ethereum/public-attacknets/master/attacknets/beta-0/teku-attack-0/lighthouse/boot_enr.yaml| grep "enr:" | sed -e "s/^- enr://" | tr "\n" "," |sed -e "s/,$//g")
    #BOOTSTRAP_BOOTNODES=-LK4QFqcQToZriXkTP-_oP1sVIXoynwTWG5yBrTUw2v04kDqcWEISm5pKrX7q2gVj3fetOENcJ-nFOIKxIKFYa78R2kQh2F0dG5ldHOIerk4dAIt-SeEZXRoMpAVfTA0AAAAAP__________gmlkgnY0gmlwhA1yJbCJc2VjcDI1NmsxoQOirrUOnSvlsumnw9K-ZZElP5fmBph8j2uA0HuqU9lHuIN0Y3CCIyiDdWRwgiMo
elif [ $NETWORK = "mc-attack" ]; then
    FORK_DIGEST=2e44918e
    BOOTSTRAP_BOOTNODES=-LK4QC4pexs3ghjGOItTkTttDiow--WuQqjtieE0YRVKxnvHWAUjt2GKH8-WRDoj8ZSOIBJodAWG-ZftKOPwqTK8QtxBh2F0dG5ldHOI__________-EZXRoMpAuRJGOAACwHf__________gmlkgnY0gmlwhDZBuWeJc2VjcDI1NmsxoQOuTMWhnh6C8oEcCNEyLueXHOJD4zsZr06rGRFC-pbDzIN0Y3CCIyiDdWRwgiMo,-LK4QGaMPlU00DocKfM4ciAnzn44SWYfDNw0Vk2nVj8Uv7r8KEhO_wqbHzEWAzBpjuQLBIwXj91sQCHFR_kDMglcf4IGh2F0dG5ldHOI__________-EZXRoMpAuRJGOAACwHf__________gmlkgnY0gmlwhDRBowiJc2VjcDI1NmsxoQLNz5fvAiF0KiVdV9YU_AucPRlA9vtgaCfpxpiA9MSrB4N0Y3CCIyiDdWRwgiMo,-LK4QKT94GydM4k0rV9LNIQn3vGwn8qO8Osh-3wSV970M241IT7hCG17Vzv5fiMAJ95b4v6Pkp814X0OAc-SWzM7pnAEh2F0dG5ldHOI__________-EZXRoMpAuRJGOAACwHf__________gmlkgnY0gmlwhAMZfQmJc2VjcDI1NmsxoQKVusiPi55UDoIJS6R0A_wiygNr4ZK-vbYWwoJ6Fbb4E4N0Y3CCIyiDdWRwgiMo
elif [ $NETWORK = "ad-hoc" ]; then
    FORK_DIGEST=a2ec54bd
    BOOTSTRAP_BOOTNODES=-LK4QI-uUnpZRDPrqcCHNbriK0MhS3DUAPyCHTTrIJ2VlFUTCQ4gMqN9cU7j0M-lGOuejIyHtZ6FIJqyQXKVENNqeqUBh2F0dG5ldHOIAAAAAAAAAACEZXRoMpCi7FS9AAAAAP__________gmlkgnY0gmlwhH8AAAGJc2VjcDI1NmsxoQPkffo8ZFXvYkkQkUTjTSL73ZBa9-qm8yySp6aW4LhsKYN0Y3CCIymDdWRwgiM
else
    echo network $NETWORK "not supported"
    exit 1
fi

TIMESTAMP=$(date +%s)
DATA_DIR=$HOME/.$NETWORK
BACKUP_DIR=$HOME/.imp/$NETWORK/$TIMESTAMP
if [ $HOME = "/" ]; then
    if [[ -z "${PWD//*\/scripts*/}" ]]; then
        DATA_DIR=$PWD/../.$NETWORK
        BACKUP_DIR=$PWD/../.imp/$NETWORK/$TIMESTAMP
    else
        DATA_DIR=$PWD/.$NETWORK
        BACKUP_DIR=$PWD/.imp/$NETWORK/$TIMESTAMP
    fi
fi
mkdir -p $DATA_DIR
mkdir -p $BACKUP_DIR


BOOTNODES=
BOOTNODES=$BOOTSTRAP_BOOTNODES

if [ "$OUTPUT_MODE" != "none" ]; then
    echo "Backing up $DATA_DIR to $BACKUP_DIR"
    cp -r $DATA_DIR $BACKUP_DIR/
    rm -f $DATA_DIR/crawler*
fi
PORT=12000
for i in $(seq 1 $NUM_CRAWLERS); do
    echo cat $DATA_DIR/crawler$PORT.csv
    if [ -f $DATA_DIR/enrs.csv ]; then 
        echo "Additional bootnodes found in file"
        RUST_LOG=libp2p_discv5=debug ./../target/debug/imp --p2p-protocol-version imp/libp2p --enr-file $DATA_DIR/enrs.csv --debug-level trace crawler --output-mode $OUTPUT_MODE --datadir $DATA_DIR --listen-address $IP_ADDRESS --port $PORT --fork-digest "$FORK_DIGEST" --boot-nodes $BOOTNODES &
    else
        RUST_LOG=libp2p_discv5=debug ./../target/debug/imp --p2p-protocol-version imp/libp2p --debug-level trace crawler --output-mode $OUTPUT_MODE --datadir $DATA_DIR --listen-address $IP_ADDRESS --port $PORT --fork-digest "$FORK_DIGEST" --boot-nodes $BOOTNODES &
    fi
    
    let PORT++;
done

wait 
