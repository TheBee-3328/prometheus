//! PROMETHEUS shared core: Model trait, order-2 + word CM, 32-bit arithmetic coder.

mod arithmetic;
mod model;
mod word_model;

pub use arithmetic::{ArithmeticDecoder, ArithmeticEncoder};
pub use model::{CombinedModel, Model, Order2Model};
pub use word_model::WordModel;

/// Archive magic for Phase 1+ (context model + arithmetic coding).
pub const MAGIC: &[u8; 4] = b"PRM1";
