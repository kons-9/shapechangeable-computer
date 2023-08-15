# Shape changeable display

## WIP

## test of hardware
You can use example/\*.rs as test of hardware.  
But be careful using efuse-write function. it is not recoverable after you execute it.(you can generate using generate.zsh)
<!-- But you should not use efuse.rs before  -->

# documents
## Flit
Flit consists of 64 bits. Flit type is only four types, basic three types, head, body and tail, and irregular one type, nope flit.
A header of flit(packet) that need only head flit begin with H.
An ack flit (, which header is HAck) is generally sent by the system when it receive a flit.
Nope flit will used for timing adjustment.  

### HeadFlit
HeadFlit id is 0.

FlitType(2) | LengthOfFlit(6) | Header(8) | SourceId(16) | DestinationId(16) | PacketId(8) | Checksum(8)
:--:|:--:|:--:|:--:|:--:|:--:|:--:
### Body and TailFlit

FlitType(2) | FlitId(6) | Message(48) | Checksum(8)
:--:|:--:|:--:|:--:
### NopeFlit

 FlitType(2) | (undefined)(62) 
:--:|:--:

todo: flitId and length of flit is mod 6bit.

## Packet
General packet, which means the packet have body and tail flit, has packetid, global sourceId, global destinationId and checksum like below.

 packetId(8) | checksum(8) | globalDestinationId(16) | globalDestinationId(16) | data(...)
:--:|:--:|:--:|:--:|:--:

This means first body flit doesn't have any messages.

## Network Protocol
Network Protocol must implement `network::protocol::Protocol` trait.

## About Each Process and detail of Packets
This section explains processes and their packets.
Note that explanations of these packets is data region of Packet.

### General case
If you want to use the library with other application, you use these packets.

#### General ack
##### Explanation
If you want to make ack packet in system such as TCP, you can use this packet.
##### Implementation
Only head flit. 
Header is `GeneralAck`, not `HAck`, which is used for the system flit.

#### General data
##### Explanation
This library is not process this packet.
So, you should reshape it into any form you want.
##### Implementation
Header is `Data`.

#### Error
##### Explanation
Flit error is mainly processed by the library, but packet error isn't.
So, you should handle with this packet.
This error may occur in checksum process of packet. 
So, you should resend packet, but it's optional.
##### Implementation
Header is `Error`.

### Making local network
These packets are used for making local network by system.
The process is this:
1. the system sent check connection packet periodically, and then, receive check connection.
2. If the system receive check connection packet, then sent request confirmed coordinate packet.
3. If the system receive sufficient number of confirm coordinate packets, the system can analyse coordinate.

#### Check connection
##### Explanation
In this system, because unit of shapechangeable display is 4 nodes, these nodes connect each others. But, the unit should connect other units. This packet is used then firstly. 
##### Implementation
Only head flit. This packet is broadcast but processed only in other local networks.
Header is `HCheckConnection`.

#### Request confirmed coordinate
##### Explanation
Due to get global coordinate, you should access other nodes' coordinate.
##### Implementation
Only head flit. This packet is sent by broadcast.
Header is `HRequestConfirmedCoordinate`

#### Reply for request confirmed coordinate
##### Explanation
this packet is reply for request confirmed coordinate packet.

##### Implementation
When the system receive this packet, system will check the global source address of packet. 
If it is sent from a node which is in the same local network and is not confirmed, the packet has information of nodes which is next to the node not in local network. Otherwise, the packet is only sent by confirmed node.

Data form is like this:

is confirmed(8) | id(16) | x(16) | y(16) | id(16) | ...
:--:|:--:|:--:|:--:|:--:|:--:

Node that received this packet is in the same node as local 
This packet is sent by broadcast.
Header is `ConfirmCoordinate`

### Joining global network
Todo: These packets are used for making global network by system.

#### Request join network
##### Explanation
##### Implementation

#### Reply for request join network
##### Explanation
##### Implementation

## Mac address
The software defines original mac address. This address is contained in block3 efuse register[7], which size is 32bit.
You can use `write_efuse_generator.zsh`.

unique node id(29) | local location(2) | is root(1)
:--:|:--:|:--:

# Note
* a requirement of packet that has only headflit is that
** this data doesn't need to have a address that specify original source id, or this data is sent to just next node and this information is included in header.

# todo
I put `todo:` in source file in detail. Others are below.
* Handle uart interruption
* Handle uart buffer overflow 
* Use flit buffer
