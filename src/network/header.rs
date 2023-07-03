use num_enum::TryFromPrimitive;

#[derive(TryFromPrimitive, PartialEq, Debug)]
#[repr(u8)]
pub enum Header {
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

impl Header {
    // todo) consider whether to use body and tail flits
    pub fn is_only_head(&self) -> bool {
        match self {
            Header::SendLocalnetId
            | Header::SendNodeId
            | Header::GetConnectedNeighborCoordinate
            | Header::ShareNeighborCoordinate
            | Header::ConfirmCoordinate
            | Header::SendParentId
            | Header::ReceiveParentId
            | Header::SendChildId
            | Header::ReceiveChildId
            | Header::Ack => false,
            _ => true,
        }
    }
}
