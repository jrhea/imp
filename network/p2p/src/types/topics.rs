use eth2_libp2p::types::{GossipEncoding, GossipKind, GossipTopic};

pub fn create_topics(fork_digest: [u8; 4]) -> Vec<String> {
    let beacon_block_topic: String = GossipTopic::new(
        GossipKind::BeaconBlock,
        GossipEncoding::default(),
        fork_digest,
    )
    .into();
    let beacon_aggregate_and_proof_topic: String = GossipTopic::new(
        GossipKind::BeaconAggregateAndProof,
        GossipEncoding::default(),
        fork_digest,
    )
    .into();
    let voluntary_exit_topic: String = GossipTopic::new(
        GossipKind::VoluntaryExit,
        GossipEncoding::default(),
        fork_digest,
    )
    .into();
    let proposer_slashing_topic: String = GossipTopic::new(
        GossipKind::ProposerSlashing,
        GossipEncoding::default(),
        fork_digest,
    )
    .into();
    let attester_slashing_topic: String = GossipTopic::new(
        GossipKind::AttesterSlashing,
        GossipEncoding::default(),
        fork_digest,
    )
    .into();

    vec![
        beacon_block_topic.clone(),
        beacon_aggregate_and_proof_topic.clone(),
        voluntary_exit_topic.clone(),
        proposer_slashing_topic.clone(),
        attester_slashing_topic.clone(),
    ]
}
