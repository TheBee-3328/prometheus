//! PROMETHEUS shared core: Model trait, multi-order + match CM, 32-bit arithmetic coder.

mod arithmetic;
mod contexts;
mod match_model;
mod model;
pub mod params;
mod preprocessor;

pub use arithmetic::{ArithmeticDecoder, ArithmeticEncoder};
pub use contexts::HashedOrders;
pub use match_model::{MatchModel, Order2MatchModel, OrdersMatchModel};
pub use model::{Model, Order2Model};
pub use preprocessor::{decode, encode};

/// Archive magic for Phase 1 (order-2 + arithmetic coding).
pub const MAGIC: &[u8; 4] = b"PRM1";
