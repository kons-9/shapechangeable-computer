use num_enum::TryFromPrimitive;

/// todo) now we use only data and ack header
/// initial of a header that use only head flit is H
#[derive(TryFromPrimitive, Eq, PartialEq, Debug, Clone, Copy)]
#[repr(u8)]
pub enum Header {
    // ////////////////////////////////
    // general case
    // ////////////////////////////////

    // general data
    Data,
    // general ack
    GeneralAck,
    // general error
    Error,

    // ////////////////////////////////
    // Making local network
    // ////////////////////////////////
    HCheckConnection,
    HRequestConfirmedCoordinate,
    ConfirmCoordinate,

    // ////////////////////////////////
    // Making general network
    // ////////////////////////////////
    SendParentId,
    ReceiveParentId,
    SendChildId,
    ReceiveChildId,

    // System ack
    HAck,
}

impl Header {
    // todo) consider whether to use body and tail flits
    pub fn is_only_head(&self) -> bool {
        match self {
            Header::Data
            | Header::GeneralAck
            | Header::ConfirmCoordinate
            | Header::SendParentId
            | Header::ReceiveParentId
            | Header::SendChildId
            | Header::ReceiveChildId
            | Header::Error => false,
            Header::HAck | Header::HRequestConfirmedCoordinate | Header::HCheckConnection => true,
        }
    }
    pub fn is_require_ack(&self) -> bool {
        !self.is_only_head()
    }
}
