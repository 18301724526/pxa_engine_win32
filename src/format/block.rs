use std::io::{Read, Write};
use crate::format::error::Result;
use crate::format::header::BlockHeader;
use crate::format::stream::{PxadWriter, PxadReader};

pub fn write_block<W: Write>(writer: &mut PxadWriter<W>, block_type: [u8; 4], payload: &[u8]) -> Result<()> {
    let header = BlockHeader::new(block_type, payload.len() as u64);
    
    header.write_to(writer)?;
    
    writer.write_all(payload)?;
    
    writer.write_padding()?;
    
    Ok(())
}

pub fn read_block<R: Read>(reader: &mut PxadReader<R>) -> Result<([u8; 4], Vec<u8>)> {
    let header = BlockHeader::read_from(reader)?;
    if header.payload_length > 256 * 1024 * 1024 {
        return Err(crate::format::error::FormatError::InvalidData(rust_i18n::t!("error.block_too_large").to_string()));
    }
    
    let mut payload = vec![0u8; header.payload_length as usize];
    reader.read_exact(&mut payload)?;
    
    reader.skip_padding()?;
    
    Ok((header.block_type, payload))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use crate::format::stream::{PxadWriter, PxadReader};

    #[test]
    fn test_block_read_write() {
        let mut buffer = Vec::new();
        
        {
            let mut writer = PxadWriter::new(&mut buffer);
            write_block(&mut writer, *b"CANV", b"12345").unwrap();
            write_block(&mut writer, *b"LAYR", b"Hello PXAD!").unwrap();
            writer.finish().unwrap();
        }

        let mut cursor = Cursor::new(buffer);
        let mut reader = PxadReader::new(&mut cursor);

        let (b_type1, payload1) = read_block(&mut reader).unwrap();
        assert_eq!(&b_type1, b"CANV");
        assert_eq!(payload1, b"12345");

        let (b_type2, payload2) = read_block(&mut reader).unwrap();
        assert_eq!(&b_type2, b"LAYR");
        assert_eq!(payload2, b"Hello PXAD!");

        assert!(reader.verify_footer().is_ok(), "文件尾部 CRC 校验必须通过");
    }
}