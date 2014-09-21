rust-dht
========

[![Build
Status](https://travis-ci.org/Divius/rust-dht.svg)](https://travis-ci.org/Divius/rust-dht)

A work-in-progress of
[Kademlia](http://pdos.csail.mit.edu/~petar/papers/maymounkov-kademlia-lncs.pdf)-based
DHT in Rust language.

Goals
-----

The goal of this project is to provide flexible implementation of DHT
for different kind of Rust applications. There will be 3 loosely coupled
parts:

1. DHT neighborhood table implementation, will be represented by
   `GenericNodeTable` trait and `kademlia::NodeTable` implementation.
2. RPC implementation, will be represented by `GenericRpc` trait,
   exact implementation to by defined, likely some JSON-over-UDP.
3. Generic struct `Service<TNodeTable: GenericNodeTable, TRpc: GenericRpc>`
   that will connect previous two.

Status
------

Currently implemented or have a good progress:

* `Node` struct: endpoint address + ID, representing this Node in the system.

* `kademlia::KBucket`: k-bucket implementation, without replacing of nodes.
   Goal is to have periodic task taking away all last elements, pinging them
   and inserting back, if they're alive. Thus will avoid too much of
   interdependency between node table and RPC.

* `kademlia::NodeTable`: node table with k-buckets.
