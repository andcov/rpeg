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

    pub fn debug(&self) {
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
                            for _ in 0..5 {
                                // for "JFIF\0" text
                                next_byte(&mut img_iter);
                            }
                            let vers_major = next_byte(&mut img_iter);
                            let vers_minor = next_byte(&mut img_iter);

                            let density_unit = next_byte(&mut img_iter);
                            let density_horizontal =
                                next_byte(&mut img_iter) + next_byte(&mut img_iter);
                            let density_vertical =
                                next_byte(&mut img_iter) + next_byte(&mut img_iter);
                            let thumbnail_horizontal = next_byte(&mut img_iter);
                            let thumbnail_vertical = next_byte(&mut img_iter);

                            if thumbnail_vertical != 0 {
                                // TODO : read thumbnail if it exists
                                todo!("must read thumbnail")
                            }

                            println!("APP-O:");
                            println!("\tsegment length: {}", segment_length);
                            println!("\tversion: {}.{}", vers_major, vers_minor);
                            println!("\tdensity unit: {} (00 for no units, 01 for pixels per inch, 02 for pixels per cm)", density_unit);
                            println!(
                                "\tpixel density: {}x{}",
                                density_horizontal, density_vertical
                            );
                            println!(
                                "\tthumbnail size: {}x{}",
                                thumbnail_horizontal, thumbnail_vertical
                            );
                        }
                        0xdb => {
                            let segment_length =
                                next_byte(&mut img_iter) + next_byte(&mut img_iter);

                            let prec_dest_byte = next_byte(&mut img_iter);
                            let precision = prec_dest_byte >> 4;
                            let destination = prec_dest_byte << 4 >> 4;

                            let mut quantization_table = [0u8; 64];
                            for i in 0..64 {
                                quantization_table[i] = next_byte(&mut img_iter);
                            }

                            println!("DQT:");
                            println!("\tsegment length: {}", segment_length);
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
                        0xc4 => {
                            let segment_length =
                                next_byte(&mut img_iter) + next_byte(&mut img_iter);
                            let table_info = next_byte(&mut img_iter);
                            let class = table_info >> 4;
                            let destination = table_info << 4 >> 4;

                            let mut table = vec![];
                            for _ in 0..(segment_length - 3) {
                                table.push(next_byte(&mut img_iter));
                            }

                            println!("DHT:");
                            println!("\tsegment length: {}", segment_length);
                            println!("\tclass: {} (0 for DC, 1 for AC)", class);
                            println!("\tdestination: {}", destination);
                            println!("\thuffman table:");
                            print!("\t\t");
                            for i in table.iter() {
                                print!("{:02X} ", i);
                            }
                            println!();
                        }
                        0xc0 => {
                            let segment_length =
                                next_byte(&mut img_iter) + next_byte(&mut img_iter);

                            let precision = next_byte(&mut img_iter);
                            let img_height = next_byte(&mut img_iter) + next_byte(&mut img_iter);
                            let img_width = next_byte(&mut img_iter) + next_byte(&mut img_iter);

                            println!("\nSOF:");
                            println!("\tsegment length: {}", segment_length);
                            println!("\tprecision: {}", precision);
                            println!("\timage size: {}x{}", img_height, img_width);
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
}
