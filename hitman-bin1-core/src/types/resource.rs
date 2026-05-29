use facet::Facet;
use serde::{Deserialize, Serialize};
use tryvial::try_fn;

use crate::{
	de::{Bin1Deserialize, Bin1Deserializer, DeserializeError},
	ser::{Aligned, Bin1Serialize, Bin1Serializer, SerializeError}
};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Facet)]
pub struct ZRuntimeResourceID {
	#[serde(rename = "m_IDHigh")]
	#[facet(rename = "m_IDHigh")]
	pub id_high: u32,

	#[serde(rename = "m_IDLow")]
	#[facet(rename = "m_IDLow")]
	pub id_low: u32
}

impl ZRuntimeResourceID {
	pub fn from_u64(id: u64) -> Self {
		Self {
			id_high: (id >> 32) as u32,
			id_low: (id & 0xFFFFFFFF) as u32
		}
	}

	pub fn as_u64(&self) -> u64 {
		((self.id_high as u64) << 32) | (self.id_low as u64)
	}
}

impl Aligned for ZRuntimeResourceID {
	const ALIGNMENT: usize = 8;
}

impl Bin1Serialize for ZRuntimeResourceID {
	fn alignment(&self) -> usize {
		Self::ALIGNMENT
	}

	fn write(&self, ser: &mut Bin1Serializer) -> Result<(), SerializeError> {
		ser.write_runtime_resource_id(self.id_high, self.id_low);

		Ok(())
	}
}

impl Bin1Deserialize for ZRuntimeResourceID {
	const SIZE: usize = 8;

	#[try_fn]
	fn read(de: &mut Bin1Deserializer) -> Result<Self, DeserializeError> {
		let id_high = u32::read(de)?;
		let id_low = u32::read(de)?;
		Self { id_high, id_low }
	}
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Facet)]
pub struct TResourcePtr {
	#[serde(rename = "m_IDHigh")]
	#[facet(rename = "m_IDHigh")]
	pub id_high: u32,

	#[serde(rename = "m_IDLow")]
	#[facet(rename = "m_IDLow")]
	pub id_low: u32
}

impl TResourcePtr {
	pub fn from_u64(id: u64) -> Self {
		Self {
			id_high: (id >> 32) as u32,
			id_low: (id & 0xFFFFFFFF) as u32
		}
	}

	pub fn as_u64(&self) -> u64 {
		((self.id_high as u64) << 32) | (self.id_low as u64)
	}
}

impl Aligned for TResourcePtr {
	const ALIGNMENT: usize = 8;
}

impl Bin1Serialize for TResourcePtr {
	fn alignment(&self) -> usize {
		Self::ALIGNMENT
	}

	fn write(&self, ser: &mut Bin1Serializer) -> Result<(), SerializeError> {
		ser.write_resource_ptr(self.id_high, self.id_low);

		Ok(())
	}
}

impl Bin1Deserialize for TResourcePtr {
	const SIZE: usize = 8;

	#[try_fn]
	fn read(de: &mut Bin1Deserializer) -> Result<Self, DeserializeError> {
		let id_high = u32::read(de)?;
		let id_low = u32::read(de)?;
		Self { id_high, id_low }
	}
}
