//! PROMETHEUS shared core: Model trait, order-2 + match CM, 32-bit arithmetic coder.

mod arithmetic;
mod match_model;
mod model;
pub mod params;
mod preprocessor;
mod sparse;

pub use arithmetic::{ArithmeticDecoder, ArithmeticEncoder};
pub use match_model::{MatchModel, Order2MatchModel};
pub use model::{Model, Order2Model};
pub use preprocessor::{decode, encode};
pub use sparse::SparseModel;

/// Archive magic for Phase 1 (order-2 + arithmetic coding).
pub const MAGIC: &[u8; 4] = b"PRM1";
