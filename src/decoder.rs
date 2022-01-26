use crate::huffman_tree::HuffmanTree;
use crate::mcu::MCU;
use std::fs;

pub struct Decoder {
    img_path: String,

    img_bytes: Vec<u8>,

    huffman_luma_dc: HuffmanTree,
    huffman_luma_ac: HuffmanTree,
    huffman_chroma_ac: HuffmanTree,
    huffman_chroma_dc: HuffmanTree,

    mcus: Vec<MCU>,
}

fn next_byte<T: Iterator<Item = u8>>(it: &mut T) -> u8 {
    it.next().expect("[E] - image ended unexpectedly").clone()
}

fn next_n_bytes<T: Iterator<Item = u8>>(it: &mut T, length: usize) -> Vec<u8> {
    (0..length).into_iter().map(|_| next_byte(it)).collect()
}

impl Decoder {
    pub fn new(img_path: String) -> Self {
        let img_bytes = fs::read(&img_path).expect("[E] - no such file exists");

        Self {
            img_path,
            img_bytes,

            huffman_luma_dc: HuffmanTree::new(),
            huffman_luma_ac: HuffmanTree::new(),
            huffman_chroma_dc: HuffmanTree::new(),
            huffman_chroma_ac: HuffmanTree::new(),

            mcus: vec![],
        }
    }

    pub fn debug(&mut self) {
        let mut img_iter = self.img_bytes.clone().into_iter();

        if !(next_byte(&mut img_iter) == 0xff && next_byte(&mut img_iter) == 0xd8) {
            println!("[E] - file provided is not a jpg image");
        }

        while let Some(byte) = img_iter.next() {
            match byte {
                0xff => {
                    let byte_marker = next_byte(&mut img_iter);
                    match byte_marker {
                        0xe0 => {
                            let segment_length =
                                next_byte(&mut img_iter) + next_byte(&mut img_iter);
                            let seg = next_n_bytes(&mut img_iter, segment_length as usize - 2);

                            println!("Parsing APP-O segment:");
                            println!("\tsegment length: {}", segment_length);
                            self.parse_APP0(seg);
                        }
                        0xdb => {
                            let segment_length =
                                next_byte(&mut img_iter) + next_byte(&mut img_iter);
                            let seg = next_n_bytes(&mut img_iter, segment_length as usize - 2);

                            println!("Parsing DQT segment:");
                            println!("\tsegment length: {}", segment_length);
                            self.parse_DQT(seg);
                        }
                        0xc4 => {
                            let segment_length =
                                next_byte(&mut img_iter) + next_byte(&mut img_iter);
                            let seg = next_n_bytes(&mut img_iter, segment_length as usize - 2);

                            println!("Parsing DHT segment:");
                            println!("\tsegment length: {}", segment_length);
                            self.parse_DHT(seg);
                        }
                        0xc0 => {
                            let segment_length =
                                next_byte(&mut img_iter) + next_byte(&mut img_iter);
                            let seg = next_n_bytes(&mut img_iter, segment_length as usize - 2);

                            println!("Parsing SOF segment:");
                            println!("\tsegment length: {}", segment_length);
                            self.parse_SOF(seg);
                        }
                        0xda => println!("\nSOS:"),
                        0xfe => println!("\nComment marker:"),
                        0xd9 => println!("\nEnd of image"),
                        _ => println!("\nUnknown segment ({:02X})", byte_marker),
                    }
                }
                _ => (),
            }
        }
    }

    fn parse_APP0(&mut self, bytes: Vec<u8>) {
        let mut img_iter = bytes.into_iter();

        let jfif_string = (0..5)
            .into_iter()
            .map(|_| next_byte(&mut img_iter))
            .collect::<Vec<u8>>();

        let jfif_string = match std::str::from_utf8(&jfif_string) {
            Ok(v) => v,
            Err(e) => {
                panic!("[E] - this jpg file may be corrupted, it does not present the 'JFIF\\0' string")
            }
        };

        if jfif_string != "JFIF\0" {
            panic!("[E] - this jpg file may be corrupted, it does not present the 'JFIF\\0' string")
        }

        let vers_major = next_byte(&mut img_iter);
        let vers_minor = next_byte(&mut img_iter);

        let density_unit = next_byte(&mut img_iter);
        let density_horizontal = next_byte(&mut img_iter) + next_byte(&mut img_iter);
        let density_vertical = next_byte(&mut img_iter) + next_byte(&mut img_iter);
        let thumbnail_horizontal = next_byte(&mut img_iter);
        let thumbnail_vertical = next_byte(&mut img_iter);

        if thumbnail_vertical != 0 {
            // TODO : deal thumbnail if it exists
            todo!("must deal thumbnail")
        }

        println!("\tversion: {}.{}", vers_major, vers_minor);
        println!(
            "\tdensity unit: {} (00 for no units, 01 for pixels per inch, 02 for pixels per cm)",
            density_unit
        );
        println!(
            "\tpixel density: {}x{}",
            density_horizontal, density_vertical
        );
        println!(
            "\tthumbnail size: {}x{}",
            thumbnail_horizontal, thumbnail_vertical
        );
    }

    fn parse_DQT(&mut self, bytes: Vec<u8>) {
        let mut img_iter = bytes.into_iter();

        let prec_dest_byte = next_byte(&mut img_iter);
        let precision = prec_dest_byte >> 4;
        let destination = prec_dest_byte << 4 >> 4;

        let mut quantization_table = [0u8; 64];
        for i in 0..64 {
            quantization_table[i] = next_byte(&mut img_iter);
        }

        println!("\tprecision: {} (0 = 8-bit, 1 = 16-bit)", precision);
        println!(
            "\tdestination: {} (0 = luminance, 1 = chrominance)",
            destination
        );
        println!("\tquantization table:");
        for i in 0..8 {
            print!("\t\t");
            for j in 0..8 {
                print!("{} ", quantization_table[8 * i + j]);
            }
            println!();
        }
    }

    fn parse_DHT(&mut self, bytes: Vec<u8>) {
        let mut img_iter = bytes.into_iter();

        let table_info = next_byte(&mut img_iter);
        let class = table_info >> 4;
        let destination = table_info << 4 >> 4;

        let mut characters_len = 0;
        let mut characters = Vec::new();
        let mut lengths = Vec::new();
        let mut zeros_len = 0;
        for _ in 0..16 {
            zeros_len += 1;
            lengths.push(next_byte(&mut img_iter));
            characters_len += lengths.last().unwrap();

            if *lengths.last().unwrap() != 0 {
                zeros_len = 0;
            }
        }

        lengths.truncate(16 - zeros_len);

        for _ in 0..characters_len {
            characters.push(next_byte(&mut img_iter));
        }

        println!("\tclass: {} (0 for DC, 1 for AC)", class);
        println!("\tdestination: {}", destination);
        println!("\thuffman table lengths:");
        print!("\t\t");
        for i in lengths.iter() {
            print!("{} ", i);
        }
        println!();
        println!("\thuffman table characters:");
        print!("\t\t");
        for i in characters.iter() {
            print!("{:02X} ", i);
        }
        println!();
    }

    fn parse_SOF(&mut self, bytes: Vec<u8>) {
        let mut img_iter = bytes.into_iter();

        let precision = next_byte(&mut img_iter);
        let img_height = next_byte(&mut img_iter) + next_byte(&mut img_iter);
        let img_width = next_byte(&mut img_iter) + next_byte(&mut img_iter);

        println!("\tprecision: {}", precision);
        println!("\timage size: {}x{}", img_height, img_width);
    }
}
