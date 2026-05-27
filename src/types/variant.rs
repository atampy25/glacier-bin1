use std::{
	fmt::Debug,
	ops::{Deref, DerefMut},
	sync::Arc
};

use const_format::concatcp;
use ecow::{EcoString, eco_format};
use serde::{Deserialize, Serialize};
use tryvial::try_fn;

use crate::{
	de::{Bin1Deserialize, Bin1Deserializer, DeserializeError},
	ser::{Aligned, Bin1Serialize, Bin1Serializer, SerializeError}
};

pub trait StaticVariant {
	const TYPE_ID: &'static str;
}

#[dynex::dyn_trait]
pub trait Variant: VariantArc + Bin1Serialize + Send + Sync + Debug + Clone + PartialEq {
	fn type_id(&self) -> EcoString;

	/// Serialise this variant value into a serde_json Value. Does not include type information.
	fn to_serde(&self) -> Result<serde_json::Value, serde_json::Error>;

	/// Attempt to downcast this value as a Vec of Variants (which will all be of the same element type), allowing for generic operations on individual elements where the main Vec<T> type is unimportant.
	/// If this value is a Vec<T: Variant>, returns Some(Vec<&dyn Variant>), else returns None.
	fn as_vec(&self) -> Option<Vec<&dyn Variant>> {
		None
	}
}

pub trait VariantArc {
	fn into_inner_boxed_dyn(self: Arc<Self>) -> Option<Box<dyn Variant>>;
	fn unwrap_or_clone_boxed_dyn(self: Arc<Self>) -> Box<dyn Variant>;
	fn clone_underlying(&self) -> Arc<dyn Variant>;
}

impl<T: Variant + Clone> VariantArc for T {
	fn into_inner_boxed_dyn(self: Arc<Self>) -> Option<Box<dyn Variant>> {
		Arc::into_inner(self).map(|x| Box::new(x) as _)
	}

	fn unwrap_or_clone_boxed_dyn(self: Arc<Self>) -> Box<dyn Variant> {
		Box::new(Arc::unwrap_or_clone(self))
	}

	fn clone_underlying(&self) -> Arc<dyn Variant> {
		Arc::new(self.clone())
	}
}

impl dyn Variant {
	pub fn variant_type(&self) -> EcoString {
		Variant::type_id(self)
	}

	/// Get the std::any::TypeId of the underlying type.
	pub fn any_type(&self) -> std::any::TypeId {
		std::any::Any::type_id(self)
	}

	pub fn is<T: Variant>(&self) -> bool {
		self.as_any().is::<T>()
	}

	pub fn into_boxed<T: Variant>(self: Box<dyn Variant>) -> Option<Box<T>> {
		self.as_any_box().downcast().ok()
	}

	pub fn into_unboxed<T: Variant>(self: Box<dyn Variant>) -> Option<T> {
		self.as_any_box().downcast().ok().map(|x| *x)
	}

	/// The first Option is the result of obtaining exclusive access to the Arc. The second Option is the result of downcasting into T.
	pub fn into_inner_boxed<T: Variant>(self: Arc<dyn Variant>) -> Option<Option<Box<T>>> {
		self.into_inner_boxed_dyn().map(|x| x.as_any_box().downcast().ok())
	}

	/// The first Option is the result of obtaining exclusive access to the Arc. The second Option is the result of downcasting into T.
	pub fn into_inner_unboxed<T: Variant>(self: Arc<dyn Variant>) -> Option<Option<T>> {
		self.into_inner_boxed_dyn()
			.map(|x| x.as_any_box().downcast().ok().map(|x| *x))
	}

	pub fn unwrap_or_clone_boxed<T: Variant>(self: Arc<dyn Variant>) -> Option<Box<T>> {
		self.unwrap_or_clone_boxed_dyn().as_any_box().downcast().ok()
	}

	pub fn unwrap_or_clone_unboxed<T: Variant>(self: Arc<dyn Variant>) -> Option<T> {
		self.unwrap_or_clone_boxed_dyn()
			.as_any_box()
			.downcast()
			.ok()
			.map(|x| *x)
	}

	pub fn as_ref<T: Variant>(&self) -> Option<&T> {
		self.as_any().downcast_ref()
	}

	pub fn as_mut<T: Variant>(&mut self) -> Option<&mut T> {
		self.as_any_mut().downcast_mut()
	}
}

macro_rules! impl_primitive {
	($ty:ty, $type_id:literal) => {
		impl StaticVariant for $ty {
			const TYPE_ID: &'static str = $type_id;
		}

		impl Variant for $ty {
			fn type_id(&self) -> EcoString {
				Self::TYPE_ID.into()
			}

			fn to_serde(&self) -> Result<serde_json::Value, serde_json::Error> {
				Ok((*self).into())
			}
		}

		impl StaticVariant for Vec<$ty> {
			const TYPE_ID: &'static str = concatcp!("TArray<", $type_id, ">");
		}
	};
}

impl_primitive!(u8, "uint8");
impl_primitive!(u16, "uint16");
impl_primitive!(u32, "uint32");
impl_primitive!(u64, "uint64");

impl_primitive!(i8, "int8");
impl_primitive!(i16, "int16");
impl_primitive!(i32, "int32");
impl_primitive!(i64, "int64");

impl_primitive!(f32, "float32");
impl_primitive!(f64, "float64");

impl_primitive!(bool, "bool");

impl StaticVariant for () {
	const TYPE_ID: &'static str = "void";
}

impl StaticVariant for Vec<()> {
	const TYPE_ID: &'static str = "TArray<void>";
}

impl Variant for () {
	fn type_id(&self) -> EcoString {
		Self::TYPE_ID.into()
	}

	fn to_serde(&self) -> Result<serde_json::Value, serde_json::Error> {
		Ok(serde_json::Value::Null)
	}
}

impl<
	T: Bin1Serialize + Aligned + Serialize + StaticVariant + Send + Sync + Clone + Debug + PartialEq + 'static,
	U: Bin1Serialize + Aligned + Serialize + StaticVariant + Send + Sync + Clone + Debug + PartialEq + 'static
> Variant for (T, U)
{
	fn type_id(&self) -> EcoString {
		eco_format!("TPair<{},{}>", T::TYPE_ID, U::TYPE_ID)
	}

	fn to_serde(&self) -> Result<serde_json::Value, serde_json::Error> {
		serde_json::to_value(self)
	}
}

impl<
	T: Bin1Serialize + Aligned + Serialize + StaticVariant + Variant + Send + Sync + Clone + Debug + PartialEq + 'static
> Variant for Vec<T>
{
	fn type_id(&self) -> EcoString {
		eco_format!("TArray<{}>", T::TYPE_ID)
	}

	fn to_serde(&self) -> Result<serde_json::Value, serde_json::Error> {
		serde_json::to_value(self)
	}

	fn as_vec(&self) -> Option<Vec<&dyn Variant>> {
		Some(self.iter().map(|x| x as &dyn Variant).collect())
	}
}

impl StaticVariant for EcoString {
	const TYPE_ID: &'static str = "ZString";
}

impl StaticVariant for Vec<EcoString> {
	const TYPE_ID: &'static str = "TArray<ZString>";
}

impl Variant for EcoString {
	fn type_id(&self) -> EcoString {
		Self::TYPE_ID.into()
	}

	fn to_serde(&self) -> Result<serde_json::Value, serde_json::Error> {
		Ok(serde_json::Value::String(self.as_str().into()))
	}
}

/// An arbitrary string, serialised and deserialised as a BIN1 type ID.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TypeID(pub EcoString);

impl Deref for TypeID {
	type Target = EcoString;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl DerefMut for TypeID {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

impl Aligned for TypeID {
	const ALIGNMENT: usize = 8;
}

impl Bin1Serialize for TypeID {
	fn alignment(&self) -> usize {
		Self::ALIGNMENT
	}

	#[try_fn]
	fn write(&self, ser: &mut Bin1Serializer) -> Result<(), SerializeError> {
		ser.write_type(self.0.to_owned());
	}
}

impl Bin1Deserialize for TypeID {
	const SIZE: usize = 8;

	#[try_fn]
	fn read(de: &mut Bin1Deserializer) -> Result<Self, DeserializeError> {
		Self(de.read_type()?)
	}
}
