use std::f64::consts::PI;
use std::f64::consts::SQRT_2;

pub struct MCU {
    order: usize,
    dc_sums: [i32; 3],
    dct_coeff: [[i32; 64]; 3],
    idct_coeff: [[i32; 64]; 3],
}
fn two_coord_to_lin_coord(x: usize, y: usize) -> usize {
    if x < 0 || x > 7 {
        panic!("[E] - index out of bounds ({} is not in [0, 7])", x);
    }
    if y < 0 || y > 7 {
        panic!("[E] - index out of bounds ({} is not in [0, 7])", y);
    }
    let table = [
        [0, 1, 5, 6, 14, 15, 27, 28],
        [2, 4, 7, 13, 16, 26, 29, 42],
        [3, 8, 12, 17, 25, 30, 41, 43],
        [9, 11, 18, 24, 31, 40, 44, 53],
        [10, 19, 23, 32, 39, 45, 52, 54],
        [20, 22, 33, 38, 46, 51, 55, 60],
        [21, 34, 37, 47, 50, 56, 59, 61],
        [35, 36, 48, 49, 57, 58, 62, 63],
    ];

    table[x][y]
}

fn lin_coord_to_two_coord(i: usize) -> (usize, usize) {
    (0, 0)
}

impl MCU {
    pub fn new(mcu_order: usize, run_length_encoding: Vec<Vec<i32>>, dc_sums: [i32; 3]) -> Self {
        if run_length_encoding.len() != 3 {
            panic!(
                "[E] - you need 3 components to build an MCU ({} != 3)",
                run_length_encoding.len()
            );
        }

        let mut dct_coeff = [[0; 64]; 3];
        let idct_coeff = [[0; 64]; 3];

        for (comp_id, comp) in run_length_encoding.iter().enumerate() {
            let mut index = 0;
            for i in (0..comp.len()).step_by(2) {
                let zeros = comp[i] as usize;
                let val = comp[i + 1];
                for _ in 0..zeros {
                    dct_coeff[comp_id][index] = 0;
                    index += 1;
                }
                dct_coeff[comp_id][index] = val;
                index += 1;
            }
            dct_coeff[comp_id][0] += dc_sums[comp_id];
        }

        Self {
            order: mcu_order,
            dc_sums,
            dct_coeff,
            idct_coeff,
        }
    }

    //pub fn build_idct(&mut self, quantization_dc: [i32; 64], quantization_ac: [i32; 64]) {
    pub fn build_idct(&mut self) {
        for comp_id in 0..3 {
            for x in 0..7 {
                for y in 0..7 {
                    let mut sum: f64 = 0.0;
                    for u in 0..7 {
                        for v in 0..7 {
                            let mut dct_val =
                                self.dct_coeff[comp_id][two_coord_to_lin_coord(u, v)] as f64;
                            let cos_val = f64::cos(((2 * x + 1) * u) as f64 * PI / 16.0)
                                * f64::cos(((2 * y + 1) * v) as f64 * PI / 16.0);
                            if u == 0 {
                                dct_val /= SQRT_2;
                            }
                            if v == 0 {
                                dct_val /= SQRT_2;
                            }
                            dct_val *= cos_val;
                            sum += dct_val;
                        }
                    }
                    sum /= 4.0;
                    self.idct_coeff[comp_id][two_coord_to_lin_coord(x, y)] = sum as i32;
                }
            }
        }
    }

    pub fn print(&self) {
        println!("MCU {}", self.order);
        for i in 0..3 {
            print!("\t");
            for j in 0..64 {
                print!("{} ", self.idct_coeff[i][j]);
            }
            println!();
        }
    }
}
