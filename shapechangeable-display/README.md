# Shape changeable display

## WIP

## test of hardware
You can use example/\*.rs as test of hardware.  
But be careful using efuse-write function. it is not recoverable after you execute it.(you can generate a file that write efuse,using generate.zsh)
<!-- But you should not use efuse.rs before  -->

# Documents
## definitions of word
* node: a device that has microcomputer.
* unit: a group of 4 node.
* local network: network in unit
* global network: network in the whole system

## Flit
Flit consists of 64 bits. Flit type is only 4 types:
* basic three types, head, body and tail
* irregular one type, nope flit.

A header of flit (packet) that need only head flit begins with H.
An ack flit, which header is HAck, is generally sent by the system when it receives a flit.  
Nope flit will be used for timing adjustment(WIP).  

### NopeFlit
NopeFlit's flittype is `00`.

 FlitType(2) | (undefined)(62) 
:--:|:--:

### HeadFlit
HeadFlit's flittype is `01`.

FlitType(2) | LengthOfFlit(6) | Header(8) | SourceId(16) | DestinationId(16) | PacketId(8) | Checksum(8)
:--:|:--:|:--:|:--:|:--:|:--:|:--:
### Body and TailFlit
BodyFlit's flittype is `10`.
TailFlit's flittype is `11`.

FlitType(2) | FlitId(6) | Message(48) | Checksum(8)
:--:|:--:|:--:|:--:

todo: flitId and length of flit is mod 6bit.

## Packet
General packet, which means the packet has body and tail flit, has packetid, global sourceId, global destinationId and checksum like below.

 packetId(8) | checksum(8) | globalDestinationId(16) | globalDestinationId(16) | data(...)
:--:|:--:|:--:|:--:|:--:

This means first body flit doesn't have any messages.
The data section must finish with `0bFF0*`. In other words, the last `FF` represents eof.

## Network Protocol
Network Protocol must implement `network::protocol::Protocol` trait(WIP).

## About Each Process and details of Packets
This section explains processes and their packets.

### 1. General case

If you want to use the crate with other application, you will use these packets.

#### 1.1 General ack
#### Explanation
If you want to make ack packet in node such as TCP, you can use this packet.
#### Implementation
Only head flit. 
Header is `GeneralAck`, not `HAck`, which is used for the manual flit.

#### 1.2 General data
#### Explanation
This crate is not process this packet.
So, you should reshape it into any form you want.
#### Implementation
Header is `Data`.

#### 1.3 Error
#### Explanation
Flit error is mainly processed by the crate, but this packet error isn't.
This error may occur in checksum process of packet. 
You can choose whether you resend packet or not manually.
#### Implementation
Header is `Error`.

### 2. Making local network
These packets are used for making local network.

The process is this:  
**1. if this node is not confirmed**
1. This node sends request confirmed coordinate packet(2.2) periodically.
2. If the node receives reply for request confirmed coordinate packet(2.3), then it stores the neighbor information in neighbor confirmed buffer.
3. If the node receives request confirned coordinate packet, then sends the information of neighbor confirmed nodes.
2. If the node receives sufficient number of confirm coordinate packets, it will calculate coordinate.

**2. if this node has already confirmed**
1. If the node receives request confirmed coordinate packet(2.2), then sends request confirmed coordinate packet.

#### 2.1 Request confirmed coordinate
#### Explanation
Due to getting global coordinate, you should access other unit's coordinate.
#### Implementation
Only head flit. This packet is sent by broadcast.
Header is `HRequestConfirmedCoordinate`

#### 2.2 Reply for request confirmed coordinate
#### Explanation
This packet is reply for request confirmed coordinate packet.

#### Implementation
When the node receive this packet, it will check the global source address of packet. 
If it is sent from a node which is in the same local network, the packet has information of nodes which is next to the node in not same unit. 
Otherwise, the packet is only sent by confirmed node, and has information of only its coordinate.

Data form is like this:

is confirmed(8) | id(16) | x(16) | y(16) | id(16) | ...
:--:|:--:|:--:|:--:|:--:|:--:

This packet is sent by broadcast.
Header is `ConfirmCoordinate`

These confirmed coordinate information are stored in `neighbor_confirmed`

### 3. Joining global network
Todo: These packets are used for making global network by system.

#### 3.1 Request join network
#### Explanation
WIP
#### Implementation
WIP

#### 3.2 Reply for request join network
#### Explanation
WIP
#### Implementation
WIP

#### 3.3 Check connection
#### Explanation
WIP

#### Implementation
Only head flit. This packet is broadcast but processed only in the other units.
Header is `HCheckConnection`.

## Mac address
The software defines original mac address. This address is contained in block3 efuse register[7], which size is 32bit.
You can use `write_efuse_generator.zsh`.

unique node id(29) | local location(2) | is root(1)
:--:|:--:|:--:

# todo
I put `todo:` in source file in detail. 
Others:
* Handle uart interruption
* Handle uart buffer overflow 
* Use flit buffer
