use crate::matrix_display::matrix_display_thread::{
    matrix_display_thread, MatrixDisplayMessage, SharedMatrixDisplayMessage,
};
use std::thread;

pub struct MatrixDisplay {
    message: SharedMatrixDisplayMessage,
}

impl MatrixDisplay {
    pub fn new() -> Self {
        let shared_message = SharedMatrixDisplayMessage::new();

        thread::spawn({
            let shared_message = shared_message.share();
            move || {
                matrix_display_thread(shared_message);
            }
        });

        Self {
            message: shared_message,
        }
    }

    pub fn init(self: &mut Self) {
        self.message.set(MatrixDisplayMessage::ShowAnimation);
    }

    pub fn show_song(
        self: &mut Self,
        song_name: &str,
        artist: &str,
        release_year: &Option<String>,
        cover_image: &Option<Vec<u8>>,
    ) {
        self.message.set(MatrixDisplayMessage::ShowSong {
            song_name: song_name.to_owned(),
            artist: artist.to_owned(),
            release_year: release_year.clone(),
            cover_image: cover_image.clone(),
        });
    }

    pub fn off(self: &mut Self) {
        self.message.set(MatrixDisplayMessage::Off);
    }
}
