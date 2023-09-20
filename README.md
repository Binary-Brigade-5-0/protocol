# PROTOCOL

This is the base implementation of the _Decentralized data distribution_ protocol.
The repository itself hosts a basic implementations of the server/distributor
side of the protocol.

## What is it?

The protocol relies on the connected clients to share data among themselves, and
the server itself just acts as a centralized broker of sorts for these clients
to fluently communicate. Unlike many other implementations of the peer to peer
data sharing implementations like **torrents**, in our implementation the **server
tends to do the heavy lifting**, thus allowing the clients to be as flexible as
they want to, i.e, some clients may just act as a repository for data sharing,
just like dedicated seeds in torrents.


## How does it work?

The current implementation of the server tends to allow bi-directional
communication via the use of websockets (due to team decisions). It might not be
the best medium to allow performant full duplex communication between systems,
but is much more _ergonomic (apparently)_. Data passing works by sending protocol
agnostic JSON based messages(soon to be moved to `protobuf(s)`).


### Message passing

At present there are 6 different kinds of messages that can passed around
through the network, they are as follows.

```
struct PostHeader { /* ... */ }
struct Post       { /* ... */ }

enum MessageKind {
    Query     (String),
    Connected (Uuid),

    Error    { criminal: Uuid, error: String },
    Response { target: Uuid, posts: Vec<PostHeader> },
    Post     { target: Uuid, post: Post },
    Get      { target: Uuid, id: Uuid },
}
```

Of the fore-mentioned message kinds `Connected` and `Error` are reserved for the
server/broker. No general purpose client can send them.

A general message is structured in the following method:

```jsonc
{
    "sender" : "sender-uuid",
    "time"   : "system time",
    "method" : "message kind",
    "body"   : "message body",
}
```


### What the messages mean

To explain how the actual protocol works we present a sequence diagram

```mermaid
sequenceDiagram

actor C1
participant S as Server
participant B as Broadcast
actor C2

C1 -->> S: GET ws://server:port/proto/v1/ws
S ->> C1: { ..., connected, client_id }

C2 -->> S: GET ws://server:port/proto/v2/ws
S->> C2: { ..., connected, client_id }

C1 ->> C1: updates registry to post content
C2 ->> S: { ..., query, post_title }
S --) B: { sender: C2, ..., query, post_title }
B --) C1: repeat broadcast
B --) C2: repeat broadcast

C2 --) C2: skip broadcast since mesg.sender == self.id

alt if C1 has content matching required query
	C1 ->> S: { ..., response, { id, posted, title } }
end

loop collect message until pre-decided timeout
	S --> S: collect responses for C2
end

S ->> C2: { ..., response, vec[{ id, posted, title, from }] }
C2 ->> S: { ..., get, { target, post_id } }

note right of S: assuming target from previous get request is C1
S ->> C1: repeat get request
C1 ->> S: { ..., post, { ...post_header, content } }
S ->> C2: { sender: C1, ..., post, { ...post_header, content } }

C2 --) C2: update registry to contain post as long as it exists

C1 --x S: Disconnected
C2 --x S: Disconnected
```


# LICENSE

This project is licensed under the GNU GPL V3 License.


# NOTE

At the time this readme was written, the protocol itself is unfinished and needs
to be fleshed out significantly. The server implementation is supposed to be a
demonstration of how a server might be implemented following the protocol
guidlines.
