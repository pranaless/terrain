use bracket_noise::prelude::FastNoise;
use image::{ImageOutputFormat, Rgb, RgbImage};
use rand::{rngs::StdRng, thread_rng, RngCore, SeedableRng};
use std::io::Cursor;
use wasm_bindgen::prelude::*;
use web_sys::HtmlTableElement;

fn generate_layer(
    noise: &mut FastNoise,
    size: (usize, usize),
    scale: f32,
    offset: f32,
    output: &mut Vec<f32>,
) {
    for y in 0..size.1 {
        for x in 0..size.0 {
            output[x + y * size.0] += noise.get_noise(x as f32, y as f32) * scale + offset;
        }
    }
}

#[wasm_bindgen]
pub struct Map {
    size: (usize, usize),
    height: (f32, f32),
    roughness: usize,
    data: Vec<f32>,
}
#[wasm_bindgen]
impl Map {
    #[wasm_bindgen(constructor)]
    pub fn new(size_x: usize, size_y: usize, min_z: f32, max_z: f32, roughness: usize) -> Self {
        Map {
            size: (size_x, size_y),
            height: (min_z, max_z),
            roughness,
            data: Vec::new(),
        }
    }

    pub fn generate_seeded(&mut self, seed: Option<u64>) {
        if let Some(seed) = seed {
            self.generate(&mut StdRng::seed_from_u64(seed));
        } else {
            self.generate(&mut thread_rng());
        }
    }

    pub fn to_html_table(&self, table: HtmlTableElement) {
        let document = web_sys::window().unwrap().document().unwrap();
        table.set_inner_html("");
        let mut row = table.insert_row().unwrap();
        self.iter().for_each(|((x, y), h)| {
            if x == 0 && y != 0 {
                row = table.insert_row().unwrap();
            }
            let h = h.clamp(0.0, 1.0) * (self.height.1 - self.height.0) + self.height.0;
            let cell = document.create_element("td").unwrap();
            cell.set_inner_html(&format!("{:.1}", h));
            row.append_child(&cell).unwrap();
        });
    }

    pub fn to_table(&self) -> String {
        let width = self.height.1.log10() as usize + 3;
        let mut data = String::new();
        self.iter().for_each(|((x, y), h)| {
            if x == 0 && y != 0 {
                data.pop();
                // Just in case, using \r\n
                data.push_str("\r\n");
            }
            let h = h.clamp(0.0, 1.0) * (self.height.1 - self.height.0) + self.height.0;
            data.push_str(&format!("{:width$.1} ", h, width = width));
        });
        data
    }

    pub fn to_data_uri(&self) -> String {
        let mut data = Vec::new();
        let image = self.to_image();
        image
            .write_to(&mut Cursor::new(&mut data), ImageOutputFormat::Png)
            .expect("could not encode image");
        format!("data:image/png;base64,{}", base64::encode(data))
    }
}
impl Map {
    pub fn generate(&mut self, rng: &mut impl RngCore) {
        let mut data = Vec::new();
        data.resize(self.size.0 * self.size.1, 0.0);

        let avg = (self.size.0 as f32 + self.size.1 as f32) / 2.0;
        let mut freq = 1.0 / avg;
        let mut scale = 1.0;

        let mut noise = FastNoise::seeded(rng.next_u64());
        noise.set_frequency(freq);
        generate_layer(&mut noise, self.size, 0.5 * scale, 0.5 * scale, &mut data);
        for _ in 0..self.roughness {
            freq *= 4.0;
            scale /= 6.0;
            noise.set_seed(rng.next_u64());
            noise.set_frequency(freq);
            generate_layer(&mut noise, self.size, scale, 0.0, &mut data);
        }
        self.data = data;
    }

    pub fn iter(&self) -> impl Iterator<Item = ((usize, usize), f32)> + '_ {
        let width = self.size.0;
        self.data
            .iter()
            .copied()
            .enumerate()
            .map(move |(i, v)| ((i % width, i / width), v))
    }

    pub fn to_image(&self) -> RgbImage {
        let mut image = RgbImage::new(self.size.0 as _, self.size.1 as _);
        self.iter().for_each(|((x, y), h)| {
            let c = (h.clamp(0.0, 1.0) * 255.0) as u8;
            image.put_pixel(x as _, y as _, Rgb([c, c, c]));
        });
        image
    }
}

fn main() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init().expect("could not initialize logger");
}
