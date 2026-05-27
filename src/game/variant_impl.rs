use std::{
	collections::HashMap,
	fmt::{self, Debug},
	ops::{Deref, DerefMut},
	sync::Arc
};

use ecow::EcoString;
use facet::Facet;
use polonius_the_crab::{polonius, polonius_return};
use serde::{Deserialize, Serialize, de::DeserializeOwned, ser::SerializeStruct};
use tryvial::try_fn;

use crate::types::{property::PropertyID, resource::ZRuntimeResourceID};

pub trait DeserializeVariant: Send + Sync {
	fn type_id(&self) -> &str;
	fn deserialize_serde(&self, type_id: &str, value: serde_json::Value) -> Result<Arc<dyn Variant>, String>;
	fn deserialize_bin1(&self, type_id: &str, de: &mut Bin1Deserializer) -> Result<Arc<dyn Variant>, DeserializeError>;
}

pub struct VariantDeserializer<T: StaticVariant + Variant + DeserializeOwned + 'static + Send + Sync>(
	std::marker::PhantomData<T>
);

impl<T: StaticVariant + Variant + DeserializeOwned + 'static + Send + Sync> VariantDeserializer<T> {
	#[allow(clippy::new_without_default)]
	pub const fn new() -> Self {
		Self(std::marker::PhantomData)
	}
}

impl<T: StaticVariant + Variant + Bin1Deserialize + DeserializeOwned + 'static + Send + Sync> DeserializeVariant
	for VariantDeserializer<T>
{
	fn type_id(&self) -> &str {
		T::TYPE_ID
	}

	fn deserialize_serde(&self, type_id: &str, value: serde_json::Value) -> Result<Arc<dyn Variant>, String> {
		if type_id != T::TYPE_ID {
			return Err(format!("Cannot deserialize {} into {}", type_id, T::TYPE_ID));
		}

		serde_json::from_value::<T>(value)
			.map(|v| Arc::new(v) as Arc<dyn Variant>)
			.map_err(|e| format!("{e}"))
	}

	fn deserialize_bin1(&self, type_id: &str, de: &mut Bin1Deserializer) -> Result<Arc<dyn Variant>, DeserializeError> {
		if type_id != T::TYPE_ID {
			return Err(DeserializeError::TypeMismatch {
				expected: T::TYPE_ID,
				found: type_id.to_owned()
			});
		}

		T::read(de).map(|v| Arc::new(v) as Arc<dyn Variant>)
	}
}

/// Pool of serde-deserialised variants to deduplicate identical values.
#[static_init::dynamic]
static VARIANT_POOL: papaya::HashMap<ValueWrapper, std::sync::Weak<dyn Variant>, rapidhash::fast::RandomState> =
	Default::default();

struct ValueWrapper {
	value: serde_json::Value
}

#[derive(PartialEq, Eq)]
struct BorrowedValueWrapper<'a> {
	value: serde_json_borrow::Value<'a>
}

impl PartialEq for ValueWrapper {
	fn eq(&self, other: &Self) -> bool {
		self.value == other.value
	}
}

impl Eq for ValueWrapper {}

impl<'a> equivalent::Equivalent<ValueWrapper> for BorrowedValueWrapper<'a> {
	fn equivalent(&self, other: &ValueWrapper) -> bool {
		fn equiv(a: &serde_json_borrow::Value<'_>, b: &serde_json::Value) -> bool {
			match (a, b) {
				(serde_json_borrow::Value::Null, serde_json::Value::Null) => true,
				(serde_json_borrow::Value::Bool(ab), serde_json::Value::Bool(bb)) => ab == bb,
				(serde_json_borrow::Value::Number(an), serde_json::Value::Number(bn)) => {
					if let Some(av) = an.as_i64()
						&& let Some(bv) = bn.as_i64()
					{
						av == bv
					} else if let Some(av) = an.as_u64()
						&& let Some(bv) = bn.as_u64()
					{
						av == bv
					} else if let Some(av) = an.as_f64()
						&& let Some(bv) = bn.as_f64()
					{
						av == bv
					} else {
						false
					}
				}
				(serde_json_borrow::Value::Str(as_), serde_json::Value::String(bs)) => as_ == bs,
				(serde_json_borrow::Value::Array(aa), serde_json::Value::Array(bb)) => {
					if aa.len() != bb.len() {
						return false;
					}

					for (ae, be) in aa.iter().zip(bb.iter()) {
						if !equiv(ae, be) {
							return false;
						}
					}

					true
				}
				(serde_json_borrow::Value::Object(am), serde_json::Value::Object(bm)) => {
					if am.len() != bm.len() {
						return false;
					}

					for (ak, av) in am.iter() {
						if let Some(bv) = bm.get(ak) {
							if !equiv(av, bv) {
								return false;
							}
						} else {
							return false;
						}
					}

					// Technically we should check that bm doesn't have extra keys not in am, but it doesn't really matter

					true
				}
				_ => false
			}
		}

		equiv(&self.value, &other.value)
	}
}

impl std::hash::Hash for ValueWrapper {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		fn hash_value<H: std::hash::Hasher>(value: &serde_json::Value, state: &mut H) {
			match value {
				serde_json::Value::Null => {
					0u8.hash(state);
				}
				serde_json::Value::Bool(b) => {
					1u8.hash(state);
					b.hash(state);
				}
				serde_json::Value::Number(n) => {
					2u8.hash(state);
					if let Some(i) = n.as_i64() {
						i.hash(state);
					} else if let Some(u) = n.as_u64() {
						u.hash(state);
					} else if let Some(f) = n.as_f64() {
						f.to_bits().hash(state);
					}
				}
				serde_json::Value::String(s) => {
					3u8.hash(state);
					s.hash(state);
				}
				serde_json::Value::Array(arr) => {
					4u8.hash(state);
					for item in arr {
						hash_value(item, state);
					}
				}
				serde_json::Value::Object(obj) => {
					5u8.hash(state);
					let mut entries = obj.iter().collect::<Vec<_>>();
					entries.sort_by_key(|&(k, _)| k);
					for (k, v) in entries {
						k.hash(state);
						hash_value(v, state);
					}
				}
			}
		}

		hash_value(&self.value, state);
	}
}

impl<'a> std::hash::Hash for BorrowedValueWrapper<'a> {
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		fn hash_value<H: std::hash::Hasher>(value: &serde_json_borrow::Value<'_>, state: &mut H) {
			match value {
				serde_json_borrow::Value::Null => {
					0u8.hash(state);
				}
				serde_json_borrow::Value::Bool(b) => {
					1u8.hash(state);
					b.hash(state);
				}
				serde_json_borrow::Value::Number(n) => {
					2u8.hash(state);
					if let Some(i) = n.as_i64() {
						i.hash(state);
					} else if let Some(u) = n.as_u64() {
						u.hash(state);
					} else if let Some(f) = n.as_f64() {
						f.to_bits().hash(state);
					}
				}
				serde_json_borrow::Value::Str(s) => {
					3u8.hash(state);
					s.hash(state);
				}
				serde_json_borrow::Value::Array(arr) => {
					4u8.hash(state);
					for item in arr {
						hash_value(item, state);
					}
				}
				serde_json_borrow::Value::Object(obj) => {
					5u8.hash(state);
					let mut entries = obj.iter().collect::<Vec<_>>();
					entries.sort_by_key(|&(k, _)| k);
					for (k, v) in entries {
						k.hash(state);
						hash_value(v, state);
					}
				}
			}
		}

		hash_value(&self.value, state);
	}
}

/// Reference-counted copy-on-write container for any Variant-implementing value.
#[derive(Facet, Clone, dynex::PartialEqFix)]
pub struct ZVariant {
	#[facet(opaque)]
	value: Arc<dyn Variant>
}

impl Deref for ZVariant {
	type Target = dyn Variant;

	fn deref(&self) -> &Self::Target {
		&*self.value
	}
}

impl DerefMut for ZVariant {
	fn deref_mut(&mut self) -> &mut Self::Target {
		let mut s = self;
		polonius!(|s| -> &'polonius mut Self::Target {
			if let Some(value) = Arc::get_mut(&mut s.value) {
				polonius_return!(value)
			}
		});

		s.value = s.value.clone_underlying();
		Arc::get_mut(&mut s.value).expect("Exclusive access to new Arc is unavailable")
	}
}

impl From<Arc<dyn Variant>> for ZVariant {
	fn from(value: Arc<dyn Variant>) -> Self {
		Self { value }
	}
}

impl ZVariant {
	pub fn new<T: Variant>(value: T) -> Self {
		Self { value: Arc::new(value) }
	}

	/// Determine whether the stored Variant value is a valid type for this game version.
	pub fn is_valid(&self) -> bool {
		VARIANT_TYPES.contains_key(&self.any_type())
	}

	pub fn into_inner(self) -> Arc<dyn Variant> {
		self.value
	}
}

impl Debug for ZVariant {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		f.debug_tuple("ZVariant").field(&self.value).finish()
	}
}

impl Aligned for ZVariant {
	const ALIGNMENT: usize = 8;
}

impl Bin1Serialize for ZVariant {
	fn alignment(&self) -> usize {
		Self::ALIGNMENT
	}

	fn write(&self, ser: &mut Bin1Serializer) -> Result<(), SerializeError> {
		let type_id = self.variant_type();

		if type_id == "void" {
			ser.write_type(type_id);
			ser.write_pointer(u64::MAX); // void type has no data
		} else {
			ser.write_type(type_id);
			let pointer_id = Arc::as_ptr(&self.value) as *const () as u64 | 0xBEEF000000000000;
			ser.write_pointer(pointer_id);
		}

		Ok(())
	}

	fn resolve(&self, ser: &mut Bin1Serializer) -> Result<(), SerializeError> {
		if self.variant_type() != "void" {
			let pointer_id = Arc::as_ptr(&self.value) as *const () as u64 | 0xBEEF000000000000;
			ser.write_pointee(pointer_id, None, &*self.value)?;
		}

		Ok(())
	}
}

impl Bin1Deserialize for ZVariant {
	const SIZE: usize = 8 * 2;

	#[try_fn]
	fn read(de: &mut Bin1Deserializer) -> Result<Self, DeserializeError> {
		let type_id = de.read_type()?;

		if type_id == "void" {
			de.seek_relative(8)?; // skip pointer
			Self::new(())
		} else {
			de.read_variant_ptr(|de| {
				DESERIALIZERS
					.get(type_id.as_str())
					.ok_or_else(|| DeserializeError::UnknownType(type_id.to_string()))?
					.deserialize_bin1(&type_id, de)
			})?
			.into()
		}
	}
}

impl Serialize for ZVariant {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
		S::Error: serde::ser::Error
	{
		let mut ser = serializer.serialize_struct("ZVariant", 2)?;
		ser.serialize_field("$type", &self.variant_type())?;
		ser.serialize_field(
			"$val",
			&self.value.to_serde().map_err(<S::Error as serde::ser::Error>::custom)?
		)?;
		ser.end()
	}
}

impl<'de> Deserialize<'de> for ZVariant {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>
	{
		#[derive(Deserialize)]
		#[serde(field_identifier)]
		enum Field {
			#[serde(rename = "$type")]
			Ty,
			#[serde(rename = "$val")]
			Val
		}

		struct VariantVisitor;

		impl<'de> serde::de::Visitor<'de> for VariantVisitor {
			type Value = ZVariant;

			fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
				formatter.write_str("struct ZVariant")
			}

			fn visit_seq<V>(self, mut seq: V) -> Result<ZVariant, V::Error>
			where
				V: serde::de::SeqAccess<'de>
			{
				let type_id: &'de str = seq
					.next_element()?
					.ok_or_else(|| serde::de::Error::invalid_length(0, &self))?;

				let value = seq
					.next_element()?
					.ok_or_else(|| serde::de::Error::invalid_length(1, &self))?;

				if let Some(deserializer) = DESERIALIZERS.get(&type_id) {
					let value = BorrowedValueWrapper { value };

					if let Some(variant) = VARIANT_POOL.pin().get(&value)
						&& let Some(variant) = variant.upgrade()
					{
						return Ok(variant.into());
					}

					let variant = deserializer
						.deserialize_serde(&type_id, value.value.to_owned().into())
						.map_err(serde::de::Error::custom)?;

					VARIANT_POOL.pin().insert(
						ValueWrapper {
							value: value.value.into()
						},
						Arc::downgrade(&variant)
					);

					Ok(variant.into())
				} else {
					Err(serde::de::Error::custom(format!("unknown type ID: {}", type_id)))
				}
			}

			fn visit_map<V>(self, mut map: V) -> Result<ZVariant, V::Error>
			where
				V: serde::de::MapAccess<'de>
			{
				let mut type_id = None::<&'de str>;
				let mut value = None;

				while let Some(key) = map.next_key()? {
					match key {
						Field::Ty => {
							if type_id.is_some() {
								return Err(serde::de::Error::duplicate_field("$type"));
							}
							type_id = Some(map.next_value()?);
						}

						Field::Val => {
							if value.is_some() {
								return Err(serde::de::Error::duplicate_field("$val"));
							}
							value = Some(map.next_value()?);
						}
					}
				}

				let type_id = type_id.ok_or_else(|| serde::de::Error::missing_field("$type"))?;
				let value = value.ok_or_else(|| serde::de::Error::missing_field("$val"))?;

				if let Some(deserializer) = DESERIALIZERS.get(&type_id) {
					let value = BorrowedValueWrapper { value };

					if let Some(variant) = VARIANT_POOL.pin().get(&value)
						&& let Some(variant) = variant.upgrade()
					{
						return Ok(variant.into());
					}

					let variant = deserializer
						.deserialize_serde(&type_id, value.value.to_owned().into())
						.map_err(serde::de::Error::custom)?;

					VARIANT_POOL.pin().insert(
						ValueWrapper {
							value: value.value.into()
						},
						Arc::downgrade(&variant)
					);

					Ok(variant.into())
				} else {
					Err(serde::de::Error::custom(format!("unknown type ID: {}", type_id)))
				}
			}
		}

		deserializer.deserialize_struct("ZVariant", &["$type", "$val"], VariantVisitor)
	}
}

impl StaticVariant for ZVariant {
	const TYPE_ID: &'static str = "ZVariant";
}

impl Variant for ZVariant {
	fn type_id(&self) -> EcoString {
		Self::TYPE_ID.into()
	}

	fn to_serde(&self) -> Result<serde_json::Value, serde_json::Error> {
		serde_json::to_value(self)
	}
}

impl StaticVariant for Vec<ZVariant> {
	const TYPE_ID: &'static str = "TArray<ZVariant>";
}

submit!(ZVariant);

submit!(u8);
submit!(u16);
submit!(u32);
submit!(u64);
submit!(i8);
submit!(i16);
submit!(i32);
submit!(i64);
submit!(f32);
submit!(f64);
submit!(bool);
submit!(());
submit_nofacet!(EcoString);
submit!(ZRuntimeResourceID);

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Bin1Serialize, Bin1Deserialize, Facet)]
pub struct SEntityTemplateProperty {
	#[serde(rename = "nPropertyID")]
	pub property_id: PropertyID,

	#[serde(rename = "value")]
	#[bin1(pad = 4)]
	pub value: ZVariant
}

impl StaticVariant for SEntityTemplateProperty {
	const TYPE_ID: &'static str = "SEntityTemplateProperty";
}

impl StaticVariant for Vec<SEntityTemplateProperty> {
	const TYPE_ID: &'static str = "TArray<SEntityTemplateProperty>";
}

impl Variant for SEntityTemplateProperty {
	fn type_id(&self) -> EcoString {
		Self::TYPE_ID.into()
	}

	fn to_serde(&self) -> Result<serde_json::Value, serde_json::Error> {
		serde_json::to_value(self)
	}
}

submit!(SEntityTemplateProperty);

impl StaticVariant for (EcoString, ZVariant) {
	const TYPE_ID: &'static str = "TPair<ZString,ZVariant>";
}

impl StaticVariant for Vec<(EcoString, ZVariant)> {
	const TYPE_ID: &'static str = "TArray<TPair<ZString,ZVariant>>";
}

submit_nofacet!((EcoString, ZVariant));
