use std::{
	collections::HashMap,
	fmt::Debug,
	io::{Cursor, Seek, SeekFrom}
};

use byteorder::{LittleEndian, ReadBytesExt};
use ecow::EcoString;
use thiserror::Error;
use tryvial::try_fn;

pub mod impls;

pub use hitman_bin1_derive::Bin1Serialize;

#[derive(Error, Debug)]
pub enum SerializeError {
	#[error("I/O error: {0}")]
	Io(#[from] std::io::Error)
}

pub trait Aligned {
	const ALIGNMENT: usize;
}

pub trait Bin1Serialize {
	fn alignment(&self) -> usize;

	fn write(&self, ser: &mut Bin1Serializer) -> Result<(), SerializeError>;

	fn write_aligned(&self, ser: &mut Bin1Serializer) -> Result<(), SerializeError> {
		ser.align_to(self.alignment());
		self.write(ser)?;
		ser.align_to(self.alignment());
		Ok(())
	}

	#[allow(unused)]
	fn resolve(&self, ser: &mut Bin1Serializer) -> Result<(), SerializeError> {
		Ok(())
	}
}

pub struct Bin1Serializer {
	buffer: Vec<u8>,

	/// Map of pointer IDs to where their data has been written.
	offsets: HashMap<u64, u64, rapidhash::fast::RandomState>,

	/// Offsets to pointers that need to be patched.
	pointers: Vec<u32>,

	/// Offsets to ZRuntimeResourceIDs
	runtime_resource_ids: Vec<u32>,

	/// Offsets to TResourcePtrs
	resource_ptrs: Vec<u32>,

	/// Offsets to STypeIDs
	type_ids: Vec<u32>,
	type_names: Vec<EcoString>,

	rrids_segment: bool
}

impl Default for Bin1Serializer {
	fn default() -> Self {
		Self {
			buffer: vec![],
			offsets: Default::default(),
			pointers: vec![],
			runtime_resource_ids: vec![],
			resource_ptrs: vec![],
			type_ids: vec![],
			type_names: vec![],
			rrids_segment: true
		}
	}
}

impl Bin1Serializer {
	pub fn new() -> Self {
		Default::default()
	}

	pub fn with_rrids_segment(mut self, enabled: bool) -> Self {
		self.rrids_segment = enabled;
		self
	}

	pub fn align_to(&mut self, alignment: usize) {
		let current_len = self.buffer.len();
		let padding = alignment - (current_len % alignment);
		if padding < alignment {
			self.buffer.extend(vec![0; padding]);
		}
	}

	pub fn write_unaligned(&mut self, data: &[u8]) {
		self.buffer.extend_from_slice(data);
	}

	pub fn write_aligned(&mut self, data: &[u8], alignment: usize) {
		self.align_to(alignment);
		self.buffer.extend_from_slice(data);
		self.align_to(alignment);
	}

	pub fn write_pointer(&mut self, pointer_id: u64) {
		self.align_to(8);
		self.pointers.push(self.buffer.len() as u32);
		self.buffer.extend_from_slice(&pointer_id.to_le_bytes());
	}

	pub fn write_pointee<T: Bin1Serialize + ?Sized>(
		&mut self,
		pointer_id: u64,
		end_pointer_id: Option<u64>,
		data: &T
	) -> Result<(), SerializeError> {
		if self.offsets.contains_key(&pointer_id) {
			return Ok(());
		}

		self.align_to(8); // align to "serializer alignment"
		self.align_to(data.alignment());
		self.register_pointee(pointer_id);

		data.write(self)?;
		if let Some(end_pointer_id) = end_pointer_id {
			// Register the end pointer as here, at the end of the pointee data
			self.register_pointee(end_pointer_id);
		}

		data.resolve(self)?;

		Ok(())
	}

	/// Register a pointer as referring to the current location in the serialisation buffer.
	pub fn register_pointee(&mut self, pointer_id: u64) {
		self.offsets.insert(pointer_id, self.buffer.len() as u64);
	}

	pub fn write_type(&mut self, type_name: EcoString) {
		self.align_to(8);
		self.type_ids.push(self.buffer.len() as u32);

		if let Some(existing) = self.type_names.iter().position(|name| *name == type_name) {
			self.buffer.extend_from_slice(&(existing as u64).to_le_bytes());
		} else {
			self.buffer
				.extend_from_slice(&(self.type_names.len() as u64).to_le_bytes());
			self.type_names.push(type_name);
		}
	}

	pub fn write_runtime_resource_id(&mut self, high: u32, low: u32) {
		self.runtime_resource_ids.push(self.buffer.len() as u32);
		self.write_unaligned(&high.to_le_bytes());
		self.write_unaligned(&low.to_le_bytes());
	}

	pub fn write_resource_ptr(&mut self, high: u32, low: u32) {
		self.resource_ptrs.push(self.buffer.len() as u32);
		self.write_unaligned(&high.to_le_bytes());
		self.write_unaligned(&low.to_le_bytes());
	}

	#[try_fn]
	#[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
	pub fn finalise(mut self) -> Result<Vec<u8>, SerializeError> {
		self.align_to(8);

		let mut cursor = Cursor::new(self.buffer);

		let mut segments: Vec<(u32, Vec<u8>)> = vec![];

		// Rebased pointers segment
		if !self.pointers.is_empty() {
			let mut segment = (self.pointers.len() as u32).to_le_bytes().to_vec();

			for offset in &self.pointers {
				segment.extend_from_slice(&offset.to_le_bytes());
			}

			segments.push((0x12EBA5ED, segment));
		}

		// STypeIDs segment
		if !self.type_ids.is_empty() {
			let mut segment = (self.type_ids.len() as u32).to_le_bytes().to_vec();

			for offset in self.type_ids {
				segment.extend_from_slice(&offset.to_le_bytes());
			}

			segment.extend_from_slice(&(self.type_names.len() as u32).to_le_bytes());

			for (idx, name) in self.type_names.into_iter().enumerate() {
				let padding = 4 - (segment.len() % 4);
				if padding < 4 {
					segment.extend(vec![0; padding]);
				}

				segment.extend_from_slice(&(idx as u32).to_le_bytes());
				segment.extend_from_slice(&u32::MAX.to_le_bytes());

				segment.extend_from_slice(&(name.len() as u32 + 1).to_le_bytes());
				segment.extend_from_slice(name.as_bytes());
				segment.push(0);
			}

			segments.push((0x3989BF9F, segment));
		}

		// RuntimeResourceIDs segment
		if self.rrids_segment && !self.runtime_resource_ids.is_empty() {
			let mut segment = (self.runtime_resource_ids.len() as u32).to_le_bytes().to_vec();

			for offset in self.runtime_resource_ids {
				segment.extend_from_slice(&offset.to_le_bytes());
			}

			segments.push((0x578FBCEE, segment));
		}

		// ResourcePtrs segment
		if !self.resource_ptrs.is_empty() {
			let mut segment = (self.resource_ptrs.len() as u32).to_le_bytes().to_vec();

			for offset in self.resource_ptrs {
				segment.extend_from_slice(&offset.to_le_bytes());
			}

			segments.push((0x64603664, segment));
		}

		for offset in self.pointers.drain(..) {
			cursor.seek(SeekFrom::Start(offset as u64))?;
			let pointer_id = cursor.read_u64::<LittleEndian>()?;
			if pointer_id != u64::MAX {
				cursor.get_mut()[offset as usize..offset as usize + 8]
					.copy_from_slice(&self.offsets[&pointer_id].to_le_bytes());
			}
		}

		let buffer = cursor.into_inner();

		let mut data = vec![];
		data.extend_from_slice(b"BIN1");
		data.push(0); // padding
		data.push(8); // alignment
		data.push(segments.len() as u8);
		data.push(0);
		data.extend_from_slice(&(buffer.len() as u32).to_be_bytes());
		data.extend_from_slice(&0u32.to_le_bytes());

		data.extend_from_slice(&buffer);

		for segment in segments {
			data.extend_from_slice(&segment.0.to_le_bytes());
			data.extend_from_slice(&(segment.1.len() as u32).to_le_bytes());
			data.extend_from_slice(&segment.1);
		}

		data
	}

	pub fn serialize(mut self, value: &impl Bin1Serialize) -> Result<Vec<u8>, SerializeError> {
		value.write(&mut self)?;
		value.resolve(&mut self)?;
		self.finalise()
	}
}
