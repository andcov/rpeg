use crate::mcu::MCU;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufWriter;

pub struct Image {
    width: usize,
    height: usize,
    image_rgb: [Vec<Vec<u8>>; 3],
}

impl Image {
    pub fn new(width: usize, height: usize) -> Self {
        let image_rgb = vec![vec![0; width]; height];

        Self {
            width,
            height,
            image_rgb: [image_rgb.clone(), image_rgb.clone(), image_rgb.clone()],
        }
    }

    pub fn build_from_mcus(&mut self, mcus: &Vec<MCU>) {
        let apparent_width = if self.width % 8 == 0 {
            self.width
        } else {
            (self.width / 8 + 1) * 8
        };
        for (mcu_id, mcu) in mcus.iter().enumerate() {
            let x_0 = (mcu_id * 8) % apparent_width;
            let y_0 = (mcu_id * 8) / apparent_width * 8;

            for dx in 0..8 {
                for dy in 0..8 {
                    if y_0 + dy < self.height && x_0 + dx < self.width {
                        self.image_rgb[0][y_0 + dy][x_0 + dx] = mcu.rgb[0][dy][dx];
                        self.image_rgb[1][y_0 + dy][x_0 + dx] = mcu.rgb[1][dy][dx];
                        self.image_rgb[2][y_0 + dy][x_0 + dx] = mcu.rgb[2][dy][dx];
                    }
                }
            }
        }
    }

    pub fn dump_to_ppm(&self, path: &str) -> std::io::Result<()> {
        let file = File::create(path)?;
        let mut buffer = BufWriter::new(file);

        writeln!(buffer, "P3")?;
        writeln!(buffer, "{} {}", self.width, self.height)?;
        writeln!(buffer, "255")?;

        for y in 0..self.height {
            for x in 0..self.width {
                writeln!(
                    buffer,
                    "{} {} {}",
                    self.image_rgb[0][y][x], self.image_rgb[1][y][x], self.image_rgb[2][y][x]
                )?;
            }
        }

        buffer.flush()?;

        Ok(())
    }
}
