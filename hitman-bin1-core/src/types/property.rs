use std::fmt;

use bimap::BiMap;
use ecow::EcoString;
use facet::Facet;
use serde::{
	Deserialize, Serialize,
	de::{self, Visitor}
};

use crate::{
	de::{Bin1Deserialize, Bin1Deserializer, DeserializeError},
	ser::{Aligned, Bin1Serialize, Bin1Serializer, SerializeError}
};

#[static_init::dynamic]
static PROPERTIES: BiMap<&'static str, u32> = include_str!("../../properties.txt")
	.lines()
	.zip(
		include_str!(concat!(env!("OUT_DIR"), "/properties-crc32.txt"))
			.lines()
			.map(|x| x.parse().unwrap())
	)
	.collect();

#[static_init::dynamic]
static CUSTOM_PROPERTIES: papaya::HashMap<u32, EcoString, rapidhash::fast::RandomState> = Default::default();

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Facet)]
pub struct PropertyID(pub u32);

impl PropertyID {
	pub fn as_name(&self) -> Option<EcoString> {
		if let Some(known) = PROPERTIES.get_by_right(&self.0).copied() {
			Some(known.into())
		} else {
			CUSTOM_PROPERTIES.pin().get(&self.0).map(|custom| custom.to_owned())
		}
	}

	pub fn as_known(&self) -> Option<&'static str> {
		PROPERTIES.get_by_right(&self.0).copied()
	}

	pub fn is_known(name: &str) -> bool {
		PROPERTIES.contains_left(name)
	}
}

impl Aligned for PropertyID {
	const ALIGNMENT: usize = u32::ALIGNMENT;
}

impl Bin1Serialize for PropertyID {
	fn alignment(&self) -> usize {
		Self::ALIGNMENT
	}

	fn write(&self, ser: &mut Bin1Serializer) -> Result<(), SerializeError> {
		self.0.write(ser)
	}
}

impl Bin1Deserialize for PropertyID {
	const SIZE: usize = u32::SIZE;

	fn read(de: &mut Bin1Deserializer) -> Result<Self, DeserializeError> {
		u32::read(de).map(Self)
	}
}

impl From<u32> for PropertyID {
	fn from(value: u32) -> Self {
		Self(value)
	}
}

impl From<PropertyID> for u32 {
	fn from(value: PropertyID) -> Self {
		value.0
	}
}

impl From<EcoString> for PropertyID {
	fn from(value: EcoString) -> Self {
		if let Some(known) = PROPERTIES.get_by_left(value.as_str()).copied() {
			Self(known)
		} else {
			let hash = crc32fast::hash(value.as_bytes());
			CUSTOM_PROPERTIES.pin().get_or_insert(hash, value);
			Self(hash)
		}
	}
}

impl From<&str> for PropertyID {
	fn from(value: &str) -> Self {
		if let Some(known) = PROPERTIES.get_by_left(value).copied() {
			Self(known)
		} else {
			let hash = crc32fast::hash(value.as_bytes());
			CUSTOM_PROPERTIES.pin().get_or_insert_with(hash, || value.into());
			Self(hash)
		}
	}
}

impl fmt::Display for PropertyID {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		if let Some(known) = PROPERTIES.get_by_right(&self.0).copied() {
			write!(f, "{known}")
		} else if let Some(custom) = CUSTOM_PROPERTIES.pin().get(&self.0) {
			write!(f, "{custom}")
		} else {
			write!(f, "{}", self.0)
		}
	}
}

impl fmt::Debug for PropertyID {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(f, "{self}")
	}
}

impl Serialize for PropertyID {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer
	{
		if let Some(name) = self.as_name() {
			serializer.serialize_str(&name)
		} else {
			serializer.serialize_u32(self.0)
		}
	}
}

impl<'de> Deserialize<'de> for PropertyID {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>
	{
		struct PropertyIDVisitor;

		impl<'de> Visitor<'de> for PropertyIDVisitor {
			type Value = PropertyID;

			fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
				write!(f, "a property ID as a string or integer")
			}

			fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
			where
				E: de::Error
			{
				Ok(v.into())
			}

			fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E>
			where
				E: de::Error
			{
				Ok(v.into())
			}

			fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
			where
				E: de::Error
			{
				if v <= u32::MAX as u64 {
					Ok((v as u32).into())
				} else {
					Err(E::custom("integer out of range for PropertyID"))
				}
			}

			fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
			where
				E: de::Error
			{
				if v >= 0 && v <= u32::MAX as i64 {
					Ok((v as u32).into())
				} else {
					Err(E::custom("integer out of range for PropertyID"))
				}
			}
		}

		deserializer.deserialize_any(PropertyIDVisitor)
	}
}
