//! PROMETHEUS shared core: Model trait, multi-order / match CM, 32-bit arithmetic coder.

mod arithmetic;
mod contexts;
mod match_model;
mod model;

pub use arithmetic::{ArithmeticDecoder, ArithmeticEncoder};
pub use contexts::OrdersModel;
pub use match_model::{MatchModel, Order2MatchModel};
pub use model::{Model, Order2Model};

/// Archive magic (context model + arithmetic coding).
pub const MAGIC: &[u8; 4] = b"PRM1";
