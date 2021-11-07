use crate::*;

use super::{mehd::MovieExtendsHeaderBox, trex::TrackExtendsBox};

use std::io::Write;

pub struct MovieExtendsBox {
    boks: Boks,
    mehd: MovieExtendsHeaderBox,
    trexs: Vec<TrackExtendsBox>,
}

impl MovieExtendsBox {
    pub fn new(mehd: MovieExtendsHeaderBox, trexs: Vec<TrackExtendsBox>) -> Self {
        MovieExtendsBox {
            boks: Boks::new(*b"mvex"),
            mehd,
            trexs,
        }
    }

    pub fn read(reader: &mut dyn Buffered) -> Result<Self, Mp4BoxError> {
        let _boks = Boks::read_named(reader, *b"moov")?;

        todo!()
        /*Ok(MovieExtendsBox {
            boks,
        })*/
    }

    pub fn write(self, writer: &mut dyn Write) -> Result<(), Mp4BoxError> {
        self.boks.write(writer, self.total_size())?;

        self.mehd.write(writer)?;
        for trex in self.trexs {
            trex.write(writer)?;
        }

        Ok(())
    }

    pub fn total_size(&self) -> u64 {
        self.boks.size(self.size())
    }

    fn size(&self) -> u64 {
        self.mehd.total_size() + self.trexs.iter().map(|t| t.total_size()).sum::<u64>()
    }
}
