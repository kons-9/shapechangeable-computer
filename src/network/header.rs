use num_enum::TryFromPrimitive;

/// todo) now we use only data and ack header
#[derive(TryFromPrimitive, Eq, PartialEq, Debug, Clone, Copy)]
#[repr(u8)]
pub enum Header {
    // BroadCast headers
    GetNeighborId,
    RequestConfirmedCoordinate,

    // for making localnet
    /// LocalnetId + NodeId
    ShareNeighborCoordinate,
    ConfirmCoordinate,
    SendNodeId,

    // for making tree
    SendParentId,
    ReceiveParentId,
    SendChildId,
    ReceiveChildId,

    // normal data
    Data,

    // for reply
    Ack,

    // for error
    DataError,
    EtcError,
}

impl Header {
    // todo) consider whether to use body and tail flits
    pub fn is_only_head(&self) -> bool {
        match self {
            Header::Data
            | Header::SendNodeId
            | Header::ShareNeighborCoordinate
            | Header::ConfirmCoordinate
            | Header::SendParentId
            | Header::ReceiveParentId
            | Header::SendChildId
            | Header::ReceiveChildId
            | Header::DataError
            | Header::EtcError => false,
            Header::Ack | Header::GetNeighborId => true,
        }
    }
}
