use std::{
	borrow::Cow,
	fmt::Debug,
	ops::{Deref, DerefMut},
	sync::Arc
};

use ecow::EcoString;
use facet::Facet;
use serde::{Deserialize, Serialize};
use tryvial::try_fn;

pub use crate::__impl_variant as impl_variant;
use crate::{
	de::{Bin1Deserialize, Bin1Deserializer, DeserializeError},
	ser::{Aligned, Bin1Serialize, Bin1Serializer, SerializeError},
	types::{
		repository::ZRepositoryID,
		resource::{TResourcePtr, ZRuntimeResourceID}
	}
};

pub trait StaticVariant {
	const TYPE_ID: &'static str;
}

#[dynex::dyn_trait]
pub trait Variant: VariantArc + Bin1Serialize + Send + Sync + Debug + Clone + PartialEq {
	fn type_id(&self) -> &'static str;

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
	pub fn variant_type(&self) -> &'static str {
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
			fn type_id(&self) -> &'static str {
				Self::TYPE_ID
			}

			fn to_serde(&self) -> Result<serde_json::Value, serde_json::Error> {
				Ok((*self).into())
			}
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

impl Variant for () {
	fn type_id(&self) -> &'static str {
		Self::TYPE_ID
	}

	fn to_serde(&self) -> Result<serde_json::Value, serde_json::Error> {
		Ok(serde_json::Value::Null)
	}
}

const CONST_STR_BUF_SIZE: usize = 128;

const fn make_array_ty(contained_ty: &str) -> ([u8; CONST_STR_BUF_SIZE], usize) {
	const START: &[u8] = b"TArray<";
	const END: &[u8] = b">";

	if START.len() + contained_ty.len() + END.len() > CONST_STR_BUF_SIZE {
		panic!("type ID is too long to fit in buffer, hitman-bin1 needs to be updated");
	}

	let mut buf = [0u8; CONST_STR_BUF_SIZE];
	let mut pos = 0;

	while pos != START.len() {
		buf[pos] = START[pos];
		pos += 1;
	}

	let start_pos = pos;
	while pos != start_pos + contained_ty.len() {
		buf[pos] = contained_ty.as_bytes()[pos - start_pos];
		pos += 1;
	}

	let start_pos = pos;
	while pos != start_pos + END.len() {
		buf[pos] = END[pos - start_pos];
		pos += 1;
	}

	(buf, pos)
}

impl<T: StaticVariant> StaticVariant for Vec<T> {
	const TYPE_ID: &'static str =
		match std::str::from_utf8(make_array_ty(T::TYPE_ID).0.split_at(make_array_ty(T::TYPE_ID).1).0) {
			Ok(s) => s,
			Err(_) => panic!("contained type ID is invalid")
		};
}

impl<T: Variant + StaticVariant + Serialize + Aligned + PartialEq + Clone + Bin1Deserialize> Variant for Vec<T> {
	fn type_id(&self) -> &'static str {
		Self::TYPE_ID
	}

	fn to_serde(&self) -> Result<serde_json::Value, serde_json::Error> {
		serde_json::to_value(self)
	}

	fn as_vec(&self) -> Option<Vec<&dyn Variant>> {
		Some(self.iter().map(|x| x as &dyn Variant).collect())
	}
}

const fn make_pair_ty(ty_1: &str, ty_2: &str) -> ([u8; CONST_STR_BUF_SIZE], usize) {
	const START: &[u8] = b"TPair<";
	const SEP: &[u8] = b",";
	const END: &[u8] = b">";

	if START.len() + ty_1.len() + SEP.len() + ty_2.len() + END.len() > CONST_STR_BUF_SIZE {
		panic!("type ID is too long to fit in buffer, hitman-bin1 needs to be updated");
	}

	let mut buf = [0u8; CONST_STR_BUF_SIZE];
	let mut pos = 0;

	while pos != START.len() {
		buf[pos] = START[pos];
		pos += 1;
	}

	let start_pos = pos;
	while pos != start_pos + ty_1.len() {
		buf[pos] = ty_1.as_bytes()[pos - start_pos];
		pos += 1;
	}

	let start_pos = pos;
	while pos != start_pos + SEP.len() {
		buf[pos] = SEP[pos - start_pos];
		pos += 1;
	}

	let start_pos = pos;
	while pos != start_pos + ty_2.len() {
		buf[pos] = ty_2.as_bytes()[pos - start_pos];
		pos += 1;
	}

	let start_pos = pos;
	while pos != start_pos + END.len() {
		buf[pos] = END[pos - start_pos];
		pos += 1;
	}

	(buf, pos)
}

impl<T: StaticVariant, U: StaticVariant> StaticVariant for (T, U) {
	const TYPE_ID: &'static str = match std::str::from_utf8(
		make_pair_ty(T::TYPE_ID, U::TYPE_ID)
			.0
			.split_at(make_pair_ty(T::TYPE_ID, U::TYPE_ID).1)
			.0
	) {
		Ok(s) => s,
		Err(_) => panic!("contained type ID is invalid")
	};
}

impl<
	T: Variant + StaticVariant + Serialize + Aligned + PartialEq + Clone,
	U: Variant + StaticVariant + Serialize + Aligned + PartialEq + Clone
> Variant for (T, U)
{
	fn type_id(&self) -> &'static str {
		Self::TYPE_ID
	}

	fn to_serde(&self) -> Result<serde_json::Value, serde_json::Error> {
		serde_json::to_value(self)
	}
}

#[doc(hidden)]
#[macro_export]
macro_rules! __impl_variant {
	($ty:ty, $type_id:literal) => {
		impl StaticVariant for $ty {
			const TYPE_ID: &'static str = $type_id;
		}

		impl Variant for $ty {
			fn type_id(&self) -> &'static str {
				Self::TYPE_ID
			}

			fn to_serde(&self) -> Result<serde_json::Value, serde_json::Error> {
				serde_json::to_value(self)
			}
		}
	};
}

impl_variant!(EcoString, "ZString");
impl_variant!(ZRepositoryID, "ZRepositoryID");
impl_variant!(ZRuntimeResourceID, "ZRuntimeResourceID");
impl_variant!(TResourcePtr, "TResourcePtr");

/// An arbitrary string, serialised and deserialised as a BIN1 type ID.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Facet)]
pub struct TypeID(#[facet(opaque, proxy = String)] pub EcoString);

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
		ser.write_type(Cow::Owned(self.0.to_owned().into()));
	}
}

impl Bin1Deserialize for TypeID {
	const SIZE: usize = 8;

	#[try_fn]
	fn read(de: &mut Bin1Deserializer) -> Result<Self, DeserializeError> {
		Self(de.read_type()?.into())
	}
}
