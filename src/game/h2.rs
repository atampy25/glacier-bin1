#![allow(non_camel_case_types)]

#[linkme::distributed_slice]
static VARIANT_TYPES_H2: [(std::any::TypeId, &'static str, Option<&'static facet::Shape>)];

#[static_init::dynamic]
pub static VARIANT_TYPES: HashMap<std::any::TypeId, (&'static str, Option<&'static facet::Shape>)> = VARIANT_TYPES_H2
	.into_iter()
	.map(|&(ty, name, shape)| (ty, (name, shape)))
	.collect();

#[linkme::distributed_slice]
static VARIANT_DESERIALIZERS_H2: [&'static dyn DeserializeVariant];

#[static_init::dynamic]
pub static DESERIALIZERS: HashMap<&'static str, &'static dyn DeserializeVariant> =
	VARIANT_DESERIALIZERS_H2.iter().map(|&x| (x.type_id(), x)).collect();

macro_rules! submit_nofacet {
	($ty:ty) => {
		mident::mident! {
			#[linkme::distributed_slice(VARIANT_TYPES_H2)]
			static #concat(#flatten($ty) _Ty): (std::any::TypeId, &'static str, Option<&'static facet::Shape>)
				= (std::any::TypeId::of::<$ty>(), <$ty>::TYPE_ID, None);

			#[linkme::distributed_slice(VARIANT_DESERIALIZERS_H2)]
			static #concat(#flatten($ty) _De): &'static dyn DeserializeVariant
				= &VariantDeserializer::<$ty>::new() as &dyn DeserializeVariant;

			#[linkme::distributed_slice(VARIANT_DESERIALIZERS_H2)]
			static #concat(#flatten($ty) _Vec_De): &'static dyn DeserializeVariant
				= &VariantDeserializer::<Vec<$ty>>::new() as &dyn DeserializeVariant;
		}
	};
}

macro_rules! submit {
	($ty:ty) => {
		mident::mident! {
			#[linkme::distributed_slice(VARIANT_TYPES_H2)]
			static #concat(#flatten($ty) _Ty): (std::any::TypeId, &'static str, Option<&'static facet::Shape>)
				= (std::any::TypeId::of::<$ty>(), <$ty>::TYPE_ID, Some(&<$ty>::SHAPE));

			#[linkme::distributed_slice(VARIANT_DESERIALIZERS_H2)]
			static #concat(#flatten($ty) _De): &'static dyn DeserializeVariant
				= &VariantDeserializer::<$ty>::new() as &dyn DeserializeVariant;

			#[linkme::distributed_slice(VARIANT_DESERIALIZERS_H2)]
			static #concat(#flatten($ty) _Vec_De): &'static dyn DeserializeVariant
				= &VariantDeserializer::<Vec<$ty>>::new() as &dyn DeserializeVariant;
		}
	};
}

include!("variant_impl.rs");

include!(concat!(env!("OUT_DIR"), "/h2.rs"));
