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

pub fn get_fork_digest(testnet_dir: Option<PathBuf>) -> [u8; 4] {
    // TODO: need some warnings and error handling
    let mut fork_digest = [0; 4];
    if !testnet_dir.is_none() && testnet_dir.clone().unwrap().exists() {
        let config = loadTestnetConfig::<MainnetEthSpec>(testnet_dir.unwrap());
        let state = config.genesis_state.unwrap();
        let enr_fork_id = get_enr_fork_id(state.slot, state.genesis_validators_root);
        fork_digest = enr_fork_id.fork_digest;
    } else {
        println!("testnet_dir: {:?} doesn't exist", testnet_dir);
    }

    fork_digest
}

pub fn create_topic_ids(fork_digest: [u8; 4]) -> Vec<String> {
    let network_config = NetworkConfig::default();
    let topic_kinds = network_config.topics; //type GossipKind
    let mut topic_ids: Vec<String> = vec![];
    for kind in topic_kinds {
        let topic_id = GossipTopic::new(kind, GossipEncoding::default(), fork_digest);
        topic_ids.push(topic_id.into());
    }
    topic_ids
}

pub fn get_gossip_topic_id(kind: GossipKind, fork_digest: [u8; 4]) -> String {
    GossipTopic::new(kind, GossipEncoding::default(), fork_digest).into()
}
