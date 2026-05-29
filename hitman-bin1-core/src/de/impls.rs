use std::sync::Arc;

use tryvial::try_fn;

use crate::de::{Bin1Deserialize, Bin1Deserializer, DeserializeError};

macro_rules! impl_primitive {
	($ty:ty, $size:literal, $func:ident) => {
		impl Bin1Deserialize for $ty {
			const SIZE: usize = $size;

			fn read(de: &mut Bin1Deserializer) -> Result<Self, DeserializeError> {
				de.$func()
			}
		}
	};
}

impl_primitive!(u8, 1, read_u8);
impl_primitive!(u16, 2, read_u16);
impl_primitive!(u32, 4, read_u32);
impl_primitive!(u64, 8, read_u64);

impl_primitive!(i8, 1, read_i8);
impl_primitive!(i16, 2, read_i16);
impl_primitive!(i32, 4, read_i32);
impl_primitive!(i64, 8, read_i64);

impl_primitive!(f32, 4, read_f32);
impl_primitive!(f64, 8, read_f64);

impl Bin1Deserialize for bool {
	const SIZE: usize = 1;

	fn read(de: &mut Bin1Deserializer) -> Result<Self, DeserializeError> {
		de.read_u8().map(|v| v != 0)
	}
}

impl<T: Bin1Deserialize + 'static + Send + Sync> Bin1Deserialize for Arc<T> {
	const SIZE: usize = 8;

	fn read(de: &mut Bin1Deserializer) -> Result<Self, DeserializeError> {
		de.read_pointer(|de| {
			de.align_to(T::ALIGNMENT)?;
			T::read(de)
		})
	}
}

impl<T: Bin1Deserialize + 'static + Send + Sync> Bin1Deserialize for Option<Arc<T>> {
	const SIZE: usize = 8;

	#[try_fn]
	fn read(de: &mut Bin1Deserializer) -> Result<Self, DeserializeError> {
		de.align_to(8)?;
		let ptr = de.read_u64()?;

		if ptr == u64::MAX {
			None
		} else {
			de.seek_relative(-8)?;
			Some(de.read_pointer(|de| {
				de.align_to(T::ALIGNMENT)?;
				T::read(de)
			})?)
		}
	}
}

impl<T: Bin1Deserialize, U: Bin1Deserialize> Bin1Deserialize for (T, U) {
	const SIZE: usize = {
		let alignment = if U::ALIGNMENT > T::ALIGNMENT {
			U::ALIGNMENT
		} else {
			T::ALIGNMENT
		};

		T::SIZE + ((alignment - ((T::SIZE + U::SIZE) % alignment)) % alignment) + U::SIZE
	};

	fn read(de: &mut Bin1Deserializer) -> Result<Self, DeserializeError> {
		let first = T::read(de)?;
		de.align_to(U::ALIGNMENT)?;
		let second = U::read(de)?;
		de.align_to(U::ALIGNMENT)?;
		Ok((first, second))
	}
}

impl Bin1Deserialize for () {
	const SIZE: usize = 0;

	fn read(_: &mut Bin1Deserializer) -> Result<Self, DeserializeError> {
		Ok(())
	}
}
