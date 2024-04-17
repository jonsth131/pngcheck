pub const HEADER: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];

pub const CRITICAL_CHUNKS: [&str; 4] = ["IHDR", "PLTE", "IDAT", "IEND"];

pub const ANCILLARY_CHUNKS: [&str; 18] = [
    "bKGD", "cHRM", "cICP", "dSIG", "eXIf", "gAMA", "hIST", "iCCP", "iTXt", "pHYs", "sBIT", "sPLT",
    "sRGB", "sTER", "tEXt", "tIME", "tRNS", "zTXt",
];

pub struct Chunk {
    pub length: u32,
    pub chunk_type: String,
    pub data: Option<Vec<u8>>,
    pub crc: u32,
}

#[derive(Debug)]
pub struct IHDR {
    pub width: u32,
    pub height: u32,
    pub bit_depth: u8,
    pub color_type: u8,
    pub compression_method: u8,
    pub filter_method: u8,
    pub interlace_method: u8,
}
