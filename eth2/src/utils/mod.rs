use crate::config::Eth2Config;
use crate::libp2p::discovery::enr_ext::{CombinedKeyExt, EnrExt};
use crate::libp2p::types::{EnrBitfield, GossipEncoding, GossipKind, GossipTopic};
use discv5::enr::{CombinedKey, Enr};
//pub use enr_ext::{CombinedKeyExt, EnrExt};
use crate::libp2p::NetworkConfig;
use crate::ssz::types::BitVector;
use crate::ssz::{Decode, Encode};
use crate::testnet::config::Eth2TestnetConfig;
use crate::types::{ChainSpec, EnrForkId, EthSpec, Hash256, MainnetEthSpec, Slot};

use std::path::PathBuf;

pub fn load_testnet_config<E: EthSpec>(testnet_dir: PathBuf) -> Eth2TestnetConfig<E> {
    Eth2TestnetConfig::load(testnet_dir).unwrap()
}

pub fn get_eth2_config() -> Eth2Config {
    Eth2Config::mainnet()
}

pub fn get_chain_spec() -> ChainSpec {
    ChainSpec::mainnet()
}

pub fn get_default_fork_id() -> EnrForkId {
    EnrForkId {
        fork_digest: [0; 4],
        next_fork_version: [0; 4],                //genesis_fork_version,
        next_fork_epoch: u64::max_value().into(), //far_future_epoch,
    }
}

pub fn get_fork_id(slot: Slot, genesis_validators_root: Hash256) -> EnrForkId {
    let spec = get_chain_spec();
    spec.enr_fork_id(slot, genesis_validators_root)
}

pub fn get_fork_id_from_dir(dir: Option<PathBuf>) -> Option<EnrForkId> {
    if let Some(value) = dir {
        let config = load_testnet_config::<MainnetEthSpec>(value);
        let state = config.genesis_state.unwrap();
        Some(get_fork_id(state.slot, state.genesis_validators_root))
    } else {
        None
    }
}

pub fn get_fork_id_from_enr(enr: &Enr<CombinedKey>) -> Option<EnrForkId> {
    match enr.get("eth2") {
        Some(enr_fork_id) => match EnrForkId::from_ssz_bytes(enr_fork_id) {
            Ok(enr_fork_id) => Some(enr_fork_id),
            Err(_e) => None,
        },
        None => None,
    }
}

pub fn get_attnets_from_enr(enr: &Enr<CombinedKey>) -> Vec<u64> {
    let mut attnets = vec![];

    if let Ok(bitfield) = get_bitfield_from_enr(enr) {
        if bitfield.len() > 0 {
            let subnet_count = get_chain_spec().attestation_subnet_count as usize;
            for i in 0..=subnet_count {
                match bitfield.get(i) {
                    Ok(true) => attnets.push(i as u64),
                    _ => (),
                }
            }
        }
    }
    return attnets;
}

pub fn get_bitfield_from_enr(
    enr: &Enr<CombinedKey>,
) -> Result<EnrBitfield<MainnetEthSpec>, &'static str> {
    let bitfield_bytes = enr
        .get("attnets")
        .ok_or_else(|| "ENR bitfield non-existent")?;

    BitVector::<<MainnetEthSpec as EthSpec>::SubnetBitfieldLength>::from_ssz_bytes(bitfield_bytes)
        .map_err(|_| "Could not decode the ENR SSZ bitfield")
}

pub fn get_enr_from_string(enr: String) -> Option<Enr<CombinedKey>> {
    match enr.parse::<Enr<CombinedKey>>() {
        Ok(enr) => Some(enr),
        Err(_e) => None,
    }
}

pub fn get_fork_id_from_string(enr: String) -> Option<EnrForkId> {
    match enr.parse::<Enr<CombinedKey>>() {
        Ok(enr) => get_fork_id_from_enr(&enr),
        Err(_e) => None,
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
