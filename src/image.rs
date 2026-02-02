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

/// Fully processed image
pub struct ProcessedImage {
    /// Data of the image
    pub image: image::RgbImage,

    /// Luminance values of the image
    pub luminance: Vec<f32>,

    /// Magnitude of the edge
    pub edges: Vec<f32>,

    /// Angle of the edge
    pub angles: Vec<f32>,
}

/// Calculates the dimensions the image should be resized to
fn calculate_dimensions(
    image_width: u32,
    image_height: u32,
    max_width: u32,
    max_height: u32,
    char_ratio: f32,
) -> (u32, u32) {
    let image_width = image_width as f32;
    let image_height = image_height as f32;
    let max_width = max_width as f32;
    let max_height = max_height as f32;

    let new_height = (image_height * max_width) / (char_ratio * image_width);

    if new_height <= max_height {
        (max_width as u32, new_height as u32)
    } else {
        let width = (char_ratio * image_width * max_height) / image_height;
        (width as u32, max_height as u32)
    }
}

/// Attempts to load the image from a file
pub fn load_image(file_path: &str, show_stats: bool) -> Result<DynamicImage> {
    let image = time!(show_stats, "Loading image", image::open(file_path)?);

    Ok(image)
}

/// Resizes an image to the correct number of characters
pub fn resize_image(image: &DynamicImage, settings: &Settings) -> DynamicImage {
    // If performance is an issue, using a different filter is recommended
    let filter = image::imageops::FilterType::Lanczos3;

    let (width, height) = image.dimensions();

    let (new_width, new_height) = calculate_dimensions(
        width,
        height,
        settings.max_width,
        settings.max_height,
        settings.char_ratio,
    );

    time!(
        settings.show_stats,
        "Resizing image",
        image.resize_exact(new_width, new_height, filter)
    )
}

/// Gets the luminance of an image
fn get_luminance(image: &image::RgbImage) -> Vec<f32> {
    // Using Rec. 709 luminance
    image
        .pixels()
        .map(|p| {
            let [r, g, b] = p.0;
            let y = (0.2126 * r as f32 + 0.7152 * g as f32 + 0.0722 * b as f32) / 255.0;
            y.clamp(0.0, 1.0)
        })
        .collect()
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

/// Processes an image
pub fn process_image(image: &DynamicImage, show_stats: bool) -> ProcessedImage {
    let rgb = time!(show_stats, "Converting to RGB", image.to_rgb8());

    let luminance = time!(show_stats, "Getting the luminance", get_luminance(&rgb));

    let (edges, angles) = time!(
        show_stats,
        "Sobel operator",
        sobel(&luminance, rgb.width(), rgb.height())
    );

    ProcessedImage {
        image: rgb,
        luminance,
        edges,
        angles,
    }
}

/// Convert a luminance value to its respective character
#[inline]
fn luminance_to_char(luminance: f32) -> char {
    let i = (luminance * (ASCII_CHARS.len() - 1) as f32) as usize;
    ASCII_CHARS[i] as char
}

/// Gets the appropriate edge character based off the angle
fn edge_char(angle: f32) -> char {
    let a = (angle + std::f32::consts::PI) % (2.0 * std::f32::consts::PI);

    match ((a / std::f32::consts::FRAC_PI_4).round() as i32) & 3 {
        0 => '|',
        1 => '\\',
        2 => '_',
        _ => '/',
    }
}

/// Prints an image as ASCII
pub fn print_image(image: &ProcessedImage, edge_threshold: f32) -> io::Result<()> {
    let width = image.image.width() as usize;
    let height = image.image.height() as usize;

    let mut stdout = io::BufWriter::new(io::stdout());

    let mut last_color = None;

    for y in 0..height {
        let row = y * width;

        for x in 0..width {
            let i = row + x;
            let pixel = image.image.get_pixel(x as u32, y as u32);
            let color = (pixel[0], pixel[1], pixel[2]);

            if last_color != Some(color) {
                stdout.queue(SetForegroundColor(Color::Rgb {
                    r: color.0,
                    g: color.1,
                    b: color.2,
                }))?;
                last_color = Some(color);
            }

            let c = if image.edges[i] >= edge_threshold {
                edge_char(image.angles[i])
            } else {
                luminance_to_char(image.luminance[i])
            };

            stdout.queue(Print(c))?;
        }
        stdout.queue(Print('\n'))?;
        last_color = None;
    }

    stdout.queue(style::ResetColor)?;
    stdout.flush()?;
    Ok(())
}
