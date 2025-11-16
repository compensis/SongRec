use std::convert::TryFrom;
use std::fs::File;
use std::io::BufReader;
use std::io::Cursor;
use std::time::Duration;
use std::thread;
use std::sync::{Arc, Mutex};
use image::{codecs::gif::GifDecoder, AnimationDecoder, ImageReader};
use rpi_led_matrix::{LedFont, LedColor, LedCanvas, LedMatrix, LedMatrixOptions, LedRuntimeOptions, TextDrawOptions, TextLayout};
use crate::cli::orientation_sensor::{OrientationSensor, SensorOrientation};

pub struct MatrixDisplay {
    message: Arc<Mutex<Option<MatrixDisplayMessage>>>,
}

enum MatrixDisplayMessage {
    ShwoAnimation,
    #[allow(dead_code)]
    ShowSong { 
        song_name: String,
        artist: String,
        release_year: Option<String>,
        cover_image: Option<Vec<u8>>,
    },
    Off,
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

impl MatrixDisplay {

    pub fn new() -> Self {

        let shared_message: Arc<Mutex<Option<MatrixDisplayMessage>>> = Arc::new(Mutex::new(None));

        let message_stread: Arc<Mutex<Option<MatrixDisplayMessage>>> = Arc::clone(&shared_message);
        thread::spawn(move || {

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
            let _ = options.set_brightness(75);

            runtime_options.set_gpio_slowdown(3);

            let matrix = LedMatrix::new(Some(options), Some(runtime_options)).unwrap();

            let mut canvas = matrix.offscreen_canvas();

            let font_file = std::path::Path::new("./external/rust-rpi-rgb-led-matrix/rpi-led-matrix-sys/cpp-library/fonts/Grand9K-Pixel.bdf");
            let _heightfont = LedFont::new(font_file).unwrap();
            
            let (width, _height) = canvas.canvas_size();

            let _options = TextDrawOptions::new().layout(TextLayout::Wrapped{line_width: width});

            let mut animation = Animation::new("./matrix-display/animation16x16.gif");
            
            loop {
                let mut locked_message = message_stread.lock().unwrap();

                if let Some(ref message) = locked_message.take() {
                    canvas.clear();
                    match message {
                        MatrixDisplayMessage::ShwoAnimation => {
                            // Keep showing anaimation also during the next loop iteration
                            *locked_message = Some(MatrixDisplayMessage::ShwoAnimation);
                            drop(locked_message);
                            canvas = animation.show_next_frame(canvas);
                        }
                        MatrixDisplayMessage::ShowSong{ song_name: _, artist: _, release_year: _, cover_image} => {
                            drop(locked_message);
                            /*
                            let height = canvas.draw_text(&font, song_name, &options);
                            let artist_options = TextDrawOptions::new().layout(TextLayout::Wrapped{line_width: width}).position(0, height);
                            canvas.draw_text(&font, artist, &artist_options);
                            */
                            if let Some(image_data) = cover_image {
                                canvas = show_image(image_data, canvas);
                            }
                        }
                        MatrixDisplayMessage::Off => {
                            drop(locked_message);
                            // Display nothing
                        }
                    }
                    canvas = matrix.swap(canvas);
                }
            };
        });

        Self { message: shared_message }
    }

    fn set_message(self: &mut Self, message: MatrixDisplayMessage) {
        let mut locked_message = self.message.lock().unwrap();
        *locked_message = Some(message);
    }

    pub fn init(self: &mut Self) {
        self.set_message(MatrixDisplayMessage::ShwoAnimation);
    }

    pub fn show_song(self: &mut Self, song_name: &str, artist: &str, release_year: &Option<String>, cover_image: &Option<Vec<u8>>) {
        self.set_message(MatrixDisplayMessage::ShowSong{
            song_name: song_name.to_string(),
            artist: artist.to_string(),
            release_year: release_year.clone(),
            cover_image: cover_image.clone(),
        });
    }

    pub fn off(self: &mut Self) {
        self.set_message(MatrixDisplayMessage::Off);
    }
}

struct Animation {
    frames: Vec<image::Frame>,
    index: usize,
}

impl Animation {
    fn new<P: AsRef<std::path::Path>>(path: P) -> Self {
        let file = File::open(path).unwrap();
        let reader = BufReader::new(file);
        let decoder = GifDecoder::new(reader).unwrap();

        let frames = decoder.into_frames();
        let frames = frames.collect_frames().unwrap();
        Self {
            frames,
            index: 0,
        }
    }

    fn show_next_frame(
        &mut self,
        mut canvas: LedCanvas,
    ) -> LedCanvas {
        let frame = &self.frames[self.index];
        self.index = (self.index + 1) % self.frames.len();

        let image_buffer = frame.buffer();
        let (numerator, denominator) = frame.delay().numer_denom_ms();
        let delay_ms = (numerator / denominator) as u64;

        let (width, height) = canvas.canvas_size();

        let image_width = i32::try_from(image_buffer.width()).unwrap();
        let image_height = i32::try_from(image_buffer.height()).unwrap();
        let x_offset = (width - image_width) / 2;
        let y_offset = (height - image_height) / 2;

        for (x, y, pixel) in image_buffer.enumerate_pixels() {
            let image::Rgba([red, green, blue, _alpha]) = *pixel;
            let color = LedColor { red, green, blue };
            canvas.set(x as i32 + x_offset, y as i32 + y_offset, &color);
        }

        thread::sleep(Duration::from_millis(delay_ms));

        canvas
    }
}

fn show_image(
    image_data: &Vec<u8>,
    mut canvas: LedCanvas,
) -> LedCanvas {
    let cursor = Cursor::new(&image_data);
    let dynamic_image = ImageReader::new(cursor)
        .with_guessed_format().unwrap()
        .decode().unwrap();
    let (width, height) = canvas.canvas_size();
    let resized_image = dynamic_image.resize_exact(
        width as u32, 
        height as u32, 
        image::imageops::FilterType::Lanczos3,
    );
    let rgb_image = resized_image.to_rgb8();
    for (x, y, pixel) in rgb_image.enumerate_pixels() {
        let image::Rgb([red, green, blue]) = *pixel;
        let color = LedColor { red, green, blue };
        canvas.set(x as i32, y as i32, &color);
    }
    canvas
}