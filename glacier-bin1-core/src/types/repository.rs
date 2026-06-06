use std::{fmt::Display, str::FromStr};

use facet::Facet;
use serde_with::{DeserializeFromStr, SerializeDisplay};
use thiserror::Error;

use crate::{de::Bin1Deserialize, ser::Bin1Serialize};

#[derive(
	SerializeDisplay,
	DeserializeFromStr,
	Debug,
	Clone,
	PartialEq,
	Eq,
	PartialOrd,
	Ord,
	Bin1Serialize,
	Bin1Deserialize,
	Facet,
)]
#[bin1(crate = crate)]
pub struct ZRepositoryID {
	pub data_1: u32,
	pub data_2: u16,
	pub data_3: u16,
	pub data_4: [u8; 8]
}

#[derive(Error, Debug)]
pub enum RepositoryIdError {
	#[error("failed to parse repository ID component as hex")]
	ParseError(#[from] std::num::ParseIntError),

	#[error("not enough dash separated parts")]
	NotEnoughParts,

	#[error("not enough characters in part")]
	NotEnoughChars
}

impl Display for ZRepositoryID {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"{:08X}-{:04X}-{:04X}-{:02X}{:02X}-{:02X}{:02X}{:02X}{:02X}{:02X}{:02X}",
			self.data_1,
			self.data_2,
			self.data_3,
			self.data_4[0],
			self.data_4[1],
			self.data_4[2],
			self.data_4[3],
			self.data_4[4],
			self.data_4[5],
			self.data_4[6],
			self.data_4[7],
		)
	}
}

impl FromStr for ZRepositoryID {
	type Err = RepositoryIdError;

	fn from_str(value: &str) -> Result<Self, Self::Err> {
		let mut parts = value.split('-');

		Ok(Self {
			data_1: u32::from_str_radix(parts.next().ok_or(RepositoryIdError::NotEnoughParts)?, 16)?,
			data_2: u16::from_str_radix(parts.next().ok_or(RepositoryIdError::NotEnoughParts)?, 16)?,
			data_3: u16::from_str_radix(parts.next().ok_or(RepositoryIdError::NotEnoughParts)?, 16)?,
			data_4: {
				let mut data_4 = [0u8; 8];

				let mut part = parts.next().ok_or(RepositoryIdError::NotEnoughParts)?.chars();
				for item in data_4.iter_mut().take(2) {
					let char1 = part.next().ok_or(RepositoryIdError::NotEnoughChars)?;
					let char2 = part.next().ok_or(RepositoryIdError::NotEnoughChars)?;
					*item = u8::from_str_radix(&[char1, char2].into_iter().collect::<String>(), 16)?;
				}

				let mut part = parts.next().ok_or(RepositoryIdError::NotEnoughParts)?.chars();
				for item in data_4.iter_mut().skip(2) {
					let char1 = part.next().ok_or(RepositoryIdError::NotEnoughChars)?;
					let char2 = part.next().ok_or(RepositoryIdError::NotEnoughChars)?;
					*item = u8::from_str_radix(&[char1, char2].into_iter().collect::<String>(), 16)?;
				}

				data_4
			}
		})
	}
}
