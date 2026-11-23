//! PROMETHEUS shared core: models, logistic mixer, APM, arithmetic coder.

mod apm;
mod arithmetic;
mod mixer;
mod model;

pub use apm::{squash, stretch, Apm};
pub use arithmetic::{ArithmeticDecoder, ArithmeticEncoder};
pub use mixer::{LogisticMixer, TwoLayerMixer};
pub use model::{MixedModel, Model, Order0Model, Order1Model, Order2Model};

/// Archive magic for Phase 1/2 (CM + arithmetic coding).
pub const MAGIC: &[u8; 4] = b"PRM1";
