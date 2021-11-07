use byteorder::{BigEndian, ReadBytesExt};

use crate::*;

use std::io::Write;
use std::mem::size_of;

const ES_DESCR_TAG: u8 = 0x3;
const DECODER_CONFIG_DESCR_TAG: u8 = 0x4;
const DECODER_SPECIFIC_DESCR_TAG: u8 = 0x5;
const SL_CONFIG_DESCR_TAG: u8 = 0x6;

#[derive(Debug)]
pub struct EsdBox {
    full_box: FullBox,
    pub descriptor: EsDescriptor,
}

impl EsdBox {
    pub fn new(descriptor: EsDescriptor) -> Self {
        EsdBox {
            full_box: FullBox::new(*b"esds", 0, 0),
            descriptor,
        }
    }

    pub fn read(buf: &mut dyn Buffered) -> Result<Self, Mp4BoxError> {
        let start = pos(buf)?;
        let full_box = FullBox::read_named(buf, *b"esds")?;

        let descriptor = EsDescriptor::read(buf)?;

        goto(buf, start + full_box.boks.size)?;

        Ok(EsdBox {
            full_box,
            descriptor,
        })
    }

    pub fn write(&self, writer: &mut dyn Write) -> Result<(), Mp4BoxError> {
        self.full_box.write(writer, self.total_size())?;

        self.descriptor.write(writer)?;

        Ok(())
    }

    pub fn total_size(&self) -> u64 {
        self.full_box.size(self.size())
    }

    fn size(&self) -> u64 {
        self.descriptor.total_size()
    }
}

#[derive(Debug)]
pub struct EsDescriptor {
    pub descriptor: BaseDescriptor,
    pub es_id: u16,
    pub decoder_descriptor: DecoderConfigDescriptor,
    pub sl_descriptor: SlConfigDescriptor,
}

impl EsDescriptor {
    pub fn new(es_id: u16, decoder_descriptor: DecoderConfigDescriptor) -> Self {
        Self {
            descriptor: BaseDescriptor::new(ES_DESCR_TAG),
            es_id,
            decoder_descriptor,
            sl_descriptor: SlConfigDescriptor::new(),
        }
    }

    pub fn write(&self, writer: &mut dyn Write) -> Result<(), Mp4BoxError> {
        self.descriptor.write(writer, self.total_size())?;

        let mut bytes = [0u8; 3];
        BigEndian::write_u16(&mut bytes[..], self.es_id);
        bytes[2] = 0; // flags and stream priority

        writer.write_all(&bytes)?;

        self.decoder_descriptor.write(writer)?;
        self.sl_descriptor.write(writer)?;

        Ok(())
    }

    pub fn read(buf: &mut dyn Buffered) -> Result<Self, Mp4BoxError> {
        let descriptor = BaseDescriptor::read(buf, ES_DESCR_TAG)?;
        let es_id = buf.read_u16::<BigEndian>()?;
        let flags = buf.read_u8()?;

        if (flags & 0b0000_0001) != 0 {
            skip(buf, 2)?;
        }

        if (flags & 0b0000_0010) != 0 {
            let len = buf.read_u8()?;
            skip(buf, len as _)?;
        }

        if (flags & 0b0000_0100) != 0 {
            skip(buf, 2)?;
        }

        let decoder_descriptor = DecoderConfigDescriptor::read(buf)?;

        Ok(EsDescriptor {
            descriptor,
            es_id,
            decoder_descriptor,
            sl_descriptor: todo!(),
        })
    }

    pub fn total_size(&self) -> u64 {
        self.descriptor.size(self.size())
    }

    fn size(&self) -> u64 {
        size_of::<u16>() as u64 // es_id
            + size_of::<u8>() as u64 // 0
            + self.decoder_descriptor.total_size()
            + self.sl_descriptor.total_size()
    }
}

#[derive(Debug)]
pub struct BaseDescriptor {
    pub tag: u8,
    pub size: u32,
    read_size: u8,
}

impl BaseDescriptor {
    pub fn new(tag: u8) -> Self {
        Self {
            tag,
            size: 0,
            read_size: 0,
        }
    }

    pub fn write(&self, writer: &mut dyn Write, size: u64) -> Result<(), Mp4BoxError> {
        let mut bytes = [0u8; 5];

        bytes[0] = self.tag;

        let size = size as u32 - 1 - size_of_length(size as u32);
        let count = size_of_length(size);

        for i in 1..=count {
            let offset = (count - i) * 7;
            let mut size = (size >> offset & 0b0111_1111) as u8;
            if i < count {
                size = size | 0b1000_0000;
            }

            bytes[i as usize] = size;
        }

        writer.write_all(&bytes[..(count as usize + 1)])?;

        Ok(())
    }

    pub fn peek(buf: &mut dyn Buffered) -> Result<Self, Mp4BoxError> {
        let bytes = peek(buf, 1)?;
        let tag = bytes[0];

        let mut size = 0u32;
        for i in 0..4 {
            let bytes = peek(buf, 1 + i)?;
            let b = bytes[1 + i];

            size = (size << 7) | (b & 0b0111_1111) as u32;

            if b & 0b1000_0000 == 0 {
                break;
            }
        }

        Ok(Self {
            tag,
            size,
            read_size: 0,
        })
    }

    pub fn read(buf: &mut dyn Buffered, expected: u8) -> Result<Self, Mp4BoxError> {
        let tag = buf.read_u8()?;

        if tag != expected {
            return Err(Mp4BoxError::UnexpectedTag(expected, tag));
        }

        let mut len = 1;
        let mut size = 0u32;
        for _ in 0..4 {
            let b = buf.read_u8()?;
            len += 1;

            size = (size << 7) | (b & 0b0111_1111) as u32;

            if b & 0b1000_0000 == 0 {
                break;
            }
        }

        Ok(Self {
            tag,
            size,
            read_size: len,
        })
    }

    pub fn remaining_size(&self) -> u64 {
        self.size as u64 - self.read_size as u64
    }

    pub fn size(&self, size: u64) -> u64 {
        size_of::<u8>() as u64 // tag
            + size_of_length(size as u32) as u64 // size of size
            + size
    }
}

#[derive(Debug)]
pub struct DecoderConfigDescriptor {
    pub descriptor: BaseDescriptor,
    pub object_type_indication: u8,

    pub buffer_size_db: u32,
    pub max_bitrate: u32,
    pub avg_bitrate: u32,

    pub decoder_specific: Option<DecoderSpecificInfo>,
}

impl DecoderConfigDescriptor {
    pub fn new(object_type_indication: u8, decoder_specific: Option<DecoderSpecificInfo>) -> Self {
        Self {
            descriptor: BaseDescriptor::new(DECODER_CONFIG_DESCR_TAG),
            object_type_indication,
            buffer_size_db: 0,
            max_bitrate: 0,
            avg_bitrate: 0,
            decoder_specific,
        }
    }

    pub fn read(buf: &mut dyn Buffered) -> Result<Self, Mp4BoxError> {
        let descriptor = BaseDescriptor::read(buf, DECODER_CONFIG_DESCR_TAG)?;

        let object_type_indication = buf.read_u8()?;
        let _ = buf.read_u8()?;
        let buffer_size_db = buf.read_u24::<BigEndian>()?;
        let max_bitrate = buf.read_u32::<BigEndian>()?;
        let avg_bitrate = buf.read_u32::<BigEndian>()?;

        let mut decoder_specific = Vec::new();

        // TODO: maybe parse to supported descriptor directly, instead of storing bytes
        if let Ok(desc) = BaseDescriptor::read(buf, DECODER_SPECIFIC_DESCR_TAG) {
            decoder_specific.resize(desc.remaining_size() as usize, 0);
            buf.read_exact(&mut decoder_specific[..])?;
        }

        Ok(Self {
            descriptor,
            object_type_indication,
            buffer_size_db,
            max_bitrate,
            avg_bitrate,
            decoder_specific: todo!(),
        })
    }

    fn write(&self, writer: &mut dyn Write) -> Result<(), Mp4BoxError> {
        self.descriptor.write(writer, self.total_size())?;

        let mut bytes = [0u8; 13];
        bytes[0] = self.object_type_indication;
        bytes[1] = (0x05 << 2) | 1; // streamtype + upstream + reserved
        BigEndian::write_u24(&mut bytes[2..], self.buffer_size_db);
        BigEndian::write_u32(&mut bytes[5..], self.max_bitrate);
        BigEndian::write_u32(&mut bytes[9..], self.avg_bitrate);

        writer.write_all(&bytes)?;

        if let Some(decoder_specific) = &self.decoder_specific {
            decoder_specific.write(writer)?;
        }

        Ok(())
    }

    pub fn total_size(&self) -> u64 {
        self.descriptor.size(self.size())
    }

    fn size(&self) -> u64 {
        size_of::<u8>() as u64 // object_indication
            + size_of::<u8>() as u64 // streamtype + upstream + reserved
            + size_of::<u8>() as u64 * 3 // buffer_size_db
            + size_of::<u32>() as u64 // max_bitrate
            + size_of::<u32>() as u64 // avg_bitrate
            + self.decoder_specific.as_ref().map(|d| d.total_size()).unwrap_or(0)
    }
}

fn size_of_length(size: u32) -> u32 {
    match size {
        0x0..=0x7F => 1,
        0x80..=0x3FFF => 2,
        0x4000..=0x1FFFFF => 3,
        _ => 4,
    }
}

#[derive(Debug)]
pub struct SlConfigDescriptor {
    pub descriptor: BaseDescriptor,
}

impl SlConfigDescriptor {
    pub fn new() -> Self {
        Self {
            descriptor: BaseDescriptor::new(SL_CONFIG_DESCR_TAG),
        }
    }

    fn write(&self, writer: &mut dyn Write) -> Result<(), Mp4BoxError> {
        self.descriptor.write(writer, self.total_size())?;
        writer.write(&[2u8])?;

        Ok(())
    }

    pub fn total_size(&self) -> u64 {
        self.descriptor.size(self.size())
    }

    fn size(&self) -> u64 {
        size_of::<u8>() as u64 // predefined
    }
}

#[derive(Debug)]
pub struct DecoderSpecificInfo {
    pub descriptor: BaseDescriptor,
    pub decoder_specific: Vec<u8>,
}

impl DecoderSpecificInfo {
    pub fn new(decoder_specific: Vec<u8>) -> Self {
        assert!(!decoder_specific.is_empty());

        Self {
            descriptor: BaseDescriptor::new(DECODER_SPECIFIC_DESCR_TAG),
            decoder_specific,
        }
    }
    fn write(&self, writer: &mut dyn Write) -> Result<(), Mp4BoxError> {
        self.descriptor.write(writer, self.total_size())?;

        writer.write_all(&self.decoder_specific)?;

        Ok(())
    }

    pub fn total_size(&self) -> u64 {
        self.descriptor.size(self.size())
    }

    fn size(&self) -> u64 {
        self.decoder_specific.len() as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn desc_size() {
        let mut buf = Vec::new();
        for expected in 1..(2_u32.pow(7 * 4)) {
            buf.clear();
            let desc = BaseDescriptor::new(0);

            desc.write(&mut buf, expected as u64).unwrap();

            let mut reader = av_format::buffer::AccReader::with_capacity(
                buf.len(),
                std::io::Cursor::new(buf.as_slice()),
            );
            let desc = BaseDescriptor::read(&mut reader, 0).unwrap();

            assert_eq!(expected, desc.size);
        }
    }
}
