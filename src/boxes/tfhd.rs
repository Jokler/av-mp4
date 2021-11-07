use byteorder::{BigEndian, WriteBytesExt};

use crate::*;

use crate::Mp4BoxError;

use std::mem::size_of;

bitflags::bitflags! {
    pub struct TrackFragmentHeaderFlags: u32 {
        const BASE_DATA_OFFSET_PRESENT = 0x000001;
        const SAMPLE_DESCRIPTION_INDEX_PRESENT = 0x000002;
        const DEFAULT_SAMPLE_DURATION_PRESENT = 0x000008;
        const DEFAULT_SAMPLE_SIZE_PRESENT = 0x000010;
        const DEFAULT_SAMPLE_FLAGS_PRESENT = 0x000020;
        const DURATION_IS_EMPTY = 0x010000;
        const DEFAULT_BASE_IS_MOOF = 0x020000;
    }
}

pub struct TrackFragmentHeaderBox {
    full_box: FullBox,
    pub track_id: u32,
    pub base_data_offset: Option<u64>,
    pub sample_description_index: Option<u32>,
    pub default_sample_duration: Option<u32>,
    pub default_sample_size: Option<u32>,
    pub default_sample_flags: Option<u32>,
    pub duration_is_empty: bool,
    pub default_base_is_moof: bool,
}

impl TrackFragmentHeaderBox {
    fn flags_from_fields(&self) -> TrackFragmentHeaderFlags {
        let mut flags = TrackFragmentHeaderFlags::empty();

        if self.base_data_offset.is_some() {
            flags.insert(TrackFragmentHeaderFlags::BASE_DATA_OFFSET_PRESENT);
        }

        if self.sample_description_index.is_some() {
            flags.insert(TrackFragmentHeaderFlags::SAMPLE_DESCRIPTION_INDEX_PRESENT);
        }

        if self.default_sample_duration.is_some() {
            flags.insert(TrackFragmentHeaderFlags::DEFAULT_SAMPLE_DURATION_PRESENT);
        }

        if self.default_sample_size.is_some() {
            flags.insert(TrackFragmentHeaderFlags::DEFAULT_SAMPLE_SIZE_PRESENT);
        }

        if self.default_sample_flags.is_some() {
            flags.insert(TrackFragmentHeaderFlags::DEFAULT_SAMPLE_FLAGS_PRESENT);
        }

        if self.duration_is_empty {
            flags.insert(TrackFragmentHeaderFlags::DURATION_IS_EMPTY);
        }

        if self.default_base_is_moof {
            flags.insert(TrackFragmentHeaderFlags::DEFAULT_BASE_IS_MOOF);
        }

        flags
    }

    pub fn new(
        track_id: u32,
        base_data_offset: Option<u64>,
        sample_description_index: Option<u32>,
        default_sample_duration: Option<u32>,
        default_sample_size: Option<u32>,
        default_sample_flags: Option<u32>,
        duration_is_empty: bool,
        default_base_is_moof: bool,
    ) -> Self {
        let mut tfhd = Self {
            full_box: FullBox::new(*b"tfhd", 0, 0),
            track_id,
            base_data_offset,
            sample_description_index,
            default_sample_duration,
            default_sample_size,
            default_sample_flags,
            duration_is_empty,
            default_base_is_moof,
        };

        tfhd.full_box = FullBox::new(*b"tfhd", 0, tfhd.flags_from_fields().bits());

        tfhd
    }

    pub fn write(self, writer: &mut dyn Write) -> Result<(), Mp4BoxError> {
        self.full_box.write(writer, self.total_size())?;
        let mut v = Vec::new();

        v.write_u32::<BigEndian>(self.track_id)?;

        if let Some(base_data_offset) = self.base_data_offset {
            v.write_u64::<BigEndian>(base_data_offset)?;
        }

        if let Some(sample_description_index) = self.sample_description_index {
            v.write_u32::<BigEndian>(sample_description_index)?;
        }

        if let Some(default_sample_duration) = self.default_sample_duration {
            v.write_u32::<BigEndian>(default_sample_duration)?;
        }

        if let Some(default_sample_size) = self.default_sample_size {
            v.write_u32::<BigEndian>(default_sample_size)?;
        }

        if let Some(default_sample_flags) = self.default_sample_flags {
            v.write_u32::<BigEndian>(default_sample_flags)?;
        }

        writer.write_all(&v)?;

        Ok(())
    }

    pub fn total_size(&self) -> u64 {
        self.full_box.size(self.size())
    }

    fn size(&self) -> u64 {
        let mut size = size_of::<u32>() as u64; // track_ID

        if self.base_data_offset.is_some() {
            size += size_of::<u64>() as u64;
        }

        if self.sample_description_index.is_some() {
            size += size_of::<u32>() as u64;
        }

        if self.default_sample_duration.is_some() {
            size += size_of::<u32>() as u64;
        }

        if self.default_sample_size.is_some() {
            size += size_of::<u32>() as u64;
        }

        if self.default_sample_flags.is_some() {
            size += size_of::<u32>() as u64;
        }

        size
    }
}
