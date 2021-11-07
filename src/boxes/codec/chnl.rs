use crate::*;

use std::io::Write;
use std::mem::size_of;

#[derive(Debug)]
pub struct ChannelLayout {
    full_box: FullBox,
    pub stream_structure: u8,
}

impl ChannelLayout {
    pub fn new() -> Self {
        Self {
            full_box: FullBox::new(*b"chnl", 0, 0),
            stream_structure: 0,
        }
    }

    pub fn read(buf: &mut dyn Buffered) -> Result<Self, Mp4BoxError> {
        let start = pos(buf)?;
        let full_box = FullBox::read_named(buf, *b"chnl")?;

        goto(buf, start + full_box.boks.size)?;

        todo!()
    }

    pub fn write(&self, writer: &mut dyn Write) -> Result<(), Mp4BoxError> {
        self.full_box.write(writer, self.total_size())?;

        writer.write(&[self.stream_structure])?;

        Ok(())
    }

    pub fn total_size(&self) -> u64 {
        self.full_box.size(self.size())
    }

    fn size(&self) -> u64 {
        size_of::<u8>() as u64 // stream_structure
    }
}
