#![allow(dead_code)]

use std::f32::consts::PI;
use ndarray::Array1;
use ndarray_interp::interp1d::Interp1DBuilder;
use rand::Rng;

const N: usize = 20;
const MAX_FREQ: f32 = 400.;

fn triangle_wave(freq: f32, mean: f32, sigma: f32) -> f32 {
    (sigma - (freq - mean).abs()).max(0.) / sigma
}
fn gaussian(freq: f32, mean: f32, sigma: f32) -> f32 {
    (-((freq-mean)/sigma).powi(2) / 2.).exp()
}

pub fn fourier_save_bumpmap() {
    const SIZE: (usize, usize) = (512,512);
    let mut line = Vec::with_capacity(SIZE.0);

    fn power_spectrum(freq: f32) -> f32 {
        0.001
        + (0.0005 * freq).powf(1.4)
        + triangle_wave(freq, 300., 10.) * 0.05
        + triangle_wave(freq, 14., 3.) * 0.2
        + triangle_wave(freq, 48., 10.) * 0.03
    }

    for i in 0..SIZE.0 {
        line.push(i as f32 / SIZE.0 as f32 * PI);
    }
    let mut rng = rand::thread_rng();

    // 
    let mut ks = linspace(0., MAX_FREQ, 200);
    let mut cumsum = 0.;
    let mut cumsums = vec![0.; ks.len()];
    for (i, k) in ks.iter().enumerate() {
        cumsums[i] = cumsum;
        cumsum += power_spectrum(*k);
    }
    ks.push(ks[ks.len()-1] + ks[1]);
    cumsums.push(cumsum);
    for cs in &mut cumsums {
        *cs /= cumsum;
    }
    let cumsum_interpolator = Interp1DBuilder::new(Array1::from_vec(ks))
        .x(Array1::from_vec(cumsums))
        .build().unwrap();

    // Iterate over the coordinates and pixels of the image
    let mut float_buffer = image::ImageBuffer::new(SIZE.0 as u32, SIZE.1 as u32);
    for k in 0..3 {
        for frac in linspace(0., 1., N) {
            let freq_mag = cumsum_interpolator.interp_scalar(frac).unwrap();
            let amplitude = power_spectrum(freq_mag);
            let theta = rng.gen::<f32>() * 2. * PI;
            let freq_x = freq_mag * theta.cos();
            let freq_y = freq_mag * theta.sin();
            let phase = rng.gen::<f32>() * 2. * PI;
            for (i, j, pixel) in float_buffer.enumerate_pixels_mut() {
                let x = line[i as usize];
                let y = line[j as usize];
                let image::Rgb::<f32>(mut value) = *pixel;
                value[k] += (x * freq_x + y * freq_y + phase).cos() * amplitude / (N as f32).sqrt();
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

fn linspace(start: f32, stop: f32, size: usize) -> Vec<f32> {
    let mut out = Vec::with_capacity(size);
    for i in 0..size {
        let v = start + (stop - start) * i as f32 / (size - 1) as f32;
        out.push(v)
    }
    out
}

// pub fn perlin_save_bumpmap() {
//     const SIZE: (usize, usize) = (512,512);
//     let mut line = Vec::with_capacity(SIZE.0);

//     for i in 0..SIZE.0 {
//         line.push(i as f32 / SIZE.0 as f32 * PI);
//     }
//     let mut rng = rand::thread_rng();

//     // Iterate over the coordinates and pixels of the image
//     let mut float_buffer = image::ImageBuffer::new(SIZE.0 as u32, SIZE.1 as u32);
//     for k in 0..3 {
//         for _ in 0..N {
//             let frequency_x = 400. * (rng.gen::<f32>() - 0.5);
//             let frequency_y = 400. * (rng.gen::<f32>() - 0.5);
//             let k_mag = (frequency_x*frequency_x + frequency_y*frequency_y).sqrt();
//             let amplitude = power_spectrum(k_mag);
//             let phase = rng.gen::<f32>() * 2. * PI;
//             for (i, j, pixel) in float_buffer.enumerate_pixels_mut() {
//                 let x = line[i as usize];
//                 let y = line[j as usize];
//                 let image::Rgb::<f32>(mut value) = *pixel;
//                 value[k] += (x * frequency_x + y * frequency_y + phase).cos() * amplitude / (N as f32).sqrt();
//                 *pixel = image::Rgb(value);
//             }
//         }
//     }
//     let mut byte_buffer = image::ImageBuffer::new(SIZE.0 as u32, SIZE.1 as u32);
//     for (i, j, pixel) in byte_buffer.enumerate_pixels_mut() {
//         let image::Rgb::<f32>(value) = float_buffer[(i,j)];
//         *pixel = image::Rgba([
//             ((value[0] + 0.5).clamp(0., 1.) * 255.) as u8,
//             ((value[1] + 0.5).clamp(0., 1.) * 255.) as u8,
//             ((value[2] + 0.5).clamp(0., 1.) * 255.) as u8,
//             255
//         ]);
//     }
//     byte_buffer.save("normal.png").unwrap();
// }