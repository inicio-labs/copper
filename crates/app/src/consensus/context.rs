use bytes::Bytes;
use malachitebft_app_channel::app::types::core::{
	Context, NilOrVal, Round, ValidatorSet, ValueId, VoteType,
};
use malachitebft_test::Ed25519;

use super::{
	streaming::{CopperProposal, CopperProposalPart},
	types::{
		ConsensusHeight, CopperValidator, CopperValidatorSet, CopperValue, CopperVote,
		ValidatorAddress,
	},
};

#[derive(Debug, Clone)]
pub struct ConsensusContext;

impl Context for ConsensusContext {
	type Address = ValidatorAddress;

	type Height = ConsensusHeight;

	type ProposalPart = CopperProposalPart;

	type Proposal = CopperProposal;

	type Validator = CopperValidator;

	type ValidatorSet = CopperValidatorSet;

	type Value = CopperValue;

	type Vote = CopperVote;

	type Extension = Bytes;

	type SigningScheme = Ed25519;

	fn select_proposer<'a>(
		&self,
		validator_set: &'a Self::ValidatorSet,
		height: Self::Height,
		round: Round,
	) -> &'a Self::Validator {
		assert!(validator_set.count() > 0);
		assert!(round != Round::Nil && round.as_i64() >= 0);

		let proposer_index = {
			let height = height.as_u64() as usize;
			let round = round.as_i64() as usize;

			(height - 1 + round) % validator_set.count()
		};

		validator_set.get_by_index(proposer_index).expect("proposer_index is valid")
	}

	fn new_proposal(
		&self,
		height: Self::Height,
		round: Round,
		value: Self::Value,
		pol_round: Round,
		validator_address: Self::Address,
	) -> Self::Proposal {
		CopperProposal { height, round, value, pol_round, validator_address }
	}

	fn new_prevote(
		&self,
		height: Self::Height,
		round: Round,
		value: NilOrVal<ValueId<Self>>,
		validator_address: Self::Address,
	) -> Self::Vote {
		CopperVote {
			typ: VoteType::Prevote,
			height,
			round,
			value,
			validator_address,
			extension: None,
		}
	}

	fn new_precommit(
		&self,
		height: Self::Height,
		round: Round,
		value: NilOrVal<ValueId<Self>>,
		validator_address: Self::Address,
	) -> Self::Vote {
		CopperVote {
			typ: VoteType::Precommit,
			height,
			round,
			value,
			validator_address,
			extension: None,
		}
	}
}
