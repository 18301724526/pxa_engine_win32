use std::io::{Read, Write, Result as IoResult};
use crate::format::error::{FormatError, Result};
use crc32fast::Hasher;

pub struct PxadWriter<W: Write> {
    inner: W,
    hasher: Hasher,
    bytes_written: u64,
}

impl<W: Write> PxadWriter<W> {
    pub fn new(inner: W) -> Self {
        Self {
            inner,
            hasher: Hasher::new(),
            bytes_written: 0,
        }
    }

    pub fn bytes_written(&self) -> u64 {
        self.bytes_written
    }

    pub fn write_padding(&mut self) -> Result<u64> {
        let rem = self.bytes_written % 8;
        if rem != 0 {
            let pad_len = 8 - rem;
            let padding = vec![0u8; pad_len as usize];
            self.write_all(&padding)?;
            Ok(pad_len)
        } else {
            Ok(0)
        }
    }

    pub fn finish(mut self) -> Result<()> {
        let crc = self.hasher.finalize();
        self.inner.write_all(&crc.to_le_bytes())?;
        self.inner.write_all(b"PXA_EOF_2026")?;
        self.inner.flush()?;
        Ok(())
    }
}

impl<W: Write> Write for PxadWriter<W> {
    fn write(&mut self, buf: &[u8]) -> IoResult<usize> {
        let n = self.inner.write(buf)?;
        self.hasher.update(&buf[..n]);
        self.bytes_written += n as u64;
        Ok(n)
    }

    fn flush(&mut self) -> IoResult<()> {
        self.inner.flush()
    }
}


pub struct PxadReader<R: Read> {
    inner: R,
    hasher: Hasher,
    bytes_read: u64,
}

impl<R: Read> PxadReader<R> {
    pub fn new(inner: R) -> Self {
        Self {
            inner,
            hasher: Hasher::new(),
            bytes_read: 0,
        }
    }

    pub fn bytes_read(&self) -> u64 {
        self.bytes_read
    }

    pub fn skip_padding(&mut self) -> Result<u64> {
        let rem = self.bytes_read % 8;
        if rem != 0 {
            let pad_len = 8 - rem;
            let mut dump = vec![0u8; pad_len as usize];
            self.read_exact(&mut dump)?;
            if dump.iter().any(|&b| b != 0) {
                return Err(FormatError::InvalidData("Invalid padding: non-zero bytes found".to_string()));
            }
            Ok(pad_len)
        } else {
            Ok(0)
        }
    }

    pub fn verify_footer(mut self) -> Result<()> {
        let computed_crc = self.hasher.finalize();
        
        let mut crc_buf = [0u8; 4];
        self.inner.read_exact(&mut crc_buf)?;
        let file_crc = u32::from_le_bytes(crc_buf);

        if computed_crc != file_crc {
            return Err(FormatError::InvalidData(rust_i18n::t!("error.crc_mismatch").to_string()));
        }

        let mut eof_buf = [0u8; 12];
        self.inner.read_exact(&mut eof_buf)?;
        if &eof_buf != b"PXA_EOF_2026" {
            return Err(FormatError::InvalidData(rust_i18n::t!("error.invalid_eof").to_string()));
        }

        Ok(())
    }
}

impl<R: Read> Read for PxadReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
        let n = self.inner.read(buf)?;
        self.hasher.update(&buf[..n]);
        self.bytes_read += n as u64;
        Ok(n)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_writer_padding_and_crc() {
        let mut buffer = Vec::new();
        {
            let mut writer = PxadWriter::new(&mut buffer);
            writer.write_all(b"Hello").unwrap();
            assert_eq!(writer.bytes_written(), 5);
            
            let pad_len = writer.write_padding().unwrap();
            assert_eq!(pad_len, 3);
            assert_eq!(writer.bytes_written(), 8);

            writer.finish().unwrap();
        }

        assert_eq!(buffer.len(), 24);
        
        let mut cursor = Cursor::new(buffer);
        let mut reader = PxadReader::new(&mut cursor);
        
        let mut read_buf = [0u8; 5];
        reader.read_exact(&mut read_buf).unwrap();
        assert_eq!(&read_buf, b"Hello");
        assert_eq!(reader.bytes_read(), 5);

        let skipped = reader.skip_padding().unwrap();
        assert_eq!(skipped, 3);
        assert_eq!(reader.bytes_read(), 8);

        assert!(reader.verify_footer().is_ok());
    }

    #[test]
    fn test_reader_corrupted_data() {
        let mut buffer = Vec::new();
        {
            let mut writer = PxadWriter::new(&mut buffer);
            writer.write_all(b"Important Data").unwrap();
            writer.write_padding().unwrap();
            writer.finish().unwrap();
        }

        buffer[0] = b'X';

        let mut cursor = Cursor::new(buffer);
        let mut reader = PxadReader::new(&mut cursor);
        
        let mut read_buf = [0u8; 14];
        reader.read_exact(&mut read_buf).unwrap();
        reader.skip_padding().unwrap();
        
        let result = reader.verify_footer();
        assert!(result.is_err());
        match result.unwrap_err() {
            FormatError::InvalidData(_) => {}
            _ => panic!("Expected InvalidData error"),
        }
    }
}