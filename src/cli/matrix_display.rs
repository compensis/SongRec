use std::thread;
use crate::cli::matrix_display_thread::{matrix_display_thread, MatrixDisplayMessage, SharedMatrixDisplayMessage};

pub struct MatrixDisplay {
    message: SharedMatrixDisplayMessage,
}

impl MatrixDisplay {

    pub fn new() -> Self {

        let shared_message = SharedMatrixDisplayMessage::new();

        let message_stread = shared_message.clone();
        thread::spawn(move || {
            matrix_display_thread(message_stread);
        });

        Self { message: shared_message }
    }

    pub fn init(self: &mut Self) {
        self.message.set(MatrixDisplayMessage::ShwoAnimation);
    }

    pub fn show_song(self: &mut Self, song_name: &str, artist: &str, release_year: &Option<String>, cover_image: &Option<Vec<u8>>) {
        self.message.set(MatrixDisplayMessage::ShowSong{
            song_name: song_name.to_string(),
            artist: artist.to_string(),
            release_year: release_year.clone(),
            cover_image: cover_image.clone(),
        });
    }

    pub fn off(self: &mut Self) {
        self.message.set(MatrixDisplayMessage::Off);
    }
}
