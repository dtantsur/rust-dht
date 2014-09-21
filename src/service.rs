//! DHT service

use super::GenericNodeTable;
use super::GenericRpc;


// TODO(divius): implement
/// Structure representing main DHT service.
pub struct Service<TNodeTable: GenericNodeTable, TRpc: GenericRpc> {
    #[allow(dead_code)]
    node_table: TNodeTable,
    #[allow(dead_code)]
    rpc: TRpc,
}
