rust-dht
========

[![Build
Status](https://travis-ci.org/Divius/rust-dht.svg)](https://travis-ci.org/Divius/rust-dht)
[Online Documentation](http://www.rust-ci.org/Divius/rust-dht/doc/dht/)

A work-in-progress of
[Kademlia](http://pdos.csail.mit.edu/~petar/papers/maymounkov-kademlia-lncs.pdf)-based
DHT in Rust language, see documentation for details.

Status
------

Currently implemented or have a good progress:

* `Node` struct: endpoint address + ID, representing this Node in the system.

* `kademlia::KBucket`: k-bucket implementation, without replacing of nodes.
   Goal is to have periodic task taking away all last elements, pinging them
   and inserting back, if they're alive. Thus will avoid too much of
   interdependency between node table and RPC.

* `kademlia::NodeTable`: node table with k-buckets.

* `krpc::Package`: network package format for KRPC (BitTorrent DHT RPC).

Just started or stubs:

* `Service` struct glueing together RPC and node table.

* `krpc::KRpc`: KRPC implementation.
