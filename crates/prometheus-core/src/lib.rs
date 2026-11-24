//! PROMETHEUS shared core: Model trait, order-2 + match CM, 32-bit arithmetic coder.

mod apm;
mod arithmetic;
mod match_model;
mod model;
mod order0;
pub mod params;
mod preprocessor;
mod sparse;
mod sse;
mod word;

pub use arithmetic::{ArithmeticDecoder, ArithmeticEncoder};
pub use match_model::{MatchModel, Order2MatchModel};
pub use model::{HashOrderModel, Model, Order1Model, Order2Model};
pub use preprocessor::{decode, encode};
pub use sparse::SparseModel;

/// Archive magic for Phase 1 (order-2 + arithmetic coding).
pub const MAGIC: &[u8; 4] = b"PRM1";
