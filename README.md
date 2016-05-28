rust-dht
========

[![Build
Status](https://travis-ci.org/dtantsur/rust-dht.svg?branch=master)](https://travis-ci.org/dtantsur/rust-dht)
[Online Documentation](http://dtantsur.net/rust/dht)

A work-in-progress of
[Kademlia](http://pdos.csail.mit.edu/~petar/papers/maymounkov-kademlia-lncs.pdf)-based
DHT in Rust language, see documentation for details.

Build
-----

Use [cargo](http://crates.io) tool to build and test.

Status
------

Currently implemented or have a good progress:

* `Node` struct: endpoint address + ID, representing this Node in the system.

* `knodetable::KBucket`: k-bucket implementation.

* `knodetable::KNodeTable`: node table with k-buckets.
