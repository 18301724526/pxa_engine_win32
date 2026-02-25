use std::io::{Read, Write};
use crate::format::error::{FormatError, Result};

pub const PXAD_MAGIC: [u8; 4] = *b"PXAD";
pub const CURRENT_MAJOR_VERSION: u16 = 1;
pub const CURRENT_MINOR_VERSION: u16 = 2;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct PxadHeader {
    pub magic: [u8; 4],
    pub major_version: u16,
    pub minor_version: u16,
    pub file_size: u64,
    pub block_count: u64,
    pub reserved: [u8; 8],
}

impl PxadHeader {
    pub fn new() -> Self {
        Self {
            magic: PXAD_MAGIC,
            major_version: CURRENT_MAJOR_VERSION,
            minor_version: CURRENT_MINOR_VERSION,
            file_size: 0, 
            block_count: 0, 
            reserved: [0; 8],
        }
    }

    pub fn write_to<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_all(&self.magic)?;
        writer.write_all(&self.major_version.to_le_bytes())?;
        writer.write_all(&self.minor_version.to_le_bytes())?;
        writer.write_all(&self.file_size.to_le_bytes())?;
        writer.write_all(&self.block_count.to_le_bytes())?;
        writer.write_all(&self.reserved)?;
        Ok(())
    }

    pub fn read_from<R: Read>(reader: &mut R) -> Result<Self> {
        let mut magic = [0u8; 4];
        reader.read_exact(&mut magic)?;
        
        if magic != PXAD_MAGIC {
            return Err(FormatError::InvalidData(rust_i18n::t!("error.invalid_magic").to_string()));
        }

        let mut buf_u16 = [0u8; 2];
        let mut buf_u64 = [0u8; 8];

        reader.read_exact(&mut buf_u16)?;
        let major_version = u16::from_le_bytes(buf_u16);

        reader.read_exact(&mut buf_u16)?;
        let minor_version = u16::from_le_bytes(buf_u16);

        reader.read_exact(&mut buf_u64)?;
        let file_size = u64::from_le_bytes(buf_u64);

        reader.read_exact(&mut buf_u64)?;
        let block_count = u64::from_le_bytes(buf_u64);

        let mut reserved = [0u8; 8];
        reader.read_exact(&mut reserved)?;

        Ok(Self { magic, major_version, minor_version, file_size, block_count, reserved })
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct BlockHeader {
    pub block_type: [u8; 4],
    pub payload_length: u64,
}

impl BlockHeader {
    pub fn new(block_type: [u8; 4], payload_length: u64) -> Self {
        Self { block_type, payload_length }
    }

    pub fn write_to<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_all(&self.block_type)?;
        writer.write_all(&self.payload_length.to_le_bytes())?;
        Ok(())
    }

    pub fn read_from<R: Read>(reader: &mut R) -> Result<Self> {
        let mut block_type = [0u8; 4];
        reader.read_exact(&mut block_type)?;

        let mut buf_u64 = [0u8; 8];
        reader.read_exact(&mut buf_u64)?;
        let payload_length = u64::from_le_bytes(buf_u64);

        Ok(Self { block_type, payload_length })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_pxad_header_io() {
        let mut header = PxadHeader::new();
        header.file_size = 1024;
        header.block_count = 5;

        let mut buffer = Vec::new();
        header.write_to(&mut buffer).unwrap();
        
        assert_eq!(buffer.len(), 32, "Header 必须严格为 32 字节");

        let mut cursor = Cursor::new(buffer);
        let decoded = PxadHeader::read_from(&mut cursor).unwrap();

        assert_eq!(header, decoded);
    }

    #[test]
    fn test_invalid_magic() {
        let bad_data = b"BADD\x01\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00";
        let mut cursor = Cursor::new(bad_data);
        let result = PxadHeader::read_from(&mut cursor);
        assert!(result.is_err());
    }

    #[test]
    fn test_block_header_io() {
        let block = BlockHeader::new(*b"LAYR", 16384);
        let mut buffer = Vec::new();
        block.write_to(&mut buffer).unwrap();

        assert_eq!(buffer.len(), 12, "Block Header 必须严格为 12 字节");

        let mut cursor = Cursor::new(buffer);
        let decoded = BlockHeader::read_from(&mut cursor).unwrap();

        assert_eq!(block, decoded);
    }
}