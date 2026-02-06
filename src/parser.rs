use clap::Parser;
use clap::ValueEnum;
use crossterm::terminal::size;
use image::imageops::FilterType;

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ResizeFilter {
    Nearest,
    Triangle,
    CatmullRom,
    Gaussian,
    Lanczos3,
}

/// The arguments a user can provide to the program
#[derive(Debug, Parser)]
#[command(
    version = "0.1.0",
    about = "Convert images to ASCII art",
    long_about = "This program takes an image and converts it into ASCII art"
)]
pub struct Args {
    /// Path to the image file
    pub file_path: String,

    /// Maximum width of the ASCII output
    #[arg(short = 'W', long = "max-width")]
    pub max_width: Option<u32>,

    /// Maximum height of the ASCII output
    #[arg(short = 'H', long = "max-height")]
    pub max_height: Option<u32>,

    /// Edge detection threshold (0.0 - 1.0)
    #[arg(short = 'e', long = "edge-threshold", default_value_t = 0.4)]
    pub edge_threshold: f32,

    /// Character aspect ratio
    #[arg(short = 'c', long = "char-ratio", default_value_t = 2.0)]
    pub char_ratio: f32,

    /// The filter to be used when resizing the image
    #[arg(long = "filter", value_enum, default_value_t = ResizeFilter::Lanczos3)]
    pub filter: ResizeFilter,

    /// Try to scale the image so as to fit the terminal's dimensions
    #[arg(long = "fit-terminal")]
    pub fit_terminal: bool,

    /// Show processing stats
    #[arg(long = "stats")]
    pub show_stats: bool,
}

/// All the processed settings that came from the program arguments
#[derive(Debug)]
pub struct Settings {
    /// Path to the image file
    pub file_path: String,

    /// Maximum width of the ASCII output
    pub max_width: u32,

    /// Maximum height of the ASCII output
    pub max_height: u32,

    /// Edge detection threshold (0.0 - 1.0)
    pub edge_threshold: f32,

    /// Character aspect ratio
    pub char_ratio: f32,

    /// The filter to be used when resizing the image
    pub filter: FilterType,

    /// Show processing stats
    pub show_stats: bool,
}

impl From<ResizeFilter> for image::imageops::FilterType {
    fn from(f: ResizeFilter) -> Self {
        use image::imageops::FilterType::*;
        match f {
            ResizeFilter::Nearest => Nearest,
            ResizeFilter::Triangle => Triangle,
            ResizeFilter::CatmullRom => CatmullRom,
            ResizeFilter::Gaussian => Gaussian,
            ResizeFilter::Lanczos3 => Lanczos3,
        }
    }
}

impl From<Args> for Settings {
    fn from(args: Args) -> Self {
        const DEFAULT_WIDTH: u32 = 80;
        const DEFAULT_HEIGHT: u32 = 60;

        let mut max_width = args.max_width;
        let mut max_height = args.max_height;

        // Giving precedence to width and height values provided manually
        if args.fit_terminal {
            if let Ok((w, h)) = size() {
                if max_width.is_none() {
                    max_width = Some(w as u32);
                }
                if max_height.is_none() {
                    max_height = Some(h as u32);
                }
            }
        }

        let max_width = max_width.unwrap_or(DEFAULT_WIDTH);
        let max_height = max_height.unwrap_or(DEFAULT_HEIGHT);

        Settings {
            file_path: args.file_path,
            max_width,
            max_height,
            edge_threshold: args.edge_threshold.clamp(0.0, 1.0),
            char_ratio: args.char_ratio,
            show_stats: args.show_stats,
            filter: args.filter.into(),
        }
    }
}

/// Parse the CLI arguments and resolve them into Settings
pub fn parse_args() -> Settings {
    Args::parse().into()
}
