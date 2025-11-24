use bincode::{Decode, Encode};
use derive_more::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Display, Encode, Decode)]
pub enum AggregationOp {
    #[display("mean")]
    Mean,
    #[display("max")]
    Max,
    #[display("min")]
    Min,
    #[display("sum")]
    Sum,
    /// if any boolean is true (or)
    #[display("any")]
    Any,
    /// if all booleans are true (and)
    #[display("all")]
    All,
}

pub const AGGREGATION_OPS: [AggregationOp; 6] = [
    AggregationOp::Mean,
    AggregationOp::Max,
    AggregationOp::Min,
    AggregationOp::Sum,
    AggregationOp::Any,
    AggregationOp::All,
];
