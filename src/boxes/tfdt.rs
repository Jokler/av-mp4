use byteorder::{BigEndian, ByteOrder};

use crate::*;

use crate::Mp4BoxError;

use std::mem::size_of;

pub struct TrackFragmentBaseMediaDecodeTimeBox {
    full_box: FullBox,
    pub base_media_decode_time: u64,
}

impl TrackFragmentBaseMediaDecodeTimeBox {
    const SIZE: u64 = size_of::<u64>() as u64; // base_media_decode_time

    pub fn new(base_media_decode_time: u64) -> Self {
        Self {
            full_box: FullBox::new(*b"tfdt", 1, 0),
            base_media_decode_time,
        }
    }

    pub fn write(self, writer: &mut dyn Write) -> Result<(), Mp4BoxError> {
        self.full_box.write(writer, self.total_size())?;

        let mut contents = [0u8; Self::SIZE as usize];
        BigEndian::write_u64(&mut contents, self.base_media_decode_time);

        writer.write_all(&contents)?;

        Ok(())
    }

    pub fn total_size(&self) -> u64 {
        self.full_box.size(Self::SIZE)
    }
}
