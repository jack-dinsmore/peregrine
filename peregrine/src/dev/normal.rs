use std::f32::consts::PI;
use rand::Rng;

const N: usize = 64;

pub fn save_bumpmap() {
    const SIZE: (usize, usize) = (512,512);
    let mut line = Vec::with_capacity(SIZE.0);

    fn power_spectrum(freq: f32) -> f32 {
        0.02
        + (-((freq - 150.) / (2. * 30.)).powi(2)).exp() * 0.1
        + (-((freq - 65.) / (2. * 30.)).powi(2)).exp() * 0.05
    }

    for i in 0..SIZE.0 {
        line.push(i as f32 / SIZE.0 as f32 * PI);
    }
    let mut rng = rand::thread_rng();

    // Iterate over the coordinates and pixels of the image
    let mut float_buffer = image::ImageBuffer::new(SIZE.0 as u32, SIZE.1 as u32);
    for k in 0..3 {
        for _ in 0..N {
            let frequency_x = 400. * (rng.gen::<f32>() - 0.5);
            let frequency_y = 400. * (rng.gen::<f32>() - 0.5);
            let k_mag = (frequency_x*frequency_x + frequency_y*frequency_y).sqrt();
            let amplitude = power_spectrum(k_mag);
            let phase = rng.gen::<f32>() * PI;
            for (i, j, pixel) in float_buffer.enumerate_pixels_mut() {
                let x = line[i as usize];
                let y = line[j as usize];
                let image::Rgb::<f32>(mut value) = *pixel;
                value[k] += (x * frequency_x + y * frequency_y + phase).cos() * amplitude / (N as f32).sqrt();
                *pixel = image::Rgb(value);
            }
        }
    }
    let mut byte_buffer = image::ImageBuffer::new(SIZE.0 as u32, SIZE.1 as u32);
    for (i, j, pixel) in byte_buffer.enumerate_pixels_mut() {
        let image::Rgb::<f32>(value) = float_buffer[(i,j)];
        *pixel = image::Rgba([
            ((value[0] + 0.5).clamp(0., 1.) * 255.) as u8,
            ((value[1] + 0.5).clamp(0., 1.) * 255.) as u8,
            ((value[2] + 0.5).clamp(0., 1.) * 255.) as u8,
            255
        ]);
    }
    byte_buffer.save("normal.png").unwrap();
}