pub use crate::__define_enum as define_enum;

#[doc(hidden)]
#[macro_export]
macro_rules! __define_enum {
	($ty:ty, $name:ident { $($rust_variant:ident => $orig_variant:ident = $value:expr),+ }) => {
		#[derive(facet::Facet, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
		pub struct $name(pub $ty);

		#[allow(non_upper_case_globals)]
		impl $name {
			$(
				pub const $rust_variant: Self = Self($value);
			)+

			pub const VARIANTS: &'static [Self] = &[
				$(
					Self::$rust_variant,
				)+
			];

			pub const fn from_rust_name(name: &str) -> Option<Self> {
				match name.as_bytes() {
					$(
						$crate::types::enums::__bstringify!($rust_variant) => Some(Self::$rust_variant),
					)+
					_ => None
				}
			}

			pub const fn rust_variant_name(self) -> Option<&'static str> {
				match self {
					$(
						Self::$rust_variant => Some(stringify!($rust_variant)),
					)+
					_ => None
				}
			}

			pub const fn from_name(name: &str) -> Option<Self> {
				match name.as_bytes() {
					$(
						$crate::types::enums::__bstringify!($orig_variant) => Some(Self::$rust_variant),
					)+
					_ => None
				}
			}

			pub const fn variant_name(self) -> Option<&'static str> {
				match self {
					$(
						Self::$rust_variant => Some(stringify!($orig_variant)),
					)+
					_ => None
				}
			}
		}

		impl ::std::fmt::Debug for $name {
			fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
				match if f.alternate() {
					self.rust_variant_name()
				} else {
					self.variant_name()
				} {
					Some(name) => f.write_str(name),
					None => write!(f, "{}", self.0),
				}
			}
		}

		impl $crate::ser::Aligned for $name {
			const ALIGNMENT: usize = std::mem::size_of::<$ty>();
		}

		impl $crate::de::Bin1Sized for $name {
			const SIZE: usize = std::mem::size_of::<$ty>();
		}

		impl ::std::str::FromStr for $name {
			type Err = ();

			fn from_str(s: &str) -> Result<Self, Self::Err> {
				Self::from_name(s).ok_or(())
			}
		}

		impl From<$ty> for $name {
			fn from(value: $ty) -> Self {
				Self(value)
			}
		}

		impl From<$name> for $ty {
			fn from(value: $name) -> Self {
				value.0
			}
		}

		impl $crate::ser::Bin1Serialize for $name {
			fn alignment(&self) -> usize {
				<Self as $crate::ser::Aligned>::ALIGNMENT
			}

			fn write(&self, ser: &mut $crate::ser::Bin1Serializer) -> Result<(), $crate::ser::SerializeError> {
				ser.write_unaligned(&self.0.to_le_bytes());
				Ok(())
			}
		}

		impl $crate::de::Bin1Deserialize for $name {
			fn read(de: &mut $crate::de::Bin1Deserializer) -> Result<Self, $crate::de::DeserializeError> {
				<$ty as $crate::de::Bin1Deserialize>::read(de).map(Self)
			}
		}

		impl serde::Serialize for $name {
			fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
			where
				S: serde::Serializer,
			{
				if let Some(name) = self.variant_name() {
					serializer.serialize_str(name)
				} else {
					serializer.serialize_i64(self.0 as i64)
				}
			}
		}

		impl <'de> serde::Deserialize<'de> for $name {
			fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
			where
				D: serde::Deserializer<'de>,
			{
				if deserializer.is_human_readable() {
					struct Visitor;

					impl<'de> serde::de::Visitor<'de> for Visitor {
						type Value = $name;

						fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
							formatter.write_str("an enum variant name or integer")
						}

						fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
						where
							E: serde::de::Error,
						{
							Self::Value::from_name(value).ok_or_else(|| E::custom(format!("unknown variant {value}")))
						}

						fn visit_i8<E>(self, value: i8) -> Result<Self::Value, E>
						where
							E: serde::de::Error,
						{
							Ok($name(value as $ty))
						}

						fn visit_i16<E>(self, value: i16) -> Result<Self::Value, E>
						where
							E: serde::de::Error,
						{
							Ok($name(value as $ty))
						}

						fn visit_i32<E>(self, value: i32) -> Result<Self::Value, E>
						where
							E: serde::de::Error,
						{
							Ok($name(value as $ty))
						}

						fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E>
						where
							E: serde::de::Error,
						{
							Ok($name(value as $ty))
						}

						fn visit_u8<E>(self, value: u8) -> Result<Self::Value, E>
						where
							E: serde::de::Error,
						{
							Ok($name(value as $ty))
						}

						fn visit_u16<E>(self, value: u16) -> Result<Self::Value, E>
						where
							E: serde::de::Error,
						{
							Ok($name(value as $ty))
						}

						fn visit_u32<E>(self, value: u32) -> Result<Self::Value, E>
						where
							E: serde::de::Error,
						{
							Ok($name(value as $ty))
						}

						fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E>
						where
							E: serde::de::Error,
						{
							Ok($name(value as $ty))
						}
					}

					deserializer.deserialize_any(Visitor)
				} else {
					<$ty as serde::Deserialize>::deserialize(deserializer).map(Self)
				}
			}
		}
	};
}

#[doc(hidden)]
pub use bstringify::bstringify as __bstringify;
