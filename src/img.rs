use std::collections::HashMap;

use image::{ColorType, imageops, io::Reader};


pub const IMAGE_DOWNSCALE_FACTOR: usize = 8;


#[derive(Debug, Clone)]
pub struct Rectangle<T> {
    pub top:    T,
    pub left:   T,
    pub width:  T,
    pub height: T,
}

pub trait SaveBuffer {
    fn save(&self, width: usize, height: usize, color_type: ColorType, filename: &str);
}

impl SaveBuffer for Vec<u8> {
    fn save(&self, width: usize, height: usize, color_type: ColorType, filename: &str) {
        image::save_buffer(filename, self,
            width as u32,
            height as u32,
            color_type)
            .unwrap_or_else(|e|{panic!("Unable to save image {} : {}", filename, e)})
    }
}

pub fn load(filename: &str) -> Option<(Vec<u8>, usize, usize)> {
    let reader = Reader::open(filename).ok()?;
    let image = reader.decode().ok()?;
    let buffer = image.into_bgr8();
    let width = buffer.width() as usize;
    let height = buffer.height() as usize;

    Some((buffer.into_raw(), width, height))
}

// RGB -> RGB
pub fn shrink(img: Vec<u8>, width: usize, height: usize, scale: usize) -> (usize, usize, Vec<u8>) {
    let r = image::RgbImage::from_raw(width as u32, height as u32, img)
        .expect("shrink: failed to convert img type");

    let r = imageops::thumbnail(&r, (width / scale) as u32, (height / scale) as u32);

    (r.width() as usize, r.height() as usize, r.into_raw())
}

// RGB -> L8
pub fn threshold_blue(img: &[u8], lower: u8, upper: u8) -> Vec<u8> {
    let mut out = Vec::with_capacity(img.len() / 3);

    for p in img.chunks_exact(3) {
        match p {
            [_, _, b] => {
                out.push( if *b > lower && *b < upper { u8::MAX } else { u8::MIN } )
            },
            _ => unreachable!("Invalid RGB image")
        }
    }
    out
}

// L8 -> L8
pub fn median3x3(img: &[u8], width: usize, height: usize) -> Vec<u8> {
    let mut out = vec![0; img.len()];

    for y in 1..height-1 {
        for x in 1..width-1 {
            let idx = y * width + x;

            let mut samples = [
                img[idx - width - 1], img[idx - width], img[idx - width + 1],
                img[idx         - 1], img[idx        ], img[idx         + 1],
                img[idx + width - 1], img[idx + width], img[idx + width + 1]];

            samples.sort_unstable();

            out[idx] = samples[4];
        }
    }
    out
}

// a -> a
// separate for each channel
// would be more appropriate as a triple if that is what is being passed in
pub fn mode(images: &[Vec<u8>]) -> Option<Vec<u8>> {
    let mut out = vec![0; images.first()?.len()];

    for p in 0..out.len() {
        let mut frequencies = HashMap::new();

        for image in images {
            frequencies.entry(image[p]).and_modify(|x|*x+=1).or_insert(1);
        }

        let mut mode = 0;
        let mut max_frequency = 0;
        for (value, frequency) in frequencies.drain() {
            if frequency > max_frequency {
                max_frequency = frequency;
                mode = value;
            }
        }
        out[p] = mode;
    }
    Some(out)
}

pub fn bgr_to_rgb(image: &mut [u8]) {
    for p in image.chunks_exact_mut(3) {
        match p {
            [r, _, b] => { std::mem::swap(r, b) },
            _ => unreachable!("Incorrect size bgr image")
        }
    }
}

// RGB -> RGB -> RGB
pub fn remove_background(img: &[u8], back: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(img.len());

    for (&a, &b) in img.iter().zip(back) {
        out.push(if a == b { 0 } else { a });
    }
    out
}
