use std::ops::{Deref, DerefMut};

use crate::{
	de::Bin1Deserialize,
	ser::{Aligned, Bin1Serialize, Bin1Serializer, SerializeError}
};

pub struct Owned<T> {
	pub value: T,
	identity: u64
}

impl<T> Owned<T> {
	pub fn new(value: T) -> Self {
		Self {
			value,
			identity: rand::random()
		}
	}
}

impl<T> From<T> for Owned<T> {
	fn from(value: T) -> Self {
		Self {
			value,
			identity: rand::random()
		}
	}
}

impl<T> Deref for Owned<T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		&self.value
	}
}

impl<T> DerefMut for Owned<T> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.value
	}
}

impl<T> Aligned for Owned<T> {
	const ALIGNMENT: usize = 8;
}

impl<T: Bin1Serialize> Bin1Serialize for Owned<T> {
	fn alignment(&self) -> usize {
		Self::ALIGNMENT
	}

	fn write(&self, ser: &mut Bin1Serializer) -> Result<(), SerializeError> {
		ser.write_pointer(self.identity);
		Ok(())
	}

	fn resolve(&self, ser: &mut Bin1Serializer) -> Result<(), SerializeError> {
		ser.write_pointee(self.identity, None, &self.value)
	}
}

impl<T: Bin1Deserialize> Bin1Deserialize for Owned<T> {
	const SIZE: usize = 8;

	#[tryvial::try_fn]
	fn read(de: &mut crate::de::Bin1Deserializer) -> Result<Self, crate::de::DeserializeError> {
		let ptr = de.read_u64()?;
		let pos = de.position();

		de.seek_from_start(ptr + 0x10)?;
		let value = T::read(de)?;
		de.seek_from_start(pos)?;

		Self {
			value,
			identity: rand::random()
		}
	}
}

#[allow(non_snake_case)]
pub mod WithZeroNull {
	use std::sync::Arc;

	use crate::{
		de::Bin1Deserialize,
		ser::{Aligned, Bin1Serialize, Bin1Serializer, SerializeError}
	};

	pub struct Ser<'a, T: Bin1Serialize>(pub &'a Option<Arc<T>>);

	impl<'a, T: Bin1Serialize> From<&'a Option<Arc<T>>> for Ser<'a, T> {
		fn from(value: &'a Option<Arc<T>>) -> Self {
			Self(value)
		}
	}

	impl<'a, T: Bin1Serialize> Aligned for Ser<'a, T> {
		const ALIGNMENT: usize = 8;
	}

	impl<'a, T: Bin1Serialize + Aligned> Bin1Serialize for Ser<'a, T> {
		fn alignment(&self) -> usize {
			Self::ALIGNMENT
		}

		fn write(&self, ser: &mut Bin1Serializer) -> Result<(), SerializeError> {
			if let Some(value) = self.0.as_ref() {
				value.write_aligned(ser)?;
			} else {
				ser.align_to(8);
				ser.write_unaligned(&0u64.to_le_bytes());
			}

			Ok(())
		}

		fn resolve(&self, ser: &mut Bin1Serializer) -> Result<(), SerializeError> {
			if let Some(value) = self.0.as_ref() {
				value.resolve(ser)?;
			}

			Ok(())
		}
	}

	pub struct De<T: Bin1Deserialize>(Option<Arc<T>>);

	impl<T: Bin1Deserialize> From<De<T>> for Option<Arc<T>> {
		fn from(value: De<T>) -> Self {
			value.0
		}
	}

	impl<T: Bin1Deserialize> Aligned for De<T> {
		const ALIGNMENT: usize = 8;
	}

	impl<T: Bin1Deserialize + 'static + Send + Sync> Bin1Deserialize for De<T> {
		const SIZE: usize = 8;

		#[tryvial::try_fn]
		fn read(de: &mut crate::de::Bin1Deserializer) -> Result<Self, crate::de::DeserializeError> {
			let ptr = de.read_u64()?;

			if ptr == 0 {
				De(None)
			} else {
				de.seek_relative(-8)?;
				De(Some(de.read_pointer(T::read)?))
			}
		}
	}
}
