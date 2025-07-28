mod error;

use core::str::FromStr;

pub use self::error::DenomError;

use borsh::{BorshDeserialize, BorshSerialize};
use bytes::Bytes;

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Coin {
	denom: Denom,
	amount: u128,
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Denom(Bytes);

impl Coin {
	pub fn new(denom: Denom, amount: u128) -> Self {
		Self { denom, amount }
	}

	pub fn denom(&self) -> &Denom {
		&self.denom
	}

	pub fn amount(&self) -> u128 {
		self.amount
	}

	pub fn dissolve(self) -> (Denom, u128) {
		let Self { denom, amount } = self;
		(denom, amount)
	}
}

impl Denom {
	const MIN_LENGTH: usize = 3;

	pub fn as_str(&self) -> &str {
		// unwrap is safe here because the constructor takes valid utf8.
		str::from_utf8(self.0.as_ref()).unwrap()
	}
}

impl FromStr for Denom {
	type Err = DenomError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		s.len()
			.ge(&Self::MIN_LENGTH)
			.then_some(s.as_bytes())
			.map(Bytes::copy_from_slice)
			.map(Self)
			.ok_or(DenomError::MinimumLength)
	}
}

impl TryFrom<Bytes> for Denom {
	type Error = DenomError;

	fn try_from(bz: Bytes) -> Result<Self, Self::Error> {
		if bz.len() < Self::MIN_LENGTH {
			return Err(DenomError::MinimumLength);
		}

		if str::from_utf8(&bz).is_err() {
			return Err(DenomError::InvalidChar);
		}

		Ok(Self(bz))
	}
}

impl TryFrom<String> for Denom {
	type Error = DenomError;

	fn try_from(s: String) -> Result<Self, Self::Error> {
		let bz = s.into_bytes();

		if bz.len() < Self::MIN_LENGTH {
			return Err(DenomError::MinimumLength);
		}

		Ok(Self(bz.into()))
	}
}
