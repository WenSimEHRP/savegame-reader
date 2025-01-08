use std::cmp::min;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::io::Write;
use std::vec;

/// case OTTN: no decompression, return the data as is
fn decompress_none(data: Vec<u8>) -> Vec<u8> {
    data
}

/// case OTTZ: zlib decompression, return the decompressed data
fn decompress_zlib(data: Vec<u8>) -> Vec<u8> {
    use flate2::read::ZlibDecoder;
    use std::io::prelude::*;

    let mut decoder = ZlibDecoder::new(data.as_slice());
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed).unwrap();
    decompressed
}

/// case OTTX: lzma decompression, return the decompressed data
fn decompress_lzma(data: Vec<u8>) -> Vec<u8> {
    use xz2::read::XzDecoder;
    let mut decoder = XzDecoder::new(data.as_slice());
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed).unwrap();
    decompressed
}

trait Reader {
    fn load(&mut self, start: usize, size: usize) -> Vec<u8>;
    fn read_byte(&mut self) -> u8;
    fn read_bytes(&mut self, size: usize) -> Vec<u8>;
    fn read_leftover_bytes(&mut self) -> Vec<u8>;
    fn read_all_bytes(&mut self) -> Vec<u8>;
}

struct FileReader {
    path: String,
    position: usize,
    size: usize,
}

impl FileReader {
    fn new(path: String) -> FileReader {
        let metadata = std::fs::metadata(&path).unwrap();
        let size = metadata.len() as usize;
        FileReader {
            path,
            position: 0,
            size,
        }
    }
}

impl Reader for FileReader {
    fn load(&mut self, start: usize, size: usize) -> Vec<u8> {
        if size == 0 {
            return Vec::new();
        }
        let size = min(size, self.size - start);
        let mut file = File::open(&self.path).unwrap();
        file.seek(SeekFrom::Start(start as u64)).unwrap();
        let mut data = vec![0; size];
        file.read_exact(&mut data).unwrap();
        data
    }

    fn read_byte(&mut self) -> u8 {
        let data = self.load(self.position, 1);
        self.position += 1;
        data[0]
    }

    fn read_bytes(&mut self, size: usize) -> Vec<u8> {
        let data = self.load(self.position, size);
        self.position += size;
        data
    }

    fn read_leftover_bytes(&mut self) -> Vec<u8> {
        let data = self.load(self.position, self.size - self.position);
        self.position = self.size;
        data
    }

    fn read_all_bytes(&mut self) -> Vec<u8> {
        self.load(0, self.size)
    }
}

struct DataReader<'a> {
    data: &'a Vec<u8>,
    position: usize,
    size: usize,
}

impl<'a> DataReader<'a> {
    fn new(data: &'a Vec<u8>) -> DataReader<'a> {
        let size = data.len();
        DataReader {
            data,
            position: 0,
            size,
        }
    }

    // Read gamma and auto adjust the position of the reader
    fn read_gamma(&mut self) -> (u32, u8) {
        let byte = self.read_byte();
        if byte & 0b10000000 == 0 {
            (byte as u32, 1)
        } else if byte & 0b01000000 == 0 {
            let byte2 = self.read_byte();
            ((((byte & 0b00111111) as u32) << 8) | byte2 as u32, 2)
        } else if byte & 0b00100000 == 0 {
            let byte2 = self.read_byte();
            let byte3 = self.read_byte();
            (
                (((byte & 0b00011111) as u32) << 16)
                    | ((byte2 & 0b00111111) as u32) << 8
                    | byte3 as u32,
                3,
            )
        } else if byte & 0b00010000 == 0 {
            let byte2 = self.read_byte();
            let byte3 = self.read_byte();
            let byte4 = self.read_byte();
            (
                (((byte & 0b00001111) as u32) << 24)
                    | ((byte2 & 0b00111111) as u32) << 16
                    | ((byte3 & 0b00111111) as u32) << 8
                    | byte4 as u32,
                4,
            )
        } else if byte & 0b00001000 == 0 {
            let byte2 = self.read_byte();
            let byte3 = self.read_byte();
            let byte4 = self.read_byte();
            let byte5 = self.read_byte();
            (
                (((byte & 0b00000111) as u32) << 28)
                    | ((byte2 & 0b00111111) as u32) << 22
                    | ((byte3 & 0b00111111) as u32) << 16
                    | ((byte4 & 0b00111111) as u32) << 8
                    | byte5 as u32,
                5,
            )
        } else {
            panic!("Error when decoding gamma: {}", self.position);
        }
    }
}

impl<'a> Reader for DataReader<'a> {
    fn load(&mut self, start: usize, size: usize) -> Vec<u8> {
        if size == 0 {
            return Vec::new();
        }
        let size = min(size, self.size - start);
        self.position = start;
        self.data[start..start + size].to_vec()
    }

    fn read_byte(&mut self) -> u8 {
        let data = self.load(self.position, 1);
        self.position += 1;
        data[0]
    }

    fn read_bytes(&mut self, size: usize) -> Vec<u8> {
        let data = self.load(self.position, size);
        self.position += size;
        data
    }

    fn read_leftover_bytes(&mut self) -> Vec<u8> {
        let data = self.load(self.position, self.size - self.position);
        self.position = self.size;
        data
    }

    fn read_all_bytes(&mut self) -> Vec<u8> {
        self.data.clone()
    }
}

struct SavegameDataType {
    data: HashMap<String, (u8, u16, u32, i8, i16, i32, String, SavegameDataType)>,
}

struct SavegameChunk {
    label: String,
    chunk_type: DataHandlerChunks,
    // data: SavegameDataType,
    data: Vec<u8>, //TODO handle the data
}

enum DataHandlerChunks {
    Riff = 0,
    Array = 1,
    SparseArray = 2,
    Table = 3,
    SparseTable = 4,
}

#[derive(Debug, PartialEq)]
enum TableDataType {
    None = 0,
    Int8 = 1,
    Uint8 = 2,
    Int16 = 3,
    Uint16 = 4,
    Int32 = 5,
    Uint32 = 6,
    Int64 = 7,
    Uint64 = 8,
    StringID = 9,
    Str = 10,
    Struct = 11,
}

struct DataHandler<'a> {
    data: &'a Vec<u8>,
    reader: DataReader<'a>,
}

impl<'a> DataHandler<'a> {
    fn new(data: &'a Vec<u8>) -> DataHandler<'a> {
        DataHandler {
            data,
            reader: DataReader::new(data),
        }
    }

    fn read_table_keys(&self, data: &Vec<u8>) -> Vec<String> {
        // create a new DataReader with the data
        let mut table_reader = DataReader::new(data);
        // process the strings
        let mut keys = vec![];
        loop {
            let mut first = table_reader.read_byte();
            // TODO properly handle lists
            if first == 0 {
                first = table_reader.read_byte();
                println!("skipped a byte");
            }
            let length = table_reader.read_gamma().0;
            if length == 0 {
                break;
            }
            let is_list = first & 0b00010000 == 0b00010000;
            let key_type = match first & 0b00001111 {
                0 => TableDataType::None,
                1 => TableDataType::Int8,
                2 => TableDataType::Uint8,
                3 => TableDataType::Int16,
                4 => TableDataType::Uint16,
                5 => TableDataType::Int32,
                6 => TableDataType::Uint32,
                7 => TableDataType::Int64,
                8 => TableDataType::Uint64,
                9 => TableDataType::StringID,
                10 => TableDataType::Str,
                11 => TableDataType::Struct,
                _ => panic!("Unknown table data type: {}", first),
            };
            let key = table_reader.read_bytes(length as usize);
            // turn the key into a string
            let key = String::from_utf8(key).unwrap();
            println!(
                "Key type: {:?}, is list: {}, key: {}, Length: {}",
                key_type, is_list, key, length
            );
            keys.push(key);
        }
        keys
    }

    fn handle_riff(&self) -> Vec<u8> {
        Vec::new()
    }
    fn handle_table(&mut self) -> Vec<u8> {
        let header_size = self.reader.read_gamma().0;
        println!("Header size: {}", header_size);
        let headers = self.reader.read_bytes(header_size as usize);
        let keys = self.read_table_keys(&headers);
        println!("Keys: {:?}", keys);

        let data_size = self.reader.read_gamma().0;
        println!("Data size: {}", data_size);
        let data = self.reader.read_bytes(data_size as usize);
        // handle the headers
        // also handle the data
        headers
    }

    fn handle_sparse_table(&self) -> Vec<u8> {
        Vec::new()
    }

    /// Read the chunk
    /// The chunk types are specified in DataHandlerChunks
    fn read_chunk(&mut self) -> SavegameChunk {
        // read the chunk label, four bytes, and convert to a string
        let chunk_label = self.reader.read_bytes(4);
        // determine the chunk type
        // JGRPP has special cases. Ignore them for now
        // TODO Handle JGRPP info
        let chunk_type = match self.reader.read_byte() {
            0 => DataHandlerChunks::Riff,
            1 => DataHandlerChunks::Array,
            2 => DataHandlerChunks::SparseArray,
            3 => DataHandlerChunks::Table,
            4 => DataHandlerChunks::SparseTable,
            _ => panic!("Unknown chunk type"),
        };
        let data = match chunk_type {
            DataHandlerChunks::Riff => self.handle_riff(),
            DataHandlerChunks::Array => panic!("Array chunks are not supported"),
            DataHandlerChunks::SparseArray => panic!("Sparse array chunks are not supported"),
            DataHandlerChunks::Table => self.handle_table(),
            DataHandlerChunks::SparseTable => self.handle_sparse_table(),
        };
        SavegameChunk {
            label: String::from_utf8(chunk_label).unwrap(),
            chunk_type,
            data: data, // just dump a bunch of random things into it!
        }
    }
}

pub struct Savegame {
    data: Vec<u8>,
    pub path: String,
    pub version: u16,
}

impl Savegame {
    pub fn new(path: String) -> Savegame {
        Savegame {
            path,
            version: 0,
            data: Vec::new(),
        }
    }

    /// Read the savegame file and decompress it.
    /// This function will set the version number and the data
    /// OpenTTD savegame uses big endian encoding
    fn load(&mut self) {
        let mut reader = FileReader::new(self.path.clone());
        let header_bytes = reader.read_bytes(4);
        let header = header_bytes.as_slice();

        let version_bytes = reader.read_bytes(2);
        let version_array: [u8; 2] = version_bytes.as_slice().try_into().unwrap();
        self.version = u16::from_be_bytes(version_array);

        reader.read_bytes(2); // skip 2 bytes

        let data = reader.read_leftover_bytes();
        self.data = match header {
            b"OTTX" => decompress_lzma(data),
            b"OTTZ" => decompress_zlib(data),
            b"OTTN" => decompress_none(data),
            b"OTTD" => panic!("LZO compression is not supported"),
            _ => panic!("Unknown compression type. Are you loading an OpenTTD savegame file?"),
        };
    }

    /// Save the savegame to a file
    pub fn save(&self, path: String) {
        let mut file = File::create(path).unwrap();
        // add the header and version
        // file.write_all(b"OTTN").unwrap();
        // file.write_all(&self.version.to_be_bytes()).unwrap();
        // file.write_all(&[0, 0]).unwrap();
        file.write_all(&self.data).unwrap();
    }

    /// Load the savegame file and process it
    pub fn process(&mut self) {
        self.load();
        let mut data_handler = DataHandler::new(&self.data);
        let a = data_handler.read_chunk();
        println!("{:?}, {}, {:02X?}", a.label, a.chunk_type as u8, a.data);
    }
}
