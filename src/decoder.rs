use crate::codes_markers::*;
use crate::huffman_tree::{HuffmanResult, HuffmanTree};
use crate::image::Image;
use crate::mcu::MCU;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufWriter, Write};

pub struct Decoder {
    img_path: String,

    img_bytes: Vec<u8>,

    huffman_tables: Vec<HuffmanTree>,
    component_id_to_huffman_table: HashMap<usize, (u8, u8)>,

    quantization_table_luma: [[u8; 8]; 8],
    quantization_table_chroma: [[u8; 8]; 8],

    img_height: usize,
    img_width: usize,

    mcus: Vec<MCU>,
}

fn next_byte<T: Iterator<Item = u8>>(it: &mut T) -> u8 {
    it.next().expect("[E] - image ended unexpectedly").clone()
}

fn next_bit<T: Iterator<Item = bool>>(it: &mut T) -> bool {
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

fn bitstring_to_value(mut val: u16, cat: u8) -> i32 {
    if cat == 0 {
        return 0;
    }
    let mask: u16 = 1 << (cat - 1);
    let neg = val & mask == 0;

    let mut res = val as i32;
    if neg {
        val = !val;
        val = val << (16 - cat) >> (16 - cat);
        res = -(val as i32);
    }

    res
}

fn u8_to_bool(val: u8) -> Vec<bool> {
    (0..8)
        .map(|i| {
            let mask = 1 << i;
            val & mask != 0
        })
        .rev()
        .collect::<Vec<_>>()
}

impl Decoder {
    pub fn new(img_path: String) -> Self {
        let img_bytes = fs::read(&img_path).expect("[E] - no such file exists");

        Self {
            img_path,
            img_bytes,

            huffman_tables: Vec::new(),
            component_id_to_huffman_table: HashMap::new(),

            quantization_table_luma: [[0; 8]; 8],
            quantization_table_chroma: [[0; 8]; 8],

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
                        JFIF_APP0 => {
                            let segment_length = 0x10 * 0x10 * next_byte(&mut img_iter) as u16
                                + next_byte(&mut img_iter) as u16;
                            let seg = next_n_bytes(&mut img_iter, segment_length as usize - 2);

                            println!("Parsing APP-O segment:");
                            println!("\tsegment length: {}", segment_length);
                            self.parse_app0(seg);
                        }
                        JFIF_DQT => {
                            let segment_length = 0x10 * 0x10 * next_byte(&mut img_iter) as u16
                                + next_byte(&mut img_iter) as u16;
                            let seg = next_n_bytes(&mut img_iter, segment_length as usize - 2);

                            println!("Parsing DQT segment:");
                            println!("\tsegment length: {}", segment_length);
                            self.parse_dqt(seg);
                        }
                        JFIF_DHT => {
                            let segment_length = 0x10 * 0x10 * next_byte(&mut img_iter) as u16
                                + next_byte(&mut img_iter) as u16;
                            let seg = next_n_bytes(&mut img_iter, segment_length as usize - 2);

                            println!("Parsing DHT segment:");
                            println!("\tsegment length: {}", segment_length);
                            self.parse_dht(seg);
                        }
                        JFIF_SOF0 => {
                            let segment_length = 0x10 * 0x10 * next_byte(&mut img_iter) as u16
                                + next_byte(&mut img_iter) as u16;
                            let seg = next_n_bytes(&mut img_iter, segment_length as usize - 2);

                            println!("Parsing SOF0 segment:");
                            println!("\tsegment length: {}", segment_length);
                            self.parse_sof(seg);
                        }
                        JFIF_SOS => {
                            let segment_length = 0x10 * 0x10 * next_byte(&mut img_iter) as u16
                                + next_byte(&mut img_iter) as u16;
                            let seg = next_n_bytes(&mut img_iter, segment_length as usize - 2);

                            println!("Parsing SOS segment:");
                            println!("\tsegment length: {}", segment_length);
                            self.parse_sos0(seg);

                            let mut rest_of_file = all_bytes(&mut img_iter);
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

                            let mut cleaned_file = Vec::new();
                            cleaned_file.push(rest_of_file[0]);

                            for i in 1..rest_of_file.len() {
                                if !(rest_of_file[i - 1] == 0xff && rest_of_file[i] == 0x00) {
                                    cleaned_file.push(rest_of_file[i]);
                                }
                            }

                            self.parse_image_data(cleaned_file);
                        }
                        JFIF_COM => println!("\nComment marker:"),
                        JFIF_EOI => println!("\nEnd of image"),
                        _ => println!("\nUnknown segment ({:02X})", byte_marker),
                    }
                }
                _ => (),
            }
        }
    }

    fn get_huffman_table_dc(&self, component_id: usize) -> &HuffmanTree {
        let (huffman_dc, _) = self.component_id_to_huffman_table[&component_id];

        self.huffman_tables
            .iter()
            .filter(|table| table.table_type == 0 && table.table_number == huffman_dc)
            .next()
            .unwrap()
    }

    fn get_huffman_table_ac(&self, component_id: usize) -> &HuffmanTree {
        let (_, huffman_ac) = self.component_id_to_huffman_table[&component_id];

        self.huffman_tables
            .iter()
            .filter(|table| table.table_type == 1 && table.table_number == huffman_ac)
            .next()
            .unwrap()
    }

    fn parse_app0(&mut self, bytes: Vec<u8>) {
        let mut img_iter = bytes.into_iter();

        let jfif_string = (0..5)
            .into_iter()
            .map(|_| next_byte(&mut img_iter))
            .collect::<Vec<u8>>();

        let jfif_string = match std::str::from_utf8(&jfif_string) {
            Ok(v) => v,
            Err(_) => {
                panic!("[E] - this JPG file may be corrupted, it does not present the 'JFIF\\0' string")
            }
        };

        if jfif_string != "JFIF\0" {
            panic!("[E] - this JPG file may be corrupted, it does not present the 'JFIF\\0' string")
        }

        let vers_major = next_byte(&mut img_iter);
        let vers_minor = next_byte(&mut img_iter);

        let density_unit = next_byte(&mut img_iter);
        let density_horizontal =
            0x10 * 0x10 * next_byte(&mut img_iter) as u16 + next_byte(&mut img_iter) as u16;
        let density_vertical =
            0x10 * 0x10 * next_byte(&mut img_iter) as u16 + next_byte(&mut img_iter) as u16;
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

    fn parse_dqt(&mut self, bytes: Vec<u8>) {
        let mut img_iter = bytes.into_iter();

        let prec_dest_byte = next_byte(&mut img_iter);
        let precision = prec_dest_byte >> 4;
        let table_type = prec_dest_byte << 4 >> 4;

        let mut quantization_table = [[0u8; 8]; 8];
        for i in 0..8 {
            for j in 0..8 {
                quantization_table[i][j] = next_byte(&mut img_iter);
            }
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
                print!("{} ", quantization_table[i][j]);
            }
            println!();
        }
    }

    fn parse_dht(&mut self, bytes: Vec<u8>) {
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
        //        println!("\thuffman table lengths:");
        //        print!("\t\t");
        //        for i in lengths.iter() {
        //            print!("{} ", i);
        //        }
        //        println!();
        //        println!("\thuffman table characters:");
        //        print!("\t\t");
        //        for i in characters.iter() {
        //            print!("{:02X} ", i);
        //        }
        //        self.huffman_tables.last().unwrap().print();
        //        println!();
    }

    fn parse_sof(&mut self, bytes: Vec<u8>) {
        let mut img_iter = bytes.into_iter();

        let precision = next_byte(&mut img_iter);
        let img_height =
            0x10 * 0x10 * next_byte(&mut img_iter) as usize + next_byte(&mut img_iter) as usize;
        let img_width =
            0x10 * 0x10 * next_byte(&mut img_iter) as usize + next_byte(&mut img_iter) as usize;

        self.img_height = img_height as usize;
        self.img_width = img_width as usize;

        println!("\tprecision: {}", precision);
        println!("\timage size: {}x{}", img_height, img_width);
    }

    fn parse_sos0(&mut self, bytes: Vec<u8>) {
        let mut img_iter = bytes.into_iter();

        let component_count = next_byte(&mut img_iter);
        if component_count != 3 {
            panic!(
                "[E] - component number in SOS segment is wrong ({} != 3)",
                component_count
            );
        }

        println!("\thuffman tables for components:");
        for _ in 0..component_count {
            let mut component_id = next_byte(&mut img_iter);
            component_id -= 1;
            let huffman_table = next_byte(&mut img_iter);
            let huffman_table_dc = huffman_table >> 4;
            let huffman_table_ac = huffman_table << 4 >> 4;

            println!(
                "\t\t{} -> {} (dc), {} (ac)",
                component_id, huffman_table_dc, huffman_table_ac
            );

            self.component_id_to_huffman_table
                .insert(component_id as usize, (huffman_table_dc, huffman_table_ac));
        }
    }

    fn parse_image_data(&mut self, img_data: Vec<u8>) {
        let file_idct = File::create("rust_idct.log").unwrap();
        let mut buffer_idct = BufWriter::new(file_idct);
        let file_dct = File::create("rust_dct.log").unwrap();
        let mut buffer_dct = BufWriter::new(file_dct);

        let mcu_count = self.img_width * self.img_height / 64 as usize;

        let mut img_bits = Vec::new();
        for byte in img_data.iter() {
            let bits = u8_to_bool(*byte);
            bits.into_iter().for_each(|b| img_bits.push(b));
        }

        let mut img_bits_iter = img_bits.into_iter();
        let mut dc_sums = [0; 3];
        let mut next_dc_sums = [0; 3];

        for mcu_id in 0..mcu_count {
            let mut run_length_encoding: Vec<Vec<i32>> = vec![Vec::new(), Vec::new(), Vec::new()];

            for comp_id in 0..3 {
                // decode DC
                let huffman_table = self.get_huffman_table_dc(comp_id);
                let mut scanned_bits = Vec::new();
                loop {
                    scanned_bits.push(next_bit(&mut img_bits_iter));

                    match huffman_table.try_decode(&scanned_bits) {
                        HuffmanResult::Some(val) => {
                            let zero_count = val >> 4;
                            let category = val << 4 >> 4;
                            let mut dc_coeff: u16 = 0;

                            for _ in 0..category {
                                dc_coeff = dc_coeff << 1;
                                dc_coeff += next_bit(&mut img_bits_iter) as u16;
                            }

                            let dc_coeff = bitstring_to_value(dc_coeff, category);

                            next_dc_sums[comp_id] += dc_coeff;

                            run_length_encoding[comp_id].push(zero_count as i32);
                            run_length_encoding[comp_id].push(dc_coeff);
                            break;
                        }
                        HuffmanResult::EOB => {
                            run_length_encoding[comp_id].push(0);
                            run_length_encoding[comp_id].push(0);
                            break;
                        }
                        HuffmanResult::None => (),
                    }
                }

                // decode AC
                let huffman_table = self.get_huffman_table_ac(comp_id);
                let mut scanned_bits = Vec::new();
                let mut ac_count = 0;
                loop {
                    if ac_count == 63 {
                        break;
                    }
                    scanned_bits.push(next_bit(&mut img_bits_iter));

                    match huffman_table.try_decode(&scanned_bits) {
                        HuffmanResult::Some(val) => {
                            let zero_count = val >> 4;
                            let category = val << 4 >> 4;
                            let mut ac_coeff: u16 = 0;

                            for _ in 0..category {
                                ac_coeff = ac_coeff << 1;
                                ac_coeff += next_bit(&mut img_bits_iter) as u16;
                            }

                            let ac_coeff = bitstring_to_value(ac_coeff, category);

                            run_length_encoding[comp_id].push(zero_count as i32);
                            run_length_encoding[comp_id].push(ac_coeff);

                            scanned_bits = Vec::new();
                            ac_count += zero_count + 1;
                        }
                        HuffmanResult::EOB => {
                            run_length_encoding[comp_id].push(0);
                            run_length_encoding[comp_id].push(0);
                            break;
                        }
                        HuffmanResult::None => (),
                    }
                }
            }
            let mut mcu = MCU::new(mcu_id, run_length_encoding, dc_sums, &mut buffer_dct);
            mcu.build_rgb_block(
                &self.quantization_table_luma,
                &self.quantization_table_chroma,
                &mut buffer_idct,
            );
            self.mcus.push(mcu);

            dc_sums = next_dc_sums;
        }

        buffer_idct.flush().unwrap();
        buffer_dct.flush().unwrap();

        let mut img = Image::new(self.img_width, self.img_height);
        img.build_from_mcus(&self.mcus);
        img.dump_to_ppm("test.ppm").unwrap();
    }
}
