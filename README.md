# Shape changeable display

## WIP

## test of hardware
You can use example/\*.rs as test of hardware.  
But be careful using efuse-write function. it is not recoverable after you execute it.(you can generate using generate.zsh)
<!-- But you should not use efuse.rs before  -->

# documents
## Flit
Flit consists of 64 bits. Flit type is only four types, basic three types, head, body and tail, and errgular one type, nope flit.
Nope flit will used for timing adjustment.
* HeadFlit : 
|:--:|:--:|:--:|:--:|:--:|:--:|:--:|
| FlitType(2) | LengthOfFlit(6) | Header(8) | SourceId(16) | DestinationId(16) | PacketId(8) | Checksum(8) |
* Body and TailFlit : 
|FlitType(2) | FlitId(6) | Message(48) | Checksum(8)|
* NopeFlit : 
| FlitType(2) | (undefined)(62) |

A header of flit(packet) that need only head flit begin with H.

HeadFlit id is 0.
todo: flitId and length of flit is mod 6bit.

## Packet
General packet, which means the packet have body and tail flit, has packetid, global sourceId, global destinationId and checksum like below.
* [ packetId(8) | checksum(8) | globalDestinationId(16) | globalDestinationId(16) | data(...)]
This means first body flit doesn't have any messages.

## Network Protcol
Network Protocol must implement networkProtocol trait.

## About Each Process and detail of Packets
This section explains processes and their packets.
Note that explanations of these packets is data region of Packet.

### General case
If you want to use this library with other application, you use this packets.

#### General ack
##### Explanation
##### Implementation
Only head flit. 
Header is HAck.

#### General data
##### Explanation
This library is not process this packet.
##### Implementation

#### Error
##### Explanation
Flit error is mainly processed by this library, but packet error isn't.
So, you should handle with this packet.
##### Implementation

### making local network

#### Check connection
##### Explanation
##### Implementation
Only head flit. Nothing special.
#### Request confirmed coordinate
##### Explanation
##### Implementation
Only head flit. Nothing special.
#### Reply for request confirmed coordinate

### joining global network

#### request join network
##### Explanation
##### Implementation

#### reply for request join network
##### Explanation
##### Implementation

# Note
* a requirement of packet that has only headflit is that
** this data doesn't need to have a address that specify original source id, or this data is sent to just next node and this information is included in header.

# todo
I put todo: in source file in detail. others below.
* handle uart interruption
* handle uart buffer overflow 
* use flit buffer
