use std::thread;
use std::sync::{Arc, Mutex};
use rpi_led_matrix::{LedFont, LedMatrix, LedMatrixOptions, LedRuntimeOptions, TextDrawOptions, TextLayout};

pub struct MatrixDisplay {
    message: Arc<Mutex<Option<MatrixDisplayMessage>>>,
}

enum MatrixDisplayMessage {
    ShwoAnimation,
    ShowSong { 
        song_name: String,
        artist: String,
        _release_year: Option<String>,
        _cover_image: Option<Vec<u8>>,
    },
    Off,
}

impl MatrixDisplay {

    pub fn new() -> Self {

        let shared_message: Arc<Mutex<Option<MatrixDisplayMessage>>> = Arc::new(Mutex::new(None));

        let message_stread: Arc<Mutex<Option<MatrixDisplayMessage>>> = Arc::clone(&shared_message);
        thread::spawn(move || {

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
            let font = LedFont::new(font_file).unwrap();
            
            let (width, _height) = canvas.canvas_size();

            let text = "Hello Neo!";

            let options = TextDrawOptions::new().layout(TextLayout::Wrapped{line_width: width});
            
            canvas.clear();
            canvas.draw_text(&font, text, &options);
            canvas = matrix.swap(canvas);
            
            loop {
                let mut locked_message = message_stread.lock().unwrap();

                if let Some(ref message) = locked_message.take() {
                    canvas.clear();
                    match message {
                        MatrixDisplayMessage::ShwoAnimation => {
                            canvas.draw_text(&font, "Recording started!", &options);
                        }
                        MatrixDisplayMessage::ShowSong{ song_name, artist, _release_year: _, _cover_image: _ } => {
                            let height = canvas.draw_text(&font, song_name, &options);
                            let artist_options = TextDrawOptions::new().layout(TextLayout::Wrapped{line_width: width}).position(0, height);
                            canvas.draw_text(&font, artist, &artist_options);
                        }
                        MatrixDisplayMessage::Off => {
                            // Display nothing
                        }
                    }
                    canvas = matrix.swap(canvas);
                }
            };
        });

        Self { message: shared_message }
    }

    fn set_message(self: &mut Self,message: MatrixDisplayMessage) {
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
            _release_year: release_year.clone(),
            _cover_image: cover_image.clone(),
        });
    }

    pub fn off(self: &mut Self) {
        self.set_message(MatrixDisplayMessage::Off);
    }
}
