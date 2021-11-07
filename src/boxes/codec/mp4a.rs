use crate::*;

use super::esds::EsdBox;

use std::io::Write;

#[derive(Debug)]
pub struct Mpeg4AudioSampleEntryBox {
    pub audio_sample_entry: AudioSampleEntry,
    pub esds: EsdBox,
}

impl Mpeg4AudioSampleEntryBox {
    pub fn new(channel_count: u16, sample_size: u16, sample_rate: u32, esds: EsdBox) -> Self {
        Self {
            audio_sample_entry: AudioSampleEntry::new(
                *b"mp4a",
                1,
                channel_count,
                sample_size,
                sample_rate,
            ),
            esds,
        }
    }

    pub fn write(self, writer: &mut dyn Write) -> Result<(), Mp4BoxError> {
        self.audio_sample_entry.write(writer, self.total_size())?;

        self.esds.write(writer)?;

        Ok(())
    }

    pub fn total_size(&self) -> u64 {
        self.audio_sample_entry.size(self.size())
    }

    fn size(&self) -> u64 {
        self.esds.total_size()
    }

    pub fn read(buf: &mut dyn Buffered) -> Result<Self, Mp4BoxError> {
        let audio_sample_entry = AudioSampleEntry::read(buf)?;
        let esds = EsdBox::read(buf)?;

        Ok(Self {
            audio_sample_entry,
            esds,
        })
    }
}
