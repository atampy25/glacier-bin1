use std::sync::Arc;

use tryvial::try_fn;

use crate::{
	de::{Bin1Deserialize, Bin1Deserializer, DeserializeError},
	ser::Aligned
};

macro_rules! impl_primitive {
	($ty:ty, $size:literal, $func:ident) => {
		impl Bin1Deserialize for $ty {
			const SIZE: usize = $size;

			fn read(de: &mut Bin1Deserializer) -> Result<Self, DeserializeError> {
				#[cfg(feature = "debug-log")]
				eprintln!("0x{:6X}: reading {}", de.position(), stringify!($ty));

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
		#[cfg(feature = "debug-log")]
		eprintln!("0x{:6X}: reading bool", de.position());

		de.read_u8().map(|v| v != 0)
	}
}

impl<T: Bin1Deserialize + 'static + Send + Sync> Bin1Deserialize for Arc<T> {
	const SIZE: usize = 8;

	fn read(de: &mut Bin1Deserializer) -> Result<Self, DeserializeError> {
		de.read_pointer(T::read)
	}
}

impl<T: Bin1Deserialize + 'static + Send + Sync> Bin1Deserialize for Option<Arc<T>> {
	const SIZE: usize = 8;

	#[try_fn]
	fn read(de: &mut Bin1Deserializer) -> Result<Self, DeserializeError> {
		let ptr = de.read_u64()?;

		if ptr == u64::MAX {
			None
		} else {
			de.seek_relative(-8)?;
			Some(de.read_pointer(T::read)?)
		}
	}
}

impl<T: Bin1Deserialize, U: Bin1Deserialize> Bin1Deserialize for (T, U) {
	const SIZE: usize =
		{ T::SIZE + ((Self::ALIGNMENT - ((T::SIZE + U::SIZE) % Self::ALIGNMENT)) % Self::ALIGNMENT) + U::SIZE };

	fn read(de: &mut Bin1Deserializer) -> Result<Self, DeserializeError> {
		#[cfg(feature = "debug-log")]
		eprintln!(
			"0x{:6X}: reading pair ({}, {})",
			de.position(),
			std::any::type_name::<T>(),
			std::any::type_name::<U>()
		);

		let first = T::read(de)?;
		de.align_to(U::ALIGNMENT)?;
		let second = U::read(de)?;
		de.align_to(Self::ALIGNMENT)?;
		Ok((first, second))
	}
}

impl Bin1Deserialize for () {
	const SIZE: usize = 0;

	fn read(_: &mut Bin1Deserializer) -> Result<Self, DeserializeError> {
		Ok(())
	}
}
