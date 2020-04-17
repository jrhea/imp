use crate::types::{MainnetEthSpec, ChainSpec, EnrForkId, Hash256};
use crate::config::Eth2Config;
use crate::libp2p::types::{GossipEncoding, GossipTopic, GossipKind};
use crate::libp2p::NetworkConfig;


pub fn getEth2Config() -> Eth2Config {
    Eth2Config::mainnet()
}

pub fn getChainSpec() -> ChainSpec {
    ChainSpec::mainnet()
}

pub fn get_enr_fork_id() -> EnrForkId {
    let spec = getChainSpec();
    let genesis_validators_root = Hash256::zero();
    spec.enr_fork_id(spec.genesis_slot, genesis_validators_root)
}

pub fn create_topic_ids() -> Vec<String> {
    let network_config = NetworkConfig::default();
    let topic_kinds = network_config.topics;  //type GossipKind
    let enr_fork_id =  get_enr_fork_id();
    let mut topic_ids: Vec<String> = vec![];
    for kind in topic_kinds {
        let topic_id = GossipTopic::new(
            kind,
            GossipEncoding::default(),
            enr_fork_id.fork_digest,
        );
        topic_ids.push(topic_id.into());
    }
    topic_ids
}

pub fn get_gossip_topic_id(kind: GossipKind) -> String {
    let enr_fork_id =  get_enr_fork_id();
    GossipTopic::new(
        kind,
        GossipEncoding::default(),
        enr_fork_id.fork_digest,
    ).into()
}