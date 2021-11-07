use byteorder::{BigEndian, ByteOrder};

use crate::*;

use crate::Mp4BoxError;

use std::mem::size_of;

pub struct MovieFragmentHeaderBox {
    full_box: FullBox,
    pub sequence_number: u32,
}

impl MovieFragmentHeaderBox {
    const SIZE: u64 = size_of::<u32>() as u64; // sequence_number

    pub fn new(sequence_number: u32) -> Self {
        Self {
            full_box: FullBox::new(*b"mfhd", 0, 0),
            sequence_number,
        }
    }

    pub fn write(self, writer: &mut dyn Write) -> Result<(), Mp4BoxError> {
        self.full_box.write(writer, self.total_size())?;

        let mut contents = [0u8; Self::SIZE as usize];
        BigEndian::write_u32(&mut contents, self.sequence_number);

        writer.write_all(&contents)?;

        Ok(())
    }

    pub fn total_size(&self) -> u64 {
        self.full_box.size(Self::SIZE)
    }
}
