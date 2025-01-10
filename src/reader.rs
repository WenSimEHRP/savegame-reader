use std::fs::File;
use std::io::Read;
use std::io::Write;

trait Reader {
    fn load(&self, start: usize, end: usize) -> &[u8];
    fn read_byte(&mut self) -> u8;
    fn read(&mut self, len: usize) -> &[u8];
    fn read_leftover(&self) -> &[u8];
    fn read_all(&self) -> &[u8];
    fn read_u8(&mut self) -> u8;
    fn read_u16(&mut self) -> u16;
    fn read_u32(&mut self) -> u32;
    fn read_u64(&mut self) -> u64;
    fn read_i8(&mut self) -> i8;
    fn read_i16(&mut self) -> i16;
    fn read_i32(&mut self) -> i32;
    fn read_i64(&mut self) -> i64;
    fn read_gamma(&mut self) -> u32;
    fn read_string(&mut self, len: u32) -> String;
}

struct FileReader {
    path: String,
    data: Vec<u8>,
    position: usize,
}

impl FileReader {
    fn new(path: String) -> Self {
        let mut file = File::open(&path).unwrap();
        let mut data = Vec::new();
        file.read_to_end(&mut data).unwrap();
        FileReader {
            path: path,
            data: data,
            position: 0,
        }
    }
}

impl Reader for FileReader {
    fn load(&self, start: usize, end: usize) -> &[u8] {
        &self.data[start..end]
    }

    fn read_byte(&mut self) -> u8 {
        let byte = self.data[self.position];
        self.position += 1;
        byte
    }

    fn read(&mut self, len: usize) -> &[u8] {
        let start = self.position;
        self.position += len;
        &self.data[start..self.position]
    }

    fn read_leftover(&self) -> &[u8] {
        let start = self.position;
        &self.data[start..]
    }

    fn read_all(&self) -> &[u8] {
        &self.data
    }

    fn read_u8(&mut self) -> u8 {
        self.read_byte()
    }
    fn read_u16(&mut self) -> u16 {
        u16::from_be_bytes(self.read(2).try_into().unwrap())
    }
    fn read_u32(&mut self) -> u32 {
        u32::from_be_bytes(self.read(4).try_into().unwrap())
    }
    fn read_u64(&mut self) -> u64 {
        u64::from_be_bytes(self.read(8).try_into().unwrap())
    }
    fn read_i8(&mut self) -> i8 {
        i8::from_be_bytes([self.read_byte()])
    }
    fn read_i16(&mut self) -> i16 {
        i16::from_be_bytes(self.read(2).try_into().unwrap())
    }
    fn read_i32(&mut self) -> i32 {
        i32::from_be_bytes(self.read(4).try_into().unwrap())
    }
    fn read_i64(&mut self) -> i64 {
        i64::from_be_bytes(self.read(8).try_into().unwrap())
    }
    fn read_gamma(&mut self) -> u32 {
        let byte = self.read_byte();
        if byte & 0b10000000 == 0 {
            byte as u32
        } else if byte & 0b01000000 == 0 {
            (((byte & 0b00111111) as u32) << 8) | self.read_u8() as u32
        } else if byte & 0b00100000 == 0 {
            (((byte & 0b00011111) as u32) << 16) | self.read_u16() as u32
        } else if byte & 0b00010000 == 0 {
            (((byte & 0b00001111) as u32) << 24)
                | (self.read_u16() as u32) << 8
                | self.read_u8() as u32
        } else if byte & 0b00001000 == 0 {
            self.read_u32()
        } else {
            panic!("Error when decoding gamma: {}", self.position);
        }
    }

    fn read_string(&mut self, len: u32) -> String {
        String::from_utf8(self.read(len as usize).to_vec()).unwrap()
    }
}

struct DataReader {
    data: Vec<u8>,
    position: usize,
}

impl DataReader {
    fn new(data: Vec<u8>) -> Self {
        DataReader {
            data: data,
            position: 0,
        }
    }
}

impl Reader for DataReader {
    fn load(&self, start: usize, end: usize) -> &[u8] {
        &self.data[start..end]
    }

    fn read_byte(&mut self) -> u8 {
        let byte = self.data[self.position];
        self.position += 1;
        byte
    }

    fn read(&mut self, len: usize) -> &[u8] {
        let start = self.position;
        self.position += len;
        &self.data[start..self.position]
    }

    fn read_leftover(&self) -> &[u8] {
        let start = self.position;
        &self.data[start..]
    }

    fn read_all(&self) -> &[u8] {
        &self.data
    }

    fn read_u8(&mut self) -> u8 {
        self.read_byte()
    }
    fn read_u16(&mut self) -> u16 {
        u16::from_be_bytes(self.read(2).try_into().unwrap())
    }
    fn read_u32(&mut self) -> u32 {
        u32::from_be_bytes(self.read(4).try_into().unwrap())
    }
    fn read_u64(&mut self) -> u64 {
        u64::from_be_bytes(self.read(8).try_into().unwrap())
    }
    fn read_i8(&mut self) -> i8 {
        i8::from_be_bytes([self.read_byte()])
    }
    fn read_i16(&mut self) -> i16 {
        i16::from_be_bytes(self.read(2).try_into().unwrap())
    }
    fn read_i32(&mut self) -> i32 {
        i32::from_be_bytes(self.read(4).try_into().unwrap())
    }
    fn read_i64(&mut self) -> i64 {
        i64::from_be_bytes(self.read(8).try_into().unwrap())
    }
    fn read_gamma(&mut self) -> u32 {
        let byte = self.read_byte();
        if byte & 0b10000000 == 0 {
            byte as u32
        } else if byte & 0b01000000 == 0 {
            (((byte & 0b00111111) as u32) << 8) | self.read_u8() as u32
        } else if byte & 0b00100000 == 0 {
            (((byte & 0b00011111) as u32) << 16) | self.read_u16() as u32
        } else if byte & 0b00010000 == 0 {
            (((byte & 0b00001111) as u32) << 24)
                | (self.read_u16() as u32) << 8
                | self.read_u8() as u32
        } else if byte & 0b00001000 == 0 {
            self.read_u32()
        } else {
            panic!("Error when decoding gamma: {}", self.position);
        }
    }

    fn read_string(&mut self, len: u32) -> String {
        String::from_utf8(self.read(len as usize).to_vec()).unwrap()
    }
}

#[derive(Debug)]
pub enum CompressionType {
    None,
    Zlib,
    Lzma,
}

/// case OTTN: no decompression, return the data as is
fn decompress_none(data: &[u8]) -> Vec<u8> {
    data.to_vec()
}

/// case OTTZ: zlib decompression, return the decompressed data
fn decompress_zlib(data: &[u8]) -> Vec<u8> {
    use flate2::read::ZlibDecoder;

    let mut decoder = ZlibDecoder::new(data);
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed).unwrap();
    decompressed
}

/// case OTTX: lzma decompression, return the decompressed data
fn decompress_lzma(data: &[u8]) -> Vec<u8> {
    use xz2::read::XzDecoder;

    let mut decoder = XzDecoder::new(data);
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed).unwrap();
    decompressed
}

#[derive(Debug)]
pub struct Savegame {
    pub path: String,
    pub data: Vec<u8>,
    pub version: u16,
    pub compression: CompressionType,
}

impl Savegame {

    pub fn new(path: String) -> Self {
        let mut reader = FileReader::new(path.clone());
        let compression = match reader.read(4) {
            b"OTTN" => CompressionType::None,
            b"OTTZ" => CompressionType::Zlib,
            b"OTTX" => CompressionType::Lzma,
            b"OTTD" => panic!("LZO compression is unsupported"),
            _ => panic!("Unknown compression type"),
        };
        let version = reader.read_u16();
        reader.read(2); // skip 2 bytes
        let data = reader.read_leftover();
        let data = match compression {
            CompressionType::None => decompress_none(data),
            CompressionType::Zlib => decompress_zlib(data),
            CompressionType::Lzma => decompress_lzma(data),
        };
        Savegame {
            path: path,
            compression: compression,
            version: version,
            data: data,
        }
    }

    pub fn save(&self, path: String) {
        let mut file = File::create(path).unwrap();
        file.write_all(&self.data).unwrap();
    }
}
