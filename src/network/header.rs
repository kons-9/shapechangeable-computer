use num_enum::TryFromPrimitive;

/// todo) now we use only data and ack header
/// initial of a header that use only head flit is H
#[derive(TryFromPrimitive, Eq, PartialEq, Debug, Clone, Copy)]
#[repr(u8)]
pub enum Header {
    // BroadCast headers
    HRequestConfirmedCoordinate,

    // for making localnet
    /// LocalnetId + NodeId
    ShareNodeIdAndNeighborCoordinate,
    ConfirmCoordinate,

    // for check connection
    HCheckConnection,

    // for making tree
    SendParentId,
    ReceiveParentId,
    SendChildId,
    ReceiveChildId,

    // normal data
    Data,

    // for reply
    HAck,

    // for error
    Error,
}

impl Header {
    // todo) consider whether to use body and tail flits
    pub fn is_only_head(&self) -> bool {
        match self {
            Header::Data
            | Header::ConfirmCoordinate
            | Header::SendParentId
            | Header::ReceiveParentId
            | Header::SendChildId
            | Header::ReceiveChildId
            | Header::ShareNodeIdAndNeighborCoordinate
            | Header::Error => false,
            Header::HAck | Header::HRequestConfirmedCoordinate | Header::HCheckConnection => true,
        }
    }
    pub fn is_require_ack(&self) -> bool {
        !self.is_only_head()
    }
}
