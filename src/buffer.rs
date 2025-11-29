use std::io;

pub struct BufferReader<'a> {
    buffer: &'a [u8],
    position: usize,
}

impl<'a> BufferReader<'a> {
    pub fn new(buffer: &'a [u8]) -> Self {
        Self {
            buffer,
            position: 0,
        }
    }

    pub fn read_i32(&mut self) -> crate::Result<i32> {
        let bytes = self.read_bytes(4)?;
        Ok(i32::from_le_bytes(bytes.try_into()?))
    }

    pub fn read_u8(&mut self) -> crate::Result<u8> {
        let byte = self.read_byte()?;
        Ok(byte)
    }

    pub fn read_u16(&mut self) -> crate::Result<u16> {
        let bytes = self.read_bytes(2)?;
        Ok(u16::from_le_bytes(bytes.try_into()?))
    }

    pub fn read_u32(&mut self) -> crate::Result<u32> {
        let bytes = self.read_bytes(4)?;
        Ok(u32::from_le_bytes(bytes.try_into()?))
    }

    pub fn read_u64(&mut self) -> crate::Result<u64> {
        let bytes = self.read_bytes(8)?;
        Ok(u64::from_le_bytes(bytes.try_into()?))
    }

    pub fn read_bool(&mut self) -> crate::Result<bool> {
        Ok(self.read_u8()? != 0)
    }

    pub fn read_string(&mut self) -> crate::Result<String> {
        let length = self.read_u32()? as usize;
        let bytes = self.read_bytes(length)?;
        Ok(String::from_utf8(bytes.to_vec())?)
    }

    pub fn read_byte(&mut self) -> crate::Result<u8> {
        let byte = self.read_bytes(1)?;
        Ok(byte[0])
    }

    pub fn read_bytes(&mut self, count: usize) -> crate::Result<&[u8]> {
        if self.position + count > self.buffer.len() {
            return Err(Box::new(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "Not enough bytes in buffer",
            )));
        }

        let slice = &self.buffer[self.position..self.position + count];
        self.position += count;
        Ok(slice)
    }

    pub fn is_empty(&self) -> bool {
        self.position == self.buffer.len()
    }
}

pub struct BufferWriter {
    buffer: Vec<u8>,
}

impl BufferWriter {
    pub fn new() -> Self {
        Self { buffer: Vec::new() }
    }

    pub fn write_u8(&mut self, value: u8) -> &mut Self {
        self.buffer.push(value);
        self
    }

    pub fn write_u32(&mut self, value: u32) -> &mut Self {
        self.buffer.extend_from_slice(&value.to_le_bytes());
        self
    }

    pub fn write_bool(&mut self, value: bool) -> &mut Self {
        self.buffer.push(value as u8);
        self
    }

    pub fn write_string(&mut self, value: &str) -> &mut Self {
        self.write_u32(value.len() as u32);
        self.buffer.extend_from_slice(value.as_bytes());
        self
    }

    pub fn write_byte_array(&mut self, bytes: &[u8]) -> &mut Self {
        self.write_u32(bytes.len() as u32);
        self.buffer.extend_from_slice(bytes);
        self
    }

    pub fn buffer(&self) -> &[u8] {
        &self.buffer
    }

    pub fn to_vec(&self) -> Vec<u8> {
        self.buffer.to_vec()
    }
}
