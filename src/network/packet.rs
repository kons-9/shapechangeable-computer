pub type Id = u8;
pub type Coordinate = (u8, u8);

enum From {
    Localnet(Id),
    Node(Id),
    Coordinate(Coordinate),
}

enum To {
    Localnet(Id),
    Node(Id),
    Coordinate(Coordinate),
    Broadcast,
}

struct Packet {
    header: Header,
    from: From,
    to: To,
    data: Vec<u8>,
    checksum: u8,
}
