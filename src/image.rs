use crate::parser::Settings;
use crate::time;
use anyhow::Result;
use crossterm::{
    QueueableCommand,
    style::{self, Color, Print, SetForegroundColor},
};
use image::{DynamicImage, GenericImageView};
use std::io::{self, Write};

const ASCII_CHARS: &[u8] = b" .-=+*x#$&X@";

/// Representation of a pixel in ASCII
pub struct AsciiPixel {
    /// Character
    pub ch: u8,

    /// Red value
    pub r: u8,

    /// Green value
    pub g: u8,

    /// Blue value
    pub b: u8,
}

/// Fully processed image
pub struct ProcessedImage {
    /// Width of the image
    pub width: u32,

    /// Height of the image
    pub height: u32,

    /// Pixel data
    pub pixels: Vec<AsciiPixel>,
}

/// Attempts to load the image from a file
pub fn load_image(file_path: &str, show_stats: bool) -> Result<DynamicImage> {
    let image = time!(image::open(file_path)?, "Loading the image", show_stats);

    Ok(image)
}

/// Calculates the dimensions the image should be resized to
fn calculate_dimensions(
    image_width: u32,
    image_height: u32,
    max_width: u32,
    max_height: u32,
    char_ratio: f32,
) -> (u32, u32) {
    let iw = image_width as f32;
    let ih = image_height as f32;
    let mw = max_width as f32;
    let mh = max_height as f32;

    let new_height = ih * mw / (char_ratio * iw);

    if new_height <= mh {
        (max_width, new_height.round() as u32)
    } else {
        let width = char_ratio * iw * mh / ih;
        (width.round() as u32, max_height)
    }
}

/// Resizes an image to the correct number of characters
pub fn resize_image(image: &DynamicImage, settings: &Settings) -> DynamicImage {
    let (width, height) = image.dimensions();

    let (new_width, new_height) = calculate_dimensions(
        width,
        height,
        settings.max_width,
        settings.max_height,
        settings.char_ratio,
    );
    
    time!(
        image.resize_exact(new_width, new_height, settings.filter),
        "Resizing the image",
        settings.show_stats
    )
}

/// Gets the luminance of an image
pub fn get_luminance(image: &image::RgbImage) -> Vec<f32> {
    let mut luminance = Vec::with_capacity((image.width() * image.height()) as usize);

    // Using Rec. 709 luminance
    for pixel in image.pixels() {
        let [r, g, b] = pixel.0;
        let luma = (0.2126 * r as f32 + 0.7152 * g as f32 + 0.0722 * b as f32) / 255.0;
        luminance.push(luma.clamp(0.0, 1.0));
    }

    luminance
}

/// Applies the Sobel operator over an image
///
/// Gets the magnitude and angle of the edges of an image
fn sobel(luminance: &[f32], width: u32, height: u32) -> (Vec<f32>, Vec<f32>) {
    let width = width as usize;
    let height = height as usize;

    let mut magnitude = vec![0.0; width * height];
    let mut angle = vec![0.0; width * height];

    let max_sobel = (32.0_f32).sqrt();

    // The borders of the image are intentionally kept at 0
    for y in 1..height - 1 {
        // Pre-calculate row offsets to avoid repeated multiplication
        let prev_row = (y - 1) * width;
        let curr_row = y * width;
        let next_row = (y + 1) * width;

        for x in 1..width - 1 {
            let x_left = x - 1;
            let x_right = x + 1;

            // Apply the Sobel operator
            let gx = (luminance[prev_row + x_right] - luminance[prev_row + x_left])
                + 2.0 * (luminance[curr_row + x_right] - luminance[curr_row + x_left])
                + (luminance[next_row + x_right] - luminance[next_row + x_left]);

            let gy = (luminance[prev_row + x_left]
                + 2.0 * luminance[prev_row + x]
                + luminance[prev_row + x_right])
                - (luminance[next_row + x_left]
                    + 2.0 * luminance[next_row + x]
                    + luminance[next_row + x_right]);

            let i = curr_row + x;

            let raw_mag = gx.hypot(gy);
            magnitude[i] = (raw_mag / max_sobel).clamp(0.0, 1.0);
            angle[i] = gy.atan2(gx);
        }
    }

    (magnitude, angle)
}

/// Convert the given data into the adequate ASCII characters and colours
fn to_ascii(
    image: &image::RgbImage,
    luminance: Vec<f32>,
    edges: Vec<f32>,
    angles: Vec<f32>,
    edge_threshold: f32,
) -> Vec<AsciiPixel> {
    let (width, height) = (image.width() as usize, image.height() as usize);
    let length = width * height;
    let mut pixels = Vec::with_capacity(length);

    for i in 0..(length) {
        let x = i % width;
        let y = i / width;

        let p = image.get_pixel(x as u32, y as u32);
        let [r, g, b] = p.0;

        let c = if edges[i] >= edge_threshold {
            edge_char(angles[i])
        } else {
            luminance_to_char(luminance[i])
        };

        pixels.push(AsciiPixel { ch: c, r, g, b });
    }

    pixels
}

/// Processes an image
pub fn process_image(
    image: &DynamicImage,
    edge_threshold: f32,
    show_stats: bool,
) -> ProcessedImage {
    let rgb = time!(image.to_rgb8(), "Converting to RGB", show_stats);

    let (width, height) = (rgb.width(), rgb.height());

    let luminance = time!(get_luminance(&rgb), "Getting the luminance", show_stats);

    let (edges, angles) = time!(
        sobel(&luminance, rgb.width(), rgb.height()),
        "Sobel operator",
        show_stats
    );

    let pixels = time!(
        to_ascii(&rgb, luminance, edges, angles, edge_threshold),
        "Converting to ASCII",
        show_stats
    );

    ProcessedImage {
        width,
        height,
        pixels,
    }
}

/// Convert a luminance value to its respective character
#[inline]
fn luminance_to_char(luminance: f32) -> u8 {
    let i = (luminance * (ASCII_CHARS.len() - 1) as f32) as usize;
    ASCII_CHARS[i]
}

/// Gets the appropriate edge character based off the angle
fn edge_char(angle: f32) -> u8 {
    let a = (angle + std::f32::consts::PI) % (2.0 * std::f32::consts::PI);

    match ((a / std::f32::consts::FRAC_PI_4).round() as i32) & 3 {
        0 => b'|',
        1 => b'\\',
        2 => b'_',
        _ => b'/',
    }
}

/// Prints an image as ASCII characters
pub fn print_image(image: &ProcessedImage) -> io::Result<()> {
    let mut stdout = io::BufWriter::new(io::stdout());

    let width = image.width as usize;
    let height = image.height as usize;

    for y in 0..height {
        let row = y * width;

        for x in 0..width {
            let i = row + x;
            let pixel = &image.pixels[i];

            stdout.queue(SetForegroundColor(Color::Rgb {
                r: pixel.r,
                g: pixel.g,
                b: pixel.b,
            }))?;

            stdout.queue(Print(pixel.ch as char))?;
        }
        stdout.queue(Print('\n'))?;
    }

    stdout.queue(style::ResetColor)?;
    stdout.flush()?;
    Ok(())
}
