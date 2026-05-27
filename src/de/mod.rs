use std::{
	any::Any,
	collections::HashMap,
	io::{Cursor, Read, Seek, SeekFrom},
	sync::Arc
};

use byteorder::{BigEndian, LittleEndian, ReadBytesExt};
use ecow::EcoString;
use thiserror::Error;
use tryvial::try_fn;

use crate::{ser::Aligned, types::variant::Variant};

pub mod impls;

pub use hitman_bin1_derive::Bin1Deserialize;

#[derive(Error, Debug)]
pub enum DeserializeError {
	#[error("file is not in BIN1 format")]
	InvalidMagic,

	#[error("I/O error: {0}")]
	Io(#[from] std::io::Error),

	#[error("invalid string: {0}")]
	FromUtf8Error(#[from] std::string::FromUtf8Error),

	#[error("invalid string: {0}")]
	Utf8Error(#[from] std::str::Utf8Error),

	#[error("string length exceeded remaining file")]
	StringTooLarge,

	#[error("expected type {expected} but found {found}")]
	TypeMismatch { expected: &'static str, found: String },

	#[error("no such type ID with index {0}")]
	NoSuchTypeID(u64),

	#[error("unknown type {0}")]
	UnknownType(String),

	#[error("invalid enum value {0}")]
	InvalidEnumValue(i64)
}

pub struct Bin1Deserializer<'a> {
	buffer: Cursor<&'a [u8]>,

	parsed_strings: HashMap<u64, EcoString, rapidhash::fast::RandomState>,
	parsed_pointers: HashMap<u64, Box<dyn Any>, rapidhash::fast::RandomState>,
	parsed_variants: HashMap<u64, Arc<dyn Variant>, rapidhash::fast::RandomState>,

	type_names: HashMap<u32, EcoString, rapidhash::fast::RandomState>
}

pub trait Bin1Deserialize: Sized + Aligned {
	const SIZE: usize;

	fn read(de: &mut Bin1Deserializer) -> Result<Self, DeserializeError>;
}

impl<'a> Bin1Deserializer<'a> {
	pub fn new(data: &'a [u8]) -> Self {
		Self {
			buffer: Cursor::new(data),
			parsed_strings: Default::default(),
			parsed_pointers: Default::default(),
			parsed_variants: Default::default(),
			type_names: Default::default()
		}
	}

	#[try_fn]
	pub fn align_to(&mut self, alignment: usize) -> Result<(), DeserializeError> {
		self.buffer
			.seek_relative(((alignment - (self.buffer.position() as usize % alignment)) % alignment) as i64)?;
	}

	#[try_fn]
	#[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
	pub fn init(&mut self) -> Result<(), DeserializeError> {
		let mut magic = [0u8; 4];
		self.buffer.read_exact(&mut magic)?;

		if magic != *b"BIN1" {
			return Err(DeserializeError::InvalidMagic);
		}

		self.buffer.seek_relative(2)?;
		let segments_count = self.buffer.read_u8()?;
		self.buffer.seek_relative(1)?;
		let data_size = self.buffer.read_u32::<BigEndian>()?;
		self.buffer.seek_relative(4)?;
		let data_start = self.buffer.position();

		// Skip to STypeIDs segment
		self.buffer.seek_relative(data_size as i64)?;
		let mut skipped_segments = 0;
		while skipped_segments < segments_count && self.buffer.read_u32::<LittleEndian>()? != 0x3989BF9F {
			let segment_size = self.buffer.read_u32::<LittleEndian>()?;
			self.buffer.seek_relative(segment_size as i64)?;
			skipped_segments += 1;
		}

		if skipped_segments < segments_count {
			self.buffer.seek_relative(4)?; // skip segment size

			let offsets_count = self.buffer.read_u32::<LittleEndian>()?;
			self.buffer.seek_relative(offsets_count as i64 * 4)?; // skip past offsets

			let type_ids_start = self.buffer.position();
			let type_ids_count = self.buffer.read_u32::<LittleEndian>()?;
			for _ in 0..type_ids_count {
				// align to 4 within this segment
				self.buffer
					.seek_relative(((4 - ((self.buffer.position() - type_ids_start) as usize % 4)) % 4) as i64)?;

				let index = self.buffer.read_u32::<LittleEndian>()?;
				self.buffer.seek_relative(4)?;

				let len = self.buffer.read_u32::<LittleEndian>()?;
				let mut name = vec![0u8; len as usize - 1];
				self.buffer.read_exact(&mut name)?;
				let name = String::from_utf8(name)?;
				self.buffer.seek_relative(1)?;

				self.type_names.insert(index, name.into());
			}
		}

		self.buffer.seek(SeekFrom::Start(data_start))?;
	}

	#[try_fn]
	pub fn read_u8(&mut self) -> Result<u8, DeserializeError> {
		self.buffer.read_u8()?
	}

	#[try_fn]
	pub fn read_u16(&mut self) -> Result<u16, DeserializeError> {
		self.buffer.read_u16::<LittleEndian>()?
	}

	#[try_fn]
	pub fn read_u32(&mut self) -> Result<u32, DeserializeError> {
		self.buffer.read_u32::<LittleEndian>()?
	}

	#[try_fn]
	pub fn read_u64(&mut self) -> Result<u64, DeserializeError> {
		self.buffer.read_u64::<LittleEndian>()?
	}

	#[try_fn]
	pub fn read_i8(&mut self) -> Result<i8, DeserializeError> {
		self.buffer.read_i8()?
	}

	#[try_fn]
	pub fn read_i16(&mut self) -> Result<i16, DeserializeError> {
		self.buffer.read_i16::<LittleEndian>()?
	}

	#[try_fn]
	pub fn read_i32(&mut self) -> Result<i32, DeserializeError> {
		self.buffer.read_i32::<LittleEndian>()?
	}

	#[try_fn]
	pub fn read_i64(&mut self) -> Result<i64, DeserializeError> {
		self.buffer.read_i64::<LittleEndian>()?
	}

	#[try_fn]
	pub fn read_f32(&mut self) -> Result<f32, DeserializeError> {
		self.buffer.read_f32::<LittleEndian>()?
	}

	#[try_fn]
	pub fn read_f64(&mut self) -> Result<f64, DeserializeError> {
		self.buffer.read_f64::<LittleEndian>()?
	}

	pub fn position(&self) -> u64 {
		self.buffer.position()
	}

	#[try_fn]
	pub fn seek_from_start(&mut self, offset: u64) -> Result<u64, DeserializeError> {
		self.buffer.seek(SeekFrom::Start(offset))?
	}

	#[try_fn]
	pub fn seek_relative(&mut self, offset: i64) -> Result<(), DeserializeError> {
		self.buffer.seek_relative(offset)?
	}

	#[try_fn]
	pub fn read_type(&mut self) -> Result<EcoString, DeserializeError> {
		self.align_to(8)?;
		let id = self.buffer.read_u64::<LittleEndian>()?;

		self.type_names
			.get(&(id as u32))
			.cloned()
			.ok_or(DeserializeError::NoSuchTypeID(id))?
	}

	#[try_fn]
	pub fn read_zstring(&mut self) -> Result<EcoString, DeserializeError> {
		self.align_to(8)?;
		let len = self.buffer.read_u32::<LittleEndian>()? & 0xBFFFFFFF;
		self.align_to(8)?;
		let ptr = self.buffer.read_u64::<LittleEndian>()?;

		if let Some(parsed) = self.parsed_strings.get(&ptr) {
			parsed.clone()
		} else {
			let start = ptr as usize + 0x10;
			let mut result = EcoString::with_capacity(len as usize);
			result.push_str(str::from_utf8(
				self.buffer
					.get_ref()
					.get(start..start + len as usize)
					.ok_or(DeserializeError::StringTooLarge)?
			)?);
			result
		}
	}

	#[try_fn]
	pub fn read_pointer<T: 'static>(
		&mut self,
		parser: impl Fn(&mut Bin1Deserializer) -> Result<T, DeserializeError>
	) -> Result<Arc<T>, DeserializeError> {
		self.align_to(8)?;

		let ptr = self.buffer.read_u64::<LittleEndian>()?;

		if let Some(parsed) = self.parsed_pointers.get(&ptr) {
			parsed.downcast_ref::<Arc<T>>().unwrap().clone()
		} else {
			let pos = self.buffer.position();

			self.buffer.seek(SeekFrom::Start(ptr + 0x10))?;
			let result = parser(self)?;
			self.buffer.seek(SeekFrom::Start(pos))?;

			let result = Arc::new(result);
			self.parsed_pointers
				.insert(ptr, Box::new(result.clone()) as Box<dyn Any>);

			result
		}
	}

	#[try_fn]
	pub fn read_variant_ptr(
		&mut self,
		parser: impl Fn(&mut Bin1Deserializer) -> Result<Arc<dyn Variant>, DeserializeError>
	) -> Result<Arc<dyn Variant>, DeserializeError> {
		self.align_to(8)?;

		let ptr = self.buffer.read_u64::<LittleEndian>()?;

		if let Some(parsed) = self.parsed_variants.get(&ptr) {
			parsed.clone()
		} else {
			let pos = self.buffer.position();

			self.buffer.seek(SeekFrom::Start(ptr + 0x10))?;
			let result = parser(self)?;
			self.buffer.seek(SeekFrom::Start(pos))?;

			self.parsed_variants.insert(ptr, result.clone());

			result
		}
	}
}
