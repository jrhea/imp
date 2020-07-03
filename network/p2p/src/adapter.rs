use chrono::Local;
use clap::ArgMatches;
use eth2::ssz::{Decode, Encode};
use eth2::types::{MainnetEthSpec, SignedBeaconBlock};
use eth2::utils::{create_topic_ids, get_fork_id_from_dir, get_fork_id_from_string};
use serde_derive::Serialize;
use slog::{debug, info, o, trace, warn};
use snap::raw::{decompress_len, Decoder, Encoder};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::SystemTime;
use types::events::Events;
use tokio::sync::watch;
use tokio::{signal, sync::mpsc, task, time, runtime};
use std::fs::{File,OpenOptions};
use std::cell::Cell;
use csv;

#[cfg(not(feature = "local"))]
use mothra::{Mothra, NetworkGlobals, NetworkMessage, Subscriber, TaskExecutor};
#[cfg(feature = "local")]
use mothra_local::{Mothra, NetworkGlobals, NetworkMessage, Subscriber, TaskExecutor};

#[derive(Serialize, Default, Clone)]
struct GossipRecord {
    index: u64,
    timestamp: String,
    message_id: String,
    peer_id: String,
    topic: String,
    message_size: usize,
    slot: u64,
    proposer_index: u64,
}
//SignedBeaconBlock
impl GossipRecord {
    pub fn new(
        index: u64,
        timestamp: String,
        message_id: String,
        peer_id: String,
        topic: String,
        data: Vec<u8>,
    ) -> Result<Self, String> {
        let mut decoder = Decoder::new();
        let mut decompressed_data: Vec<u8> = Vec::new();
        match decompress_len(&data) {
            Ok(n) if n > 1_048_576 => {
                return Err("ssz_snappy decoded data > GOSSIP_MAX_SIZE".into());
            }
            Ok(n) => decompressed_data.resize(n, 0),
            Err(e) => {
                return Err(format!("{}", e));
            }
        };
        let mut decoder = Decoder::new();
        let data = match decoder.decompress(&data, &mut decompressed_data) {
            Ok(n) => {
                decompressed_data.truncate(n);
                &decompressed_data
            }
            Err(e) => return Err(format!("{}", e)),
        };
        match SignedBeaconBlock::<MainnetEthSpec>::from_ssz_bytes(&data) {
            Ok(decoded_data) => Ok(GossipRecord {
                index,
                timestamp,
                message_id,
                peer_id,
                topic,
                message_size: data.len(),
                slot: decoded_data.message.slot.into(),
                proposer_index: decoded_data.message.proposer_index,
            }),
            Err(e) => return Err(format!("{:#?}", e)),
        }
    }
}

struct Client {
    num_records: Cell<u64>,
}

impl Client {
    pub fn new() -> Self {
        Client {
            num_records: Cell::new(0),
        }
    }

    fn write_file(&self, record: GossipRecord) {
        let mut wtr = match record.index {
            0 => {
                let file = OpenOptions::new()
                    .truncate(true)
                    .write(true)
                    .create(true)
                    .append(false)
                    .open("/Users/jonny/.imp/gossip.csv")
                    .unwrap();
                csv::WriterBuilder::new()
                    .has_headers(true)
                    .from_writer(file)
            }
            _ => {
                let file = OpenOptions::new()
                    .truncate(false)
                    .write(true)
                    .create(true)
                    .append(true)
                    .open("/Users/jonny/.imp/gossip.csv")
                    .unwrap();
                csv::WriterBuilder::new()
                    .has_headers(false)
                    .from_writer(file)
            }
        };
        let _ = wtr.serialize(&record);
        let _ = wtr.flush();
    }
}

fn pad_millis(millis: u32) -> Option<String> {
    let m = millis.to_string();
    match m.len() {
        3 => Some(m),
        2 => Some(format!("{}{}", "0", m)),
        1 => Some(format!("{}{}", "00", m)),
        0 => Some(String::from("000")), // this should never occur
        _ => None,
    }
}

impl Subscriber for Client {
    fn discovered_peer(&self, peer: String) {
        println!("Rust: discovered peer");
        println!("peer={:?}", peer);
    }

    fn receive_gossip(&self, message_id: String, peer_id: String, topic: String, data: Vec<u8>) {
        if topic.contains("beacon_block") {
            let timestamp = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
                Ok(n) => format!(
                    "{}.{}",
                    n.as_secs(),
                    pad_millis(n.subsec_millis()).expect("unexpected subsec_millis value")
                ),
                Err(_) => panic!("SystemTime before UNIX EPOCH!"),
            };
            match GossipRecord::new(
                self.num_records.get(),
                timestamp.clone(),
                message_id.clone(),
                peer_id.clone(),
                topic.clone(),
                data,
            ) {
                Ok(record) => {
                    self.write_file(record);
                    self.num_records.set(self.num_records.get() + 1);
                    println!("Rust: received gossip at {}", timestamp);
                    println!("message id={:?}", message_id);
                    println!("peer id={:?}\n", peer_id);
                }
                Err(e) => println!("Error:{}", e),
            }
        }
    }

    fn receive_rpc(&self, method: String, req_resp: u8, peer: String, data: Vec<u8>) {
        println!("Rust: received rpc");
        println!("method={:?}", method);
        println!("req_resp={:?}", req_resp);
        println!("peer={:?}", peer);
        println!("data={:?}", String::from_utf8_lossy(&data));
    }
}

// Holds variables needed to interacts with mothra
pub struct Adapter {
    network_globals: Arc<NetworkGlobals>,
    network_send: mpsc::UnboundedSender<NetworkMessage>,
    network_exit_signal: exit_future::Signal,
    enr_fork_id: Option<eth2::types::EnrForkId>,
    log: slog::Logger,
}

impl Adapter {
    pub fn new(
        runtime: &runtime::Runtime,
        client_name: String,
        platform: String,
        protocol_version: String,
        testnet_dir: Option<PathBuf>,
        arg_matches: &ArgMatches<'_>,
        log: slog::Logger,
    ) -> Self {
        let mut mothra_log = log.clone();

        // get mothra subcommand args matches
        let mothra_arg_matches = &arg_matches.subcommand_matches("mothra").unwrap();

        // if debug-level is set in subcommand
        // Note: bc mothra sets default value to info
        // mothra_arg_matches.is_present is always true
        // so we have to use mothra_arg_matches.occurrences_of
        if mothra_arg_matches.occurrences_of("debug-level") > 0 {
            let debug_level = mothra_arg_matches.value_of("debug-level").unwrap();
            // re-configure logging
            mothra_log = utils::config_logger(debug_level, false).new(o!("P2PAdapter" => "Mothra"));
        }

        // NOTE:  The reason the bootnode must be parsed form the CLI instead of using the Enr type
        // from mothra directly is bc Enr is defined in both Mothra and LH (which is a problem)
        let boot_nodes: Vec<String> = if mothra_arg_matches.is_present("boot-nodes") {
            let boot_enr_str = mothra_arg_matches.value_of("boot-nodes").unwrap();
            boot_enr_str
                .split(',')
                .map(|x| x.into())
                .collect::<Vec<String>>()
        } else {
            Default::default()
        };

        let mut config = Mothra::get_config(
            Some(client_name),
            Some(platform),
            Some(protocol_version),
            &mothra_arg_matches,
        );
        config.network_config.max_peers = 1000;
        config.network_config.propagation_percentage = Some(0);
        config.network_config.gs_config.mesh_n_high = 76;
        config.network_config.gs_config.mesh_n_low = 25;
        config.network_config.gs_config.mesh_n = 50;
        config.network_config.gs_config.gossip_lazy = 0;

        // TODO
        // Option: Learn fork_id from supplied cli arg directly

        // Option: Learn fork_id from bootnode
        let (enr_fork_id, enr_fork_id_bytes) = match get_fork_id_from_string(boot_nodes[0].clone())
        {
            Some(enr_fork_id) => {
                info!(log,"Fork-Digest: {}",hex::encode(enr_fork_id.fork_digest));
                // configure gossip topics
                config.network_config.topics = create_topic_ids(enr_fork_id.clone());
                (Some(enr_fork_id.clone()), enr_fork_id.as_ssz_bytes())
            }
            _ => {
                // Option: Learn fork_id from supplied testnet_dir
                match get_fork_id_from_dir(testnet_dir) {
                    Some(enr_fork_id) => {
                        // configure gossip topics
                        config.network_config.topics = create_topic_ids(enr_fork_id.clone());
                        (Some(enr_fork_id.clone()), enr_fork_id.as_ssz_bytes())
                    }
                    _ => (None, [0u8, 32].to_vec()),
                }
            }
        };
        let client = Box::new(Client::new()) as Box<dyn Subscriber + Send>;
        let (network_exit_signal, exit) = exit_future::signal();
        let task_executor = TaskExecutor::new(
            runtime.handle().clone(),
            exit,
            log.new(o!("Imp" => "TaskExecutor")),
        );
        // instantiate mothra
        let (network_globals, network_send) = runtime.handle().block_on(async { Mothra::new(config, enr_fork_id_bytes, &task_executor, client, log.clone()) }).unwrap();

        Adapter {
            network_globals,
            network_send,
            network_exit_signal,
            enr_fork_id,
            log,
        }
    }

    pub fn close(self) -> Result<(), ()> {
        self.network_exit_signal.fire()
    }
}
