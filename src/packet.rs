pub type Id = usize;
pub type Coordinate = (u32, u32);

enum Header {
    // for making localnet
    /// LocalnetId + NodeId
    SendLocalnetId,
    SendNodeId,
    GetConnectedNeighborCoordinate,
    ShareNeighborCoordinate,
    ConfirmCoordinate,

    // for making tree
    SendParentId,
    ReceiveParentId,
    SendChildId,
    ReceiveChildId,
    
    // for reply
    Ack,

    // for error
    HeaderError,
    FilterError,
    FromError,
    ToError,
    CheckSumError,
    EtcError,
}

enum From {
    Localnet(Id),
    Node(Id),
    Coordinate(Coordinate)
}

enum To {
    Localnet(Id),
    Node(Id),
    Coordinate(Coordinate)
}

struct Message {
    header: Header,
    from: From,
    to: To,
    data: Vec<u8>,
    checksum: u8,
}
