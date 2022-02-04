pub const JFIF_BYTE_FF: u8 = 0xFF; // All markers start with this as the MSB
pub const JFIF_SOF0: u8 = 0xC0; // Start of Frame 0, Baseline DCT
pub const JFIF_SOF1: u8 = 0xC1; // Start of Frame 1, Extended Sequential DCT
pub const JFIF_SOF2: u8 = 0xC2; // Start of Frame 2, Progressive DCT
pub const JFIF_SOF3: u8 = 0xC3; // Start of Frame 3, Lossless (Sequential)
pub const JFIF_DHT: u8 = 0xC4; // Define Huffman Table
pub const JFIF_SOF5: u8 = 0xC5; // Start of Frame 5, Differential Sequential DCT
pub const JFIF_SOF6: u8 = 0xC6; // Start of Frame 6, Differential Progressive DCT
pub const JFIF_SOF7: u8 = 0xC7; // Start of Frame 7, Differential Loessless (Sequential)
pub const JFIF_SOF9: u8 = 0xC9; // Extended Sequential DCT, Arithmetic Coding
pub const JFIF_SOF10: u8 = 0xCA; // Progressive DCT, Arithmetic Coding
pub const JFIF_SOF11: u8 = 0xCB; // Lossless (Sequential), Arithmetic Coding
pub const JFIF_SOF13: u8 = 0xCD; // Differential Sequential DCT, Arithmetic Coding
pub const JFIF_SOF14: u8 = 0xCE; // Differential Progressive DCT, Arithmetic Coding
pub const JFIF_SOF15: u8 = 0xCF; // Differential Lossless (Sequential), Arithmetic Coding
pub const JFIF_SOI: u8 = 0xD8; // Start of Image
pub const JFIF_EOI: u8 = 0xD9; // End of Image
pub const JFIF_SOS: u8 = 0xDA; // Start of Scan
pub const JFIF_DQT: u8 = 0xDB; // Define Quantization Table
pub const JFIF_APP0: u8 = 0xE0; // Application Segment 0, JPEG-JFIF Image
pub const JFIF_COM: u8 = 0xFE; // Comment

pub const HUFFMAN_DC: u8 = 0; // DC value for DHT
pub const HUFFMAN_AC: u8 = 1; // AC value for DHT

pub const CHANNEL_LUMA: u8 = 0; // Code for luminance channel
pub const CHANNEL_CHROMA: u8 = 1; // Code for chrominance channel
