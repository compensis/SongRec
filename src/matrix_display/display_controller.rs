use crate::core::thread_messages::SongRecognizedMessage;
use crate::matrix_display::matrix_display::MatrixDisplay;

enum TimeoutCounterState {
    Stopped,
    Countdown(i32),
}

struct TimeoutCounter {
    timeout: i32,
    state: TimeoutCounterState,
}

impl TimeoutCounter {
    fn new(timeout: i32) -> Self {
        Self {
            timeout,
            state: TimeoutCounterState::Stopped,
        }
    }

    fn start(self: &mut Self) -> bool {
        let restarted = match self.state {
            TimeoutCounterState::Stopped => true,
            _ => false,
        };
        self.state = TimeoutCounterState::Countdown(self.timeout);
        restarted
    }

    fn is_expired(self: &mut Self) -> bool {
        let mut expired = false;
        self.state = match self.state {
            TimeoutCounterState::Countdown(t) if t == 0 => {
                expired = true;
                TimeoutCounterState::Stopped
            }
            TimeoutCounterState::Countdown(t) => TimeoutCounterState::Countdown(t - 1),
            TimeoutCounterState::Stopped => TimeoutCounterState::Stopped,
        };
        expired
    }
}

/// Enum to represent the result of `handle_no_match`.
pub enum NoMatchResult {
    Reset,
    NoAction,
}

pub struct DisplayController {
    matrix_display: MatrixDisplay,
    no_match_timeout_counter: TimeoutCounter,
    microphone_timeout_counter: TimeoutCounter,
}

impl DisplayController {
    pub fn new() -> Self {
        Self {
            matrix_display: MatrixDisplay::new(),
            no_match_timeout_counter: TimeoutCounter::new(5),
            microphone_timeout_counter: TimeoutCounter::new(1000),
        }
    }

    /// Handles the "no match" scenario and returns a `NoMatchResult`.
    pub fn handle_no_match(self: &mut Self) -> NoMatchResult {
        if self.no_match_timeout_counter.is_expired() {
            self.matrix_display.init();
            return NoMatchResult::Reset;
        }
        NoMatchResult::NoAction
    }

    pub fn handle_recording_started(self: &mut Self) {
        self.matrix_display.init();
    }

    pub fn handle_song_recognized(self: &mut Self, message: Box<SongRecognizedMessage>) {
        self.matrix_display.show_song(
            message.song_name,
            message.artist_name,
            message.release_year,
            message.cover_image,
        );
        self.no_match_timeout_counter.start();
    }

    pub fn handle_microphone_volume_high(self: &mut Self) {
        if self.microphone_timeout_counter.start() {
            eprintln!("Display on!");
            self.matrix_display.init();
        }
    }

    pub fn handle_microphone_volume_low(self: &mut Self) {
        if self.microphone_timeout_counter.is_expired() {
            eprintln!("Display off!");
            self.matrix_display.off();
        }
    }
}
