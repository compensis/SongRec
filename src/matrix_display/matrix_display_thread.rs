use crate::matrix_display::orientation_sensor::{OrientationSensor, SensorOrientation};
use image::{codecs::gif::GifDecoder, AnimationDecoder, ImageReader};
use rpi_led_matrix::{
    LedCanvas, LedColor, LedFont, LedMatrix, LedMatrixOptions, LedRuntimeOptions, TextDrawOptions,
    TextLayout,
};
use std::borrow::Cow;
use std::convert::TryFrom;
use std::io::Cursor;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

const FONT_DATA: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/external/rust-rpi-rgb-led-matrix/rpi-led-matrix-sys/cpp-library/fonts/Grand9K-Pixel.bdf"
));
const ANIMATION_IMAGE_DATA: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/animation16x16.gif"
));

pub enum MatrixDisplayMessage {
    ShowAnimation,
    ShowSong {
        song_name: String,
        artist: String,
        release_year: Option<String>,
        cover_image: Option<Vec<u8>>,
    },
    Off,
}

pub struct SharedMatrixDisplayMessage {
    message: Arc<Mutex<Option<MatrixDisplayMessage>>>,
}

impl SharedMatrixDisplayMessage {
    pub fn new() -> Self {
        Self {
            message: Arc::new(Mutex::new(None)),
        }
    }

    pub fn share(&self) -> Self {
        Self {
            message: Arc::clone(&self.message),
        }
    }

    pub fn set(&self, message: MatrixDisplayMessage) {
        let mut locked_message = self.message.lock().unwrap();
        *locked_message = Some(message);
    }

    fn take(&self) -> Option<MatrixDisplayMessage> {
        let mut locked_message = self.message.lock().unwrap();
        locked_message.take()
    }
}

#[derive(Debug)]
enum DisplayOrientation {
    Upright,
    Right,
    UpsideDown,
    Left,
}

impl From<SensorOrientation> for DisplayOrientation {
    fn from(orientation: SensorOrientation) -> Self {
        match orientation {
            SensorOrientation::YTop => DisplayOrientation::Upright,
            SensorOrientation::ZTop => DisplayOrientation::Right,
            SensorOrientation::YDown => DisplayOrientation::UpsideDown,
            SensorOrientation::ZDown => DisplayOrientation::Left,
            _ => DisplayOrientation::Upright,
        }
    }
}

pub fn matrix_display_thread(shared_message: SharedMatrixDisplayMessage) {
    let mut orientation_sensor = OrientationSensor::new();

    let orientation = orientation_sensor.get_orientation();
    println!("Senor Orientation: {:?}", orientation);

    let orientation: DisplayOrientation = orientation_sensor.get_orientation().into();
    println!("Display Orientation: {:?}", orientation);

    let mut options = LedMatrixOptions::new();
    let mut runtime_options = LedRuntimeOptions::new();

    options.set_hardware_mapping("adafruit-hat-pwm");
    options.set_rows(64);
    options.set_cols(64);
    options.set_pwm_lsb_nanoseconds(130);
    options.set_pixel_mapper_config("Rotate:180");
    let _ = options.set_brightness(75);

    runtime_options.set_gpio_slowdown(3);

    let matrix = LedMatrix::new(Some(options), Some(runtime_options)).unwrap();

    let mut canvas = matrix.offscreen_canvas();
    let font = LedFont::new_from_buffer(FONT_DATA).unwrap();

    let mut animation = Animation::new(ANIMATION_IMAGE_DATA);

    loop {
        if let Some(ref message) = shared_message.take() {
            canvas.clear();
            match message {
                MatrixDisplayMessage::ShowAnimation => {
                    // Keep showing anaimation also during the next loop iteration
                    shared_message.set(MatrixDisplayMessage::ShowAnimation);
                    animation.show_next_frame(&mut canvas);
                }
                MatrixDisplayMessage::ShowSong {
                    song_name,
                    artist,
                    release_year,
                    cover_image,
                } => {
                    let orientation = orientation_sensor.get_orientation().into();
                    match (orientation, cover_image) {
                        (DisplayOrientation::Upright, _) | (_, None) => {
                            show_info(song_name, artist, release_year, &font, &mut canvas)
                        }
                        (orientation, Some(image_data)) => {
                            show_image(&image_data, &orientation, &mut canvas);
                        }
                    }
                }
                MatrixDisplayMessage::Off => {
                    // Display nothing
                }
            }
            canvas = matrix.swap(canvas);
        }
    }
}

struct Animation {
    frames: Vec<image::Frame>,
    index: usize,
}

impl Animation {
    fn new(data: &[u8]) -> Self {
        let cursor = Cursor::new(data);
        let decoder = GifDecoder::new(cursor).unwrap();

        let frames = decoder.into_frames();
        let frames = frames.collect_frames().unwrap();
        Self { frames, index: 0 }
    }

    fn show_next_frame(&mut self, canvas: &mut LedCanvas) {
        if self.frames.is_empty() {
            return; // nothing to do
        }

        let frame = &self.frames[self.index];
        self.index = (self.index + 1) % self.frames.len();

        let image_buffer = frame.buffer();
        let (numerator, denominator) = frame.delay().numer_denom_ms();
        let delay_ms = (numerator / denominator) as u64;

        let (width, height) = canvas.canvas_size();

        let image_width = i32::try_from(image_buffer.width()).unwrap_or(0);
        let image_height = i32::try_from(image_buffer.height()).unwrap_or(0);
        let x_offset = (width - image_width) / 2;
        let y_offset = (height - image_height) / 2;

        for (x, y, pixel) in image_buffer.enumerate_pixels() {
            let image::Rgba([red, green, blue, _alpha]) = *pixel;
            let color = LedColor { red, green, blue };
            canvas.set(x as i32 + x_offset, y as i32 + y_offset, &color);
        }

        thread::sleep(Duration::from_millis(delay_ms));
    }
}

/// Replaces the last word in the given string with an ellipsis ('…').
///
/// Behavior:
/// - If the string contains at least one space, everything after the last space is removed,
///   and an ellipsis is appended.
/// - If the string has no spaces and at least two characters, the last character is replaced
///   with an ellipsis.
/// - Strings with fewer than 2 characters remain unchanged.
///
/// # Parameter
/// - `s: &mut Cow<'_, str>`:
///   The function takes a mutable reference to a [`Cow`] (Clone-on-Write) string to allow
///   efficient modification:
///   - If the input string is borrowed (`&str`), it will be cloned only when a mutation is required,
///     avoiding unnecessary allocations if no change is needed.
///   - If the input string is already owned (`String`), it can be modified in place without cloning.
fn replace_last_word_with_ellipsis(s: &mut Cow<'_, str>) {
    // Clone-on-write: clones only if s is Borrowed
    let s = s.to_mut();

    if let Some(pos) = s.rfind(' ') {
        // Found a space: truncate everything after the last space
        s.truncate(pos);
    } else if s.chars().count() >= 2 {
        // No space found, but string has at least 2 characters
        // Use char_indices to handle Unicode safely
        let mut chars = s.char_indices().rev();
        if let Some((second_last_pos, _)) = chars.nth(1) {
            // Truncate up to the second-to-last character
            s.truncate(second_last_pos);
        }
    } else {
        // String too short (<2 characters), do nothing
        return;
    }

    // Append the ellipsis to indicate truncation
    s.push('…');
}

fn draw_year(year: &str, font: &LedFont, canvas: &mut LedCanvas) {
    // Only the last two digits of the year
    let year = &year[year.len() - 2..];

    let (canvas_width, _canvas_height) = canvas.canvas_size();

    // First draw outsice canvas to get width
    let options = TextDrawOptions::new().position(canvas_width, 0);
    let width = canvas.draw_text(font, year, &options);

    // Place the year in the upper right corner.
    let x = canvas_width - width + 1;
    let y = font.height().unwrap_or(0) - 1;
    let options = options.position(x, y);
    canvas.draw_text(font, year, &options);
}

fn show_info<'a>(
    name: &'a str,
    artist: &'a str,
    release_year: &Option<String>,
    font: &LedFont,
    canvas: &mut LedCanvas,
) {
    let (canvas_width, canvas_height) = canvas.canvas_size();

    // Use Cow instead of immediately cloning; clone-on-write only if truncation is needed
    let mut name: Cow<'a, str> = Cow::Borrowed(name);
    let mut artist: Cow<'a, str> = Cow::Borrowed(artist);

    let base_options = TextDrawOptions::new().layout(TextLayout::Wrapped {
        line_width: canvas_width,
    });

    // Determine initial heights
    let mut name_height = canvas.draw_text(font, &name, &base_options);
    let mut artist_height = canvas.draw_text(font, &artist, &base_options);

    // Shorten strings until they fit within the canvas
    while name_height + artist_height > canvas_height {
        if name_height > artist_height {
            replace_last_word_with_ellipsis(&mut name);
            name_height = canvas.draw_text(font, &name, &base_options);
        } else {
            replace_last_word_with_ellipsis(&mut artist);
            artist_height = canvas.draw_text(font, &artist, &base_options);
        }
    }

    canvas.clear();

    // Draw artist at the bottom
    let artist_y = canvas_height - artist_height;
    let artist_options = base_options.position(0, artist_y);
    canvas.draw_text(font, &artist, &artist_options);

    // Draw name centered above artist
    let name_y = (canvas_height - name_height - artist_height) / 2;
    let name_options = artist_options.position(0, name_y);
    canvas.draw_text(font, &name, &name_options);

    // Optional: draw year at the top
    if let Some(year) = release_year {
        if name_y >= font.height().unwrap_or(0) {
            draw_year(year, font, canvas);
        }
    }
}

fn show_image(image_data: &[u8], orientation: &DisplayOrientation, canvas: &mut LedCanvas) {
    let (width, height) = canvas.canvas_size();
    let cursor = Cursor::new(image_data);
    let dynamic_image = ImageReader::new(cursor)
        .with_guessed_format()
        .unwrap()
        .decode()
        .unwrap()
        .resize_exact(
            width as u32,
            height as u32,
            image::imageops::FilterType::Lanczos3,
        );
    let rotated_image = match orientation {
        DisplayOrientation::Left => dynamic_image.rotate90(),
        DisplayOrientation::Right => dynamic_image.rotate270(),
        DisplayOrientation::UpsideDown => dynamic_image.rotate180(),
        _ => dynamic_image,
    };
    let rgb_image = rotated_image.to_rgb8();
    for (x, y, pixel) in rgb_image.enumerate_pixels() {
        let image::Rgb([red, green, blue]) = *pixel;
        let color = LedColor { red, green, blue };
        canvas.set(x as i32, y as i32, &color);
    }
}
