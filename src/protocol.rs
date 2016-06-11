// Copyright 2016 Dmitry "Divius" Tantsur <divius.inside@gmail.com>
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Generic protocol bits for implementing custom protocols.

use num;

use super::Node;


/// Payload in the request.
pub enum RequestPayload<TValue> {
    Ping,
    FindNode(num::BigUint),
    FindValue(num::BigUint),
    Store(num::BigUint, TValue)
}

/// Request structure.
pub struct Request<TValue> {
    pub caller: Node,
    pub request_id: num::BigUint,
    pub payload: RequestPayload<TValue>
}

/// Payload in the response.
pub enum ResponsePayload<TValue> {
    NodesFound(Vec<Node>),
    ValueFound(TValue),
    NoResult
}

/// Response structure.
pub struct Response<TValue> {
    pub request: Request<TValue>,
    pub responder: Node,
    pub payload: ResponsePayload<TValue>
}

/// Trait for a protocol implementation.
pub trait Protocol : Send {
    /// Value type.
    type Value: Send + Sync;
    /// Parse request from binary data.
    fn parse_request(&self, data: &[u8]) -> Request<Self::Value>;
    /// Format response to binary data.
    fn format_response(&self, Response<Self::Value>) -> Vec<u8>;
}
