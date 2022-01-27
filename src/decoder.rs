use crate::codes_markers::{JFIF_BYTE_FF, JFIF_EOI};
use crate::huffman_tree::HuffmanTree;
use crate::mcu::MCU;
use std::collections::HashMap;
use std::fs;

pub struct Decoder {
    img_path: String,

    img_bytes: Vec<u8>,

    huffman_tables: Vec<HuffmanTree>,
    component_id_to_huffman_table: HashMap<usize, (u8, u8)>,

    quantization_table_luma: [u8; 64],
    quantization_table_chroma: [u8; 64],

    img_height: usize,
    img_width: usize,

    mcus: Vec<MCU>,
}

fn next_byte<T: Iterator<Item = u8>>(it: &mut T) -> u8 {
    it.next().expect("[E] - image ended unexpectedly").clone()
}

fn next_n_bytes<T: Iterator<Item = u8>>(it: &mut T, length: usize) -> Vec<u8> {
    (0..length).into_iter().map(|_| next_byte(it)).collect()
}

fn all_bytes<T: Iterator<Item = u8>>(it: &mut T) -> Vec<u8> {
    let mut res = Vec::new();
    while let Some(val) = it.next() {
        res.push(val);
    }

    res
}

impl Decoder {
    pub fn new(img_path: String) -> Self {
        let img_bytes = fs::read(&img_path).expect("[E] - no such file exists");

        Self {
            img_path,
            img_bytes,

            huffman_tables: Vec::new(),
            component_id_to_huffman_table: HashMap::new(),

            quantization_table_luma: [0; 64],
            quantization_table_chroma: [0; 64],

            img_height: 0,
            img_width: 0,

            mcus: Vec::new(),
        }
    }

    pub fn debug(&mut self) {
        let mut img_iter = self.img_bytes.clone().into_iter();

        if !(next_byte(&mut img_iter) == 0xff && next_byte(&mut img_iter) == 0xd8) {
            println!("[E] - file provided is not a JPG image");
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
                        0xda => {
                            let segment_length =
                                next_byte(&mut img_iter) + next_byte(&mut img_iter);
                            let seg = next_n_bytes(&mut img_iter, segment_length as usize - 2);

                            println!("Parsing SOS segment:");
                            println!("\tsegment length: {}", segment_length);
                            self.parse_SOS(seg);

                            let rest_of_file = all_bytes(&mut img_iter);
                            let mut end_of_img = rest_of_file.len() - 2;

                            for p in (1..rest_of_file.len()).rev() {
                                if rest_of_file[p] == JFIF_EOI
                                    && rest_of_file[p - 1] == JFIF_BYTE_FF
                                {
                                    end_of_img = p - 1;
                                    break;
                                }
                            }

                            rest_of_file.truncate(end_of_img);

                            self.parse_image_data(rest_of_file);
                        }
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
                panic!("[E] - this JPG file may be corrupted, it does not present the 'JFIF\\0' string")
            }
        };

        if jfif_string != "JFIF\0" {
            panic!("[E] - this JPG file may be corrupted, it does not present the 'JFIF\\0' string")
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
        let table_type = prec_dest_byte << 4 >> 4;

        let mut quantization_table = [0u8; 64];
        for i in 0..64 {
            quantization_table[i] = next_byte(&mut img_iter);
        }

        if table_type == 0 {
            self.quantization_table_luma = quantization_table;
        } else {
            self.quantization_table_chroma = quantization_table;
        }

        println!("\tprecision: {} (0 = 8-bit, 1 = 16-bit)", precision);
        println!("\ttype: {} (0 = luminance, 1 = chrominance)", table_type);
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
        let table_type = table_info >> 4;
        let table_number = table_info << 4 >> 4;

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

        let mut huffman_tree = HuffmanTree::new(table_type, table_number);
        huffman_tree.build(&lengths, &characters);

        self.huffman_tables.push(huffman_tree);

        println!("\ttable type: {} (0 for DC, 1 for AC)", table_type);
        println!("\ttable number: {}", table_number);
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

        self.img_height = img_height as usize;
        self.img_width = img_width as usize;

        println!("\tprecision: {}", precision);
        println!("\timage size: {}x{}", img_height, img_width);
    }

    fn parse_SOS(&mut self, bytes: Vec<u8>) {
        let mut img_iter = bytes.into_iter();

        let component_count = next_byte(&mut img_iter);
        if component_count != 3 {
            panic!(
                "[E] - component number in SOS segment is wrong ({} != 3)",
                component_count
            );
        }

        for i in 0..component_count {
            let component_id = next_byte(&mut img_iter);
            let huffman_table = next_byte(&mut img_iter);
            let huffman_table_dc = huffman_table >> 4;
            let huffman_table_ac = huffman_table << 4 >> 4;

            self.component_id_to_huffman_table
                .insert(component_id as usize, (huffman_table_dc, huffman_table_ac));
        }
    }

    fn parse_image_data(&mut self, img_data: Vec<u8>) {
        let mcu_num = self.img_width * self.img_height / 64 as usize;
    }
}
