use crate::*;

use super::{mfhd::MovieFragmentHeaderBox, traf::TrackFragmentBox};

pub struct MovieFragmentBox {
    boks: Boks,
    pub mfhd: MovieFragmentHeaderBox,
    pub traf: TrackFragmentBox,
}

impl MovieFragmentBox {
    pub fn new(mfhd: MovieFragmentHeaderBox, traf: TrackFragmentBox) -> Self {
        Self {
            boks: Boks::new(*b"moof"),
            mfhd,
            traf,
        }
    }

    pub fn write(self, writer: &mut dyn Write) -> Result<(), Mp4BoxError> {
        self.boks.write(writer, self.total_size())?;
        self.mfhd.write(writer)?;
        self.traf.write(writer)?;

        Ok(())
    }

    pub fn total_size(&self) -> u64 {
        self.boks.size(self.size())
    }

    fn size(&self) -> u64 {
        let mut size = self.mfhd.total_size();
        size += self.traf.total_size();

        size
    }
}
