use libp2p::discv5::{enr, Discv5, Discv5Config, Discv5Event};
use libp2p::identity;
use std::net::{Ipv4Addr, SocketAddr};
use std::convert::TryInto;
use std::collections::HashMap;
use eth2::types::EnrForkId;
use eth2::libp2p::discovery::ETH2_ENR_KEY;
