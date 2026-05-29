pub use hitman_bin1_core::de;
pub use hitman_bin1_core::ser;
pub use hitman_bin1_core::types;

pub mod game;

#[cfg_attr(feature = "tracing", tracing::instrument(skip_all, fields(type = std::any::type_name::<T>())))]
pub fn serialize<T: ser::Bin1Serialize>(data: &T) -> Result<Vec<u8>, ser::SerializeError> {
	ser::Bin1Serializer::new().serialize(data)
}

#[cfg_attr(feature = "tracing", tracing::instrument(skip_all, fields(type = std::any::type_name::<T>())))]
pub fn deserialize<T: de::Bin1Deserialize>(data: &[u8]) -> Result<T, de::DeserializeError> {
	let mut de = de::Bin1Deserializer::new(data);
	de.init()?;
	T::read(&mut de)
}
