use std::{borrow::Cow, collections::HashMap, fmt::Debug};

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
		Ok(())
	}

	#[allow(unused)]
	fn resolve(&self, ser: &mut Bin1Serializer) -> Result<(), SerializeError> {
		Ok(())
	}
}

pub struct Bin1Serializer {
	buffer: Vec<u8>,

	/// Map of pointer IDs to where their data has been written (relative to the data section).
	offsets: HashMap<u64, u64, rapidhash::fast::RandomState>,

	/// Offsets to pointers that need to be patched (relative to the buffer).
	pointers: Vec<u32>,

	/// Offsets to ZRuntimeResourceIDs (relative to the buffer)
	runtime_resource_ids: Vec<u32>,

	/// Offsets to TResourcePtrs (relative to the buffer)
	resource_ptrs: Vec<u32>,

	/// Offsets to STypeIDs (relative to the buffer)
	type_ids: Vec<u32>,
	type_names: Vec<Cow<'static, str>>,

	rrids_segment: bool,
	resource_ptrs_segment: bool
}

impl Default for Bin1Serializer {
	fn default() -> Self {
		Self {
			buffer: {
				let mut buffer = vec![];
				buffer.extend_from_slice(b"BIN1");
				buffer.push(0); // padding
				buffer.push(1); // alignment
				buffer.push(0); // number of segments, to be filled in later
				buffer.push(0);
				buffer.extend_from_slice(&0u32.to_be_bytes()); // data size, to be filled in later
				buffer.extend_from_slice(&0u32.to_le_bytes());
				buffer
			},
			offsets: Default::default(),
			pointers: vec![],
			runtime_resource_ids: vec![],
			resource_ptrs: vec![],
			type_ids: vec![],
			type_names: vec![],
			rrids_segment: true,
			resource_ptrs_segment: true
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

	pub fn with_resource_ptrs_segment(mut self, enabled: bool) -> Self {
		self.resource_ptrs_segment = enabled;
		self
	}

	pub fn align_to(&mut self, alignment: usize) {
		let padding = alignment - ((self.buffer.len() - 0x10) % alignment);
		if padding < alignment {
			self.buffer.extend(vec![0; padding]);
		}
	}

	pub fn position(&self) -> usize {
		self.buffer.len()
	}

	pub fn write_unaligned(&mut self, data: &[u8]) {
		self.buffer.extend_from_slice(data);
	}

	pub fn write_aligned(&mut self, data: &[u8], alignment: usize) {
		self.align_to(alignment);
		self.buffer.extend_from_slice(data);
	}

	pub fn write_pointer(&mut self, pointer_id: u64) {
		#[cfg(feature = "debug-log")]
		eprintln!("0x{:6X}: writing pointer {:X}", self.position(), pointer_id);

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

		#[cfg(feature = "debug-log")]
		eprintln!(
			"0x{:6X}: writing pointee for {:X}/{}",
			self.position(),
			pointer_id,
			end_pointer_id.map_or_else(|| "None".into(), |id| format!("{:X}", id))
		);

		self.align_to(data.alignment());
		self.register_pointee(pointer_id);

		data.write(self)?;
		if let Some(end_pointer_id) = end_pointer_id {
			// Register the end pointer as here, at the end of the pointee data
			self.register_pointee(end_pointer_id);
		}

		#[cfg(feature = "debug-log")]
		eprintln!(
			"0x{:6X}: resolving pointee for {:X}/{}",
			self.position(),
			pointer_id,
			end_pointer_id.map_or_else(|| "None".into(), |id| format!("{:X}", id))
		);
		data.resolve(self)?;

		Ok(())
	}

	/// Register a pointer as referring to the current location in the serialisation buffer.
	pub fn register_pointee(&mut self, pointer_id: u64) {
		self.offsets.insert(pointer_id, self.buffer.len() as u64 - 0x10);
	}

	pub fn write_type(&mut self, type_name: Cow<'static, str>) {
		#[cfg(feature = "debug-log")]
		eprintln!("0x{:6X}: writing type {type_name}", self.position());

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
		#[cfg(feature = "debug-log")]
		eprintln!("0x{:6X}: writing runtime resource ID", self.position());

		self.runtime_resource_ids.push(self.buffer.len() as u32);
		self.write_unaligned(&high.to_le_bytes());
		self.write_unaligned(&low.to_le_bytes());
	}

	pub fn write_resource_ptr(&mut self, high: u32, low: u32) {
		#[cfg(feature = "debug-log")]
		eprintln!("0x{:6X}: writing resource pointer", self.position());

		self.resource_ptrs.push(self.buffer.len() as u32);
		self.write_unaligned(&high.to_le_bytes());
		self.write_unaligned(&low.to_le_bytes());
	}

	#[try_fn]
	#[cfg_attr(feature = "tracing", tracing::instrument(skip_all))]
	pub fn finalise(mut self) -> Result<Vec<u8>, SerializeError> {
		for offset in &self.pointers {
			let offset = *offset as usize;
			let pointer_id = u64::from_le_bytes(self.buffer[offset..offset + 8].try_into().unwrap());
			if pointer_id != u64::MAX {
				self.buffer[offset..offset + 8].copy_from_slice(&self.offsets[&pointer_id].to_le_bytes());
			}
		}

		let data_size = self.buffer.len() as u32 - 0x10;
		self.buffer[8..12].copy_from_slice(&data_size.to_be_bytes());

		// Rebased pointers segment
		if !self.pointers.is_empty() {
			self.buffer[6] += 1;
			self.buffer.extend_from_slice(&0x12EBA5EDu32.to_le_bytes());
			self.buffer.extend_from_slice(&0u32.to_le_bytes());
			let segment_start = self.buffer.len();

			self.buffer
				.extend_from_slice(&(self.pointers.len() as u32).to_le_bytes());

			for offset in self.pointers {
				self.buffer.extend_from_slice(&(offset - 0x10).to_le_bytes());
			}

			let segment_len = self.buffer.len() - segment_start;
			self.buffer[segment_start - 4..segment_start].copy_from_slice(&(segment_len as u32).to_le_bytes());
		}

		// STypeIDs segment
		if !self.type_ids.is_empty() {
			self.buffer[6] += 1;
			self.buffer.extend_from_slice(&0x3989BF9Fu32.to_le_bytes());
			self.buffer.extend_from_slice(&0u32.to_le_bytes());
			let segment_start = self.buffer.len();

			self.buffer
				.extend_from_slice(&(self.type_ids.len() as u32).to_le_bytes());

			for offset in self.type_ids {
				self.buffer.extend_from_slice(&(offset - 0x10).to_le_bytes());
			}

			self.buffer
				.extend_from_slice(&(self.type_names.len() as u32).to_le_bytes());

			for (idx, name) in self.type_names.into_iter().enumerate() {
				let padding = 4 - ((self.buffer.len() - segment_start) % 4);
				if padding < 4 {
					self.buffer.extend(vec![0; padding]);
				}

				self.buffer.extend_from_slice(&(idx as u32).to_le_bytes());
				self.buffer.extend_from_slice(&u32::MAX.to_le_bytes());

				self.buffer.extend_from_slice(&(name.len() as u32 + 1).to_le_bytes());
				self.buffer.extend_from_slice(name.as_bytes());
				self.buffer.push(0);
			}

			let segment_len = self.buffer.len() - segment_start;
			self.buffer[segment_start - 4..segment_start].copy_from_slice(&(segment_len as u32).to_le_bytes());
		}

		// RuntimeResourceIDs segment
		if self.rrids_segment && !self.runtime_resource_ids.is_empty() {
			self.buffer[6] += 1;
			self.buffer.extend_from_slice(&0x578FBCEEu32.to_le_bytes());
			self.buffer.extend_from_slice(&0u32.to_le_bytes());
			let segment_start = self.buffer.len();

			self.buffer
				.extend_from_slice(&(self.runtime_resource_ids.len() as u32).to_le_bytes());

			for offset in self.runtime_resource_ids {
				self.buffer.extend_from_slice(&(offset - 0x10).to_le_bytes());
			}

			let segment_len = self.buffer.len() - segment_start;
			self.buffer[segment_start - 4..segment_start].copy_from_slice(&(segment_len as u32).to_le_bytes());
		}

		// ResourcePtrs segment
		if self.resource_ptrs_segment && !self.resource_ptrs.is_empty() {
			self.buffer[6] += 1;
			self.buffer.extend_from_slice(&0x578FBCEEu32.to_le_bytes());
			self.buffer.extend_from_slice(&0u32.to_le_bytes());
			let segment_start = self.buffer.len();

			self.buffer
				.extend_from_slice(&(self.resource_ptrs.len() as u32).to_le_bytes());

			for offset in self.resource_ptrs {
				self.buffer.extend_from_slice(&(offset - 0x10).to_le_bytes());
			}

			let segment_len = self.buffer.len() - segment_start;
			self.buffer[segment_start - 4..segment_start].copy_from_slice(&(segment_len as u32).to_le_bytes());
		}

		self.buffer
	}

	pub fn serialize(mut self, value: &impl Bin1Serialize) -> Result<Vec<u8>, SerializeError> {
		self.buffer[5] = value.alignment().min(8) as u8;
		value.write(&mut self)?;
		value.resolve(&mut self)?;
		self.finalise()
	}
}
