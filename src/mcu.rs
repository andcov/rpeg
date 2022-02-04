use std::f64::consts::PI;
use std::f64::consts::SQRT_2;
use std::fs::File;
use std::io::BufWriter;
use std::io::Write;

pub struct MCU {
    order: usize,
    dc_sums: [i32; 3],
    pub zz_dct_coeff: [[i32; 64]; 3],
    pub idct_coeff: [[[f64; 8]; 8]; 3],
    pub rgb: [[[u8; 8]; 8]; 3],
}

const TABLE: [[usize; 8]; 8] = [
    [0, 1, 5, 6, 14, 15, 27, 28],
    [2, 4, 7, 13, 16, 26, 29, 42],
    [3, 8, 12, 17, 25, 30, 41, 43],
    [9, 11, 18, 24, 31, 40, 44, 53],
    [10, 19, 23, 32, 39, 45, 52, 54],
    [20, 22, 33, 38, 46, 51, 55, 60],
    [21, 34, 37, 47, 50, 56, 59, 61],
    [35, 36, 48, 49, 57, 58, 62, 63],
];

fn two_coord_to_lin_coord(x: usize, y: usize) -> usize {
    if x > 7 {
        panic!("[E] - index out of bounds ({} is not in [0, 7])", x);
    }
    if y > 7 {
        panic!("[E] - index out of bounds ({} is not in [0, 7])", y);
    }

    TABLE[y][x]
}

fn lin_coord_to_two_coord(i: usize) -> (usize, usize) {
    if i > 63 {
        panic!("[E] - index out of bounds ({} is not in [0, 63])", i);
    }
    for y in 0..8 {
        for x in 0..8 {
            if TABLE[y][x] == i {
                return (x, y);
            }
        }
    }
    (0, 0)
}

impl MCU {
    pub fn new(
        mcu_order: usize,
        run_length_encoding: Vec<Vec<i32>>,
        dc_sums: [i32; 3],
        buff: &mut BufWriter<File>,
    ) -> Self {
        if run_length_encoding.len() != 3 {
            panic!(
                "[E] - you need 3 components to build an MCU ({} != 3)",
                run_length_encoding.len()
            );
        }

        let mut zz_dct_coeff = [[0; 64]; 3];

        writeln!(buff, "{}", mcu_order + 1).unwrap();

        for (comp_id, comp) in run_length_encoding.iter().enumerate() {
            let mut index = 0;
            for i in (0..comp.len()).step_by(2) {
                let zeros = comp[i] as usize;
                let val = comp[i + 1];

                if zeros == 0 && val == 0 {
                    break;
                }

                //TODO: do not iterate over the already existing zeros, just change the index var
                for _ in 0..zeros {
                    zz_dct_coeff[comp_id][index] = 0;
                    index += 1;
                }

                zz_dct_coeff[comp_id][index] = val;
                index += 1;
            }
            zz_dct_coeff[comp_id][0] += dc_sums[comp_id];

            write!(buff, "{} ", comp_id).unwrap();
            for i in 0..64 {
                write!(buff, "{} ", zz_dct_coeff[comp_id][i]).unwrap();
            }
            writeln!(buff).unwrap();
        }

        Self {
            order: mcu_order,
            dc_sums,
            zz_dct_coeff,
            idct_coeff: [[[0.0; 8]; 8]; 3],
            rgb: [[[0; 8]; 8]; 3],
        }
    }

    pub fn build_rgb_block(
        &mut self,
        quantization_luma: &[[u8; 8]; 8],
        quantization_chroma: &[[u8; 8]; 8],
        b: &mut BufWriter<File>,
    ) {
        self.compute_idct(quantization_luma, quantization_chroma, b);
        self.level_shift();
        self.convert_ycbcr_to_rgb();
    }

    fn compute_idct(
        &mut self,
        quantization_luma: &[[u8; 8]; 8],
        quantization_chroma: &[[u8; 8]; 8],
        b: &mut BufWriter<File>,
    ) {
        let mut temp = [[[0; 8]; 8]; 3];
        for comp_id in 0..3 {
            for i in 0..64 {
                let (coord_x, coord_y) = lin_coord_to_two_coord(i);
                if comp_id == 0 {
                    temp[comp_id][coord_y][coord_x] =
                        self.zz_dct_coeff[comp_id][i] * quantization_luma[i / 8][i % 8] as i32;
                } else {
                    temp[comp_id][coord_y][coord_x] =
                        self.zz_dct_coeff[comp_id][i] * quantization_chroma[i / 8][i % 8] as i32;
                }
            }
        }
        //        for comp_id in 0..3 {
        //            for y in 0..8 {
        //                for x in 0..8 {
        //                    temp[comp_id][y][x] = self.zz_dct_coeff[comp_id][two_coord_to_lin_coord(x, y)];
        //                    if comp_id == 0 {
        //                        temp[comp_id][y][x] *= quantization_luma[x][y] as i32;
        //                    } else {
        //                        temp[comp_id][y][x] *= quantization_chroma[x][y] as i32;
        //                    }
        //                }
        //            }
        //        }

        //        for comp_id in 0..3 {
        //            writeln!(b, "{}", comp_id).unwrap();
        //            for x in 0..8 {
        //                for y in 0..8 {
        //                    write!(b, "{} ", temp[comp_id][x][y]).unwrap();
        //                }
        //                writeln!(b).unwrap();
        //            }
        //            writeln!(b).unwrap();
        //        }

        for comp_id in 0..3 {
            for y in 0..8 {
                for x in 0..8 {
                    let mut sum: f64 = 0.0;
                    for u in 0..8 {
                        for v in 0..8 {
                            let cu = if u == 0 { 1.0 / SQRT_2 } else { 1.0 };
                            let cv = if v == 0 { 1.0 / SQRT_2 } else { 1.0 };
                            sum += temp[comp_id][u][v] as f64
                                * cu
                                * cv
                                * f64::cos(((2 * x + 1) * u) as f64 * PI / 16.0)
                                * f64::cos(((2 * y + 1) * v) as f64 * PI / 16.0);
                        }
                    }
                    self.idct_coeff[comp_id][x][y] = sum / 4.0;
                }
            }
        }
        for comp_id in 0..3 {
            writeln!(b, "{}", comp_id).unwrap();
            for x in 0..8 {
                for y in 0..8 {
                    write!(b, "{} ", self.idct_coeff[comp_id][x][y] as i32).unwrap();
                }
                writeln!(b).unwrap();
            }
            writeln!(b).unwrap();
        }
    }

    fn level_shift(&mut self) {
        for comp_id in 0..3 {
            for y in 0..8 {
                for x in 0..8 {
                    self.rgb[comp_id][y][x] =
                        (self.idct_coeff[comp_id][y][x] + 128.0).clamp(0.0, 255.0) as u8;
                }
            }
        }
    }

    fn convert_ycbcr_to_rgb(&mut self) {
        for i in 0..8 {
            for j in 0..8 {
                let y = self.rgb[0][i][j] as f64;
                let cb = self.rgb[1][i][j] as f64;
                let cr = self.rgb[2][i][j] as f64;

                let r = y + 1.402 * (cr - 128.0);
                let g = y - 0.344136 * (cb - 128.0) - 0.714136 * (cr - 128.0);
                let b = y + 1.772 * (cb - 128.0);

                self.rgb[0][i][j] = r.clamp(0.0, 255.0).round() as u8;
                self.rgb[1][i][j] = g.clamp(0.0, 255.0).round() as u8;
                self.rgb[2][i][j] = b.clamp(0.0, 255.0).round() as u8;
            }
        }
    }

    pub fn print(&self) {
        println!("MCU {}", self.order);
        for i in 0..3 {
            for x in 0..8 {
                print!("\t");
                for y in 0..8 {
                    print!("{} ", self.idct_coeff[i][x][y]);
                }
                println!();
            }
            println!();
        }
    }
    pub fn print_rgb(&self) {
        println!("MCU {}", self.order);
        for i in 0..3 {
            for x in 0..8 {
                print!("\t");
                for y in 0..8 {
                    print!("{} ", self.rgb[i][x][y]);
                }
                println!();
            }
            println!();
        }
    }
}
