use facet::Facet;
use serde::{Deserialize, Serialize};
use tryvial::try_fn;

use crate::{
	de::{Bin1Deserialize, Bin1Deserializer, Bin1Sized, DeserializeError},
	ser::{Aligned, Bin1Serialize, Bin1Serializer, SerializeError}
};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Facet)]
pub struct ZResourceID {
	#[serde(rename = "m_IDHigh")]
	#[facet(rename = "m_IDHigh")]
	pub id_high: u32,

	#[serde(rename = "m_IDLow")]
	#[facet(rename = "m_IDLow")]
	pub id_low: u32
}

impl ZResourceID {
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

impl Aligned for ZResourceID {
	const ALIGNMENT: usize = 4;
}

impl Bin1Serialize for ZResourceID {
	fn alignment(&self) -> usize {
		Self::ALIGNMENT
	}

	fn write(&self, ser: &mut Bin1Serializer) -> Result<(), SerializeError> {
		ser.write_runtime_resource_id(self.id_high, self.id_low);

		Ok(())
	}
}

impl Bin1Sized for ZResourceID {
	const SIZE: usize = 8;
}

impl Bin1Deserialize for ZResourceID {
	#[try_fn]
	fn read(de: &mut Bin1Deserializer) -> Result<Self, DeserializeError> {
		let id_high = u32::read(de)?;
		let id_low = u32::read(de)?;
		Self { id_high, id_low }
	}
}

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
	const ALIGNMENT: usize = 4;
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

impl Bin1Sized for ZRuntimeResourceID {
	const SIZE: usize = 8;
}

impl Bin1Deserialize for ZRuntimeResourceID {
	#[try_fn]
	fn read(de: &mut Bin1Deserializer) -> Result<Self, DeserializeError> {
		let id_high = u32::read(de)?;
		let id_low = u32::read(de)?;
		Self { id_high, id_low }
	}
}

/// Serialisation of ZResourceID/ZRuntimeResourceID without emitting an entry in the runtime resource IDs segment.
/// For "real" values where the value contains the RuntimeID inline rather than an index in the reference list.
#[allow(non_snake_case, private_bounds)]
pub mod WithoutFixup {
	use crate::{
		de::{Bin1Deserialize, Bin1Sized},
		ser::{Aligned, Bin1Serialize, Bin1Serializer, SerializeError}
	};

	pub(crate) trait ResourceID: Aligned + Bin1Sized {
		fn from_high_low(high: u32, low: u32) -> Self;
		fn high(&self) -> u32;
		fn low(&self) -> u32;
	}

	impl ResourceID for super::ZResourceID {
		fn from_high_low(high: u32, low: u32) -> Self {
			Self {
				id_high: high,
				id_low: low
			}
		}

		fn high(&self) -> u32 {
			self.id_high
		}

		fn low(&self) -> u32 {
			self.id_low
		}
	}

	impl ResourceID for super::ZRuntimeResourceID {
		fn from_high_low(high: u32, low: u32) -> Self {
			Self {
				id_high: high,
				id_low: low
			}
		}

		fn high(&self) -> u32 {
			self.id_high
		}

		fn low(&self) -> u32 {
			self.id_low
		}
	}

	pub struct Ser<'a, T: ResourceID>(pub &'a T);

	impl<'a, T: ResourceID> From<&'a T> for Ser<'a, T> {
		fn from(value: &'a T) -> Self {
			Self(value)
		}
	}

	impl<'a, T: ResourceID> Aligned for Ser<'a, T> {
		const ALIGNMENT: usize = T::ALIGNMENT;
	}

	impl<'a, T: ResourceID> Bin1Sized for Ser<'a, T> {
		const SIZE: usize = T::SIZE;
	}

	impl<'a, T: ResourceID> Bin1Serialize for Ser<'a, T> {
		fn alignment(&self) -> usize {
			Self::ALIGNMENT
		}

		fn write(&self, ser: &mut Bin1Serializer) -> Result<(), SerializeError> {
			self.0.high().write(ser)?;
			self.0.low().write(ser)?;

			Ok(())
		}
	}

	pub struct De<T: ResourceID>(T);

	impl From<De<super::ZResourceID>> for super::ZResourceID {
		fn from(value: De<super::ZResourceID>) -> Self {
			value.0
		}
	}

	impl From<De<super::ZRuntimeResourceID>> for super::ZRuntimeResourceID {
		fn from(value: De<super::ZRuntimeResourceID>) -> Self {
			value.0
		}
	}

	impl<T: ResourceID> Aligned for De<T> {
		const ALIGNMENT: usize = T::ALIGNMENT;
	}

	impl<T: ResourceID> Bin1Sized for De<T> {
		const SIZE: usize = T::SIZE;
	}

	impl<T: ResourceID> Bin1Deserialize for De<T> {
		#[tryvial::try_fn]
		fn read(de: &mut crate::de::Bin1Deserializer) -> Result<Self, crate::de::DeserializeError> {
			let id_high = u32::read(de)?;
			let id_low = u32::read(de)?;
			De(T::from_high_low(id_high, id_low))
		}
	}
}

/// Serialisation of ZResourceID/ZRuntimeResourceID without emitting an entry in the runtime resource IDs segment.
/// For "real" values where the value contains the RuntimeID inline rather than an index in the reference list.
#[allow(non_snake_case, private_bounds)]
pub mod WithoutFixupVec {
	use super::WithoutFixup::ResourceID;
	use crate::{
		de::{Bin1Deserialize, Bin1Sized},
		ser::{Aligned, Bin1Serialize, Bin1Serializer, SerializeError}
	};

	pub struct Ser<'a, T: ResourceID>(pub &'a Vec<T>);

	impl<'a, T: ResourceID> From<&'a Vec<T>> for Ser<'a, T> {
		fn from(value: &'a Vec<T>) -> Self {
			Self(value)
		}
	}

	impl<'a, T: ResourceID> Aligned for Ser<'a, T> {
		const ALIGNMENT: usize = Vec::<T>::ALIGNMENT;
	}

	impl<'a, T: ResourceID + Bin1Serialize> Bin1Serialize for Ser<'a, T> {
		fn alignment(&self) -> usize {
			Self::ALIGNMENT
		}

		#[tryvial::try_fn]
		fn write(&self, ser: &mut Bin1Serializer) -> Result<(), SerializeError> {
			if self.0.is_empty() {
				#[cfg(feature = "debug-log")]
				eprintln!(
					"0x{:6X}: writing empty Vec<{}>",
					ser.position(),
					std::any::type_name::<T>()
				);
				ser.write_pointer(u64::MAX);
				ser.write_pointer(u64::MAX);
				ser.write_pointer(u64::MAX);
			} else {
				if T::SIZE != 0 && ser.inline_arrays() && self.0.len() * T::SIZE <= 16 {
					#[cfg(feature = "debug-log")]
					eprintln!(
						"0x{:6X}: writing inline Vec<{}> of {} items",
						ser.position(),
						std::any::type_name::<T>(),
						self.0.len()
					);
					// Inline optimisation
					let pos = ser.position();
					self.0
						.iter()
						.map(super::WithoutFixup::Ser::from)
						.collect::<Vec<_>>()
						.as_slice()
						.write(ser)?;
					ser.write_unaligned(&vec![0; 16 - (ser.position() - pos)]);

					// inline flag, count, capacity
					((1u64 << 62) | (self.0.len() as u8 as u64) | ((self.0.len() as u8 as u64) << 8)).write(ser)?;
				} else {
					#[cfg(feature = "debug-log")]
					eprintln!(
						"0x{:6X}: writing allocated Vec<{}> of {} items",
						ser.position(),
						std::any::type_name::<T>(),
						self.0.len()
					);

					let start_id = self.0.as_ptr() as u64 | 0xABCD000000000000; // fake pointers to avoid colliding with actual data
					let end_id = start_id | 0xCAFE000000000000;
					ser.write_pointer(start_id);
					ser.write_pointer(end_id);
					ser.write_pointer(end_id); // allocation end, which in serialisation/deserialisation is the same as the end
				}
			}
		}

		#[tryvial::try_fn]
		fn resolve(&self, ser: &mut Bin1Serializer) -> Result<(), SerializeError> {
			if self.0.is_empty() {
				return Ok(());
			}

			if T::SIZE != 0 && ser.inline_arrays() && self.0.len() * T::SIZE <= 16 {
				#[cfg(feature = "debug-log")]
				eprintln!(
					"0x{:6X}: resolving inline Vec<{}> of {} items",
					ser.position(),
					std::any::type_name::<T>(),
					self.0.len()
				);

				for item in self.0 {
					item.resolve(ser)?;
				}
			} else {
				#[cfg(feature = "debug-log")]
				eprintln!(
					"0x{:6X}: resolving allocated Vec<{}> of {} items",
					ser.position(),
					std::any::type_name::<T>(),
					self.0.len()
				);

				let start_id = self.0.as_ptr() as u64 | 0xABCD000000000000;
				let end_id = start_id | 0xCAFE000000000000;
				ser.write_pointee(
					start_id,
					Some(end_id),
					self.0
						.iter()
						.map(super::WithoutFixup::Ser::from)
						.collect::<Vec<_>>()
						.as_slice()
				)?;
			}
		}
	}

	pub struct De<T: ResourceID>(Vec<T>);

	impl From<De<super::ZResourceID>> for Vec<super::ZResourceID> {
		fn from(value: De<super::ZResourceID>) -> Self {
			value.0
		}
	}

	impl From<De<super::ZRuntimeResourceID>> for Vec<super::ZRuntimeResourceID> {
		fn from(value: De<super::ZRuntimeResourceID>) -> Self {
			value.0
		}
	}

	impl<T: ResourceID> Aligned for De<T> {
		const ALIGNMENT: usize = Vec::<T>::ALIGNMENT;
	}

	impl<T: ResourceID> Bin1Sized for De<T> {
		const SIZE: usize = Vec::<T>::SIZE;
	}

	impl<T: ResourceID + From<super::WithoutFixup::De<T>>> Bin1Deserialize for De<T> {
		#[tryvial::try_fn]
		fn read(de: &mut crate::de::Bin1Deserializer) -> Result<Self, crate::de::DeserializeError> {
			De(Vec::<super::WithoutFixup::De<T>>::read(de)?
				.into_iter()
				.map(|x| x.into())
				.collect())
		}
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
	const ALIGNMENT: usize = 4;
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

impl Bin1Sized for TResourcePtr {
	const SIZE: usize = 8;
}

impl Bin1Deserialize for TResourcePtr {
	#[try_fn]
	fn read(de: &mut Bin1Deserializer) -> Result<Self, DeserializeError> {
		let id_high = u32::read(de)?;
		let id_low = u32::read(de)?;
		Self { id_high, id_low }
	}
}
