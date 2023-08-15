use super::flit::Flit;
use super::packet::PacketId;
use std::collections::HashMap;
use std::collections::VecDeque;

pub struct FlitBuffer {
    flits: HashMap<PacketId, VecDeque<Flit>>,
    max_size: usize,
}
