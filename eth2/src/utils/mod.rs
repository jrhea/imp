use crate::config::Eth2Config;
use crate::libp2p::types::{GossipEncoding, GossipKind, GossipTopic};
use crate::libp2p::NetworkConfig;
use crate::testnet::config::Eth2TestnetConfig;
use crate::types::{ChainSpec, EnrForkId, EthSpec, Hash256, MainnetEthSpec, Slot};
use std::path::PathBuf;

pub fn loadTestnetConfig<E: EthSpec>(testnet_dir: PathBuf) -> Eth2TestnetConfig<E> {
    Eth2TestnetConfig::load(testnet_dir.clone()).unwrap()
}

pub fn getEth2Config() -> Eth2Config {
    Eth2Config::mainnet()
}

pub fn getChainSpec() -> ChainSpec {
    ChainSpec::mainnet()
}

pub fn get_enr_fork_id(slot: Slot, genesis_validators_root: Hash256) -> EnrForkId {
    let spec = getChainSpec();
    spec.enr_fork_id(slot, genesis_validators_root)
}

pub fn get_genesis_enr_fork_id(testnet_dir: Option<PathBuf>) -> EnrForkId {
    if !testnet_dir.is_none() && testnet_dir.clone().unwrap().exists() {
        let config = loadTestnetConfig::<MainnetEthSpec>(testnet_dir.unwrap());
        let state = config.genesis_state.unwrap();
        get_enr_fork_id(state.slot, state.genesis_validators_root)
    } else {
        println!("testnet_dir: {:?} doesn't exist", testnet_dir);
        EnrForkId {
            fork_digest: [0; 4],
            next_fork_version: [0; 4], //genesis_fork_version,
            next_fork_epoch: u64::max_value().into(), //far_future_epoch,
        }
    }
}

pub fn create_topic_ids(enr_fork_id: EnrForkId) -> Vec<String> {
    let network_config = NetworkConfig::default();
    let topic_kinds = network_config.topics; //type GossipKind
    let mut topic_ids: Vec<String> = vec![];
    for kind in topic_kinds {
        let topic_id = GossipTopic::new(kind, GossipEncoding::default(), enr_fork_id.fork_digest);
        topic_ids.push(topic_id.into());
    }
    topic_ids
}

pub fn get_gossip_topic_id(kind: GossipKind, enr_fork_id: EnrForkId) -> String {
    GossipTopic::new(kind, GossipEncoding::default(), enr_fork_id.fork_digest).into()
}
