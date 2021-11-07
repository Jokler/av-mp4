use crate::*;

use super::{
    tfdt::TrackFragmentBaseMediaDecodeTimeBox, tfhd::TrackFragmentHeaderBox,
    trun::TrackFragmentRunBox,
};

pub struct TrackFragmentBox {
    boks: Boks,
    pub tfhd: TrackFragmentHeaderBox,
    pub track_runs: Vec<TrackFragmentRunBox>,
    pub base_media_decode_time: Option<TrackFragmentBaseMediaDecodeTimeBox>,
}

impl TrackFragmentBox {
    pub fn new(
        tfhd: TrackFragmentHeaderBox,
        track_runs: Vec<TrackFragmentRunBox>,
        base_media_decode_time: Option<TrackFragmentBaseMediaDecodeTimeBox>,
    ) -> Self {
        Self {
            boks: Boks::new(*b"traf"),
            tfhd,
            track_runs,
            base_media_decode_time,
        }
    }

    pub fn write(self, writer: &mut dyn Write) -> Result<(), Mp4BoxError> {
        self.boks.write(writer, self.total_size())?;

        self.tfhd.write(writer)?;

        if let Some(base_media_decode_time) = self.base_media_decode_time {
            base_media_decode_time.write(writer)?;
        }

        for run in self.track_runs {
            run.write(writer)?;
        }

        Ok(())
    }

    pub fn total_size(&self) -> u64 {
        self.boks.size(self.size())
    }

    fn size(&self) -> u64 {
        let mut size = self.tfhd.total_size();

        for trun in &self.track_runs {
            size += trun.total_size();
        }

        if let Some(base_media_decode_time) = &self.base_media_decode_time {
            size += base_media_decode_time.total_size();
        }

        size
    }
}
