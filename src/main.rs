mod benchmark;
mod image;
mod parser;

use crate::image::{load_image, print_image, process_image, resize_image};
use parser::parse_args;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings = parse_args();

    let image = load_image(&settings.file_path, settings.show_stats)?;

    let resized = resize_image(&image, &settings);

    let processed = process_image(&resized, settings.edge_threshold, settings.show_stats);

    print_image(&processed)?;

    Ok(())
}
