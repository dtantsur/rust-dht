// Copyright 2016 Dmitry "Divius" Tantsur <divius.inside@gmail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Generic protocol bits for implementing custom protocols.

use super::{GenericId, Node};


/// Payload in the request.
pub enum RequestPayload<TId, TValue> {
    Ping,
    FindNode(TId),
    FindValue(TId),
    Store(TId, TValue)
}

/// Request structure.
pub struct Request<TId, TAddr, TValue> {
    pub caller: Node<TId, TAddr>,
    pub request_id: TId,
    pub payload: RequestPayload<TId, TValue>
}

/// Payload in the response.
pub enum ResponsePayload<TId, TAddr, TValue> {
    NodesFound(Vec<Node<TId, TAddr>>),
    ValueFound(TValue),
    NoResult
}

/// Response structure.
pub struct Response<TId, TAddr, TValue> {
    pub request: Request<TId, TAddr, TValue>,
    pub responder: Node<TId, TAddr>,
    pub payload: ResponsePayload<TId, TAddr, TValue>
}

/// Trait for a protocol implementation.
pub trait Protocol : Send {
    /// Value type.
    type Id: GenericId;
    type Addr: Send + Sync;
    type Value: Send + Sync;
    /// Parse request from binary data.
    fn parse_request(&self, data: &[u8]) -> Request<Self::Id, Self::Addr, Self::Value>;
    /// Format response to binary data.
    fn format_response(&self, Response<Self::Id, Self::Addr, Self::Value>) -> Vec<u8>;
}
