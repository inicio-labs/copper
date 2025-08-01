mod height;
mod validator;
mod value;
mod vote;

pub use self::{
	height::ConsensusHeight,
	validator::{CopperValidator, CopperValidatorSet, ValidatorAddress},
	value::{CopperValue, CopperValueId},
	vote::CopperVote,
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CopperGenesis {
	pub validator_set: CopperValidatorSet,
}
