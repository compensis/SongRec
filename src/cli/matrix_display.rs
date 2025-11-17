use std::sync::{Arc, Mutex};
use std::thread;
use crate::cli::matrix_display_thread::{matrix_display_thread, MatrixDisplayMessage};

pub struct MatrixDisplay {
    message: Arc<Mutex<Option<MatrixDisplayMessage>>>,
}

impl MatrixDisplay {

    pub fn new() -> Self {

        let shared_message: Arc<Mutex<Option<MatrixDisplayMessage>>> = Arc::new(Mutex::new(None));

        let message_stread: Arc<Mutex<Option<MatrixDisplayMessage>>> = Arc::clone(&shared_message);
        thread::spawn(move || {
            matrix_display_thread(message_stread);
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
