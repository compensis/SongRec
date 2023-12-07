use crate::fingerprinting::signature_format::DecodedSignature;
#[cfg(feature = "gui")]
use crate::gui::preferences::Preferences;
#[cfg(feature = "gui")]
use crate::utils::csv_song_history::SongHistoryRecord;
/// This module contains code used from message-based communication between threads.

pub struct SongRecognizedMessage {
    pub artist_name: String,
    pub album_name: Option<String>,
    pub song_name: String,
    pub cover_image: Option<Vec<u8>>,
    pub signature: Box<DecodedSignature>,

    // Used only in the CSV export for now:
    pub track_key: String,
    pub release_year: Option<String>,
    pub genre: Option<String>,

    pub shazam_json: String,
}

#[cfg(feature = "gui")]
pub enum GUIMessage {
    ErrorMessage(String),
    // A list of audio devices, received from the microphone thread
    // because CPAL can't be called from the same thread as the GUI
    // under Windows
    DevicesList(Box<Vec<String>>),
    UpdatePreference(Preferences),
    AddFavorite(SongHistoryRecord),
    RemoveFavorite(SongHistoryRecord),
    ShowFavorites,
    NetworkStatus(bool), // Is the network reachable?
    WipeSongHistory,
    MicrophoneRecording,
    MicrophoneVolumePercent(f32),
    SongRecognized(Box<SongRecognizedMessage>),
}

#[cfg(not(feature = "gui"))]
pub enum GUIMessage {
    ErrorMessage(String),
    // A list of audio devices, received from the microphone thread
    // because CPAL can't be called from the same thread as the GUI
    // under Windows
    DevicesList(Box<Vec<String>>),
    NetworkStatus(bool), // Is the network reachable?
    MicrophoneRecording,
    MicrophoneVolumePercent(f32),
    SongRecognized(Box<SongRecognizedMessage>),
}

pub enum MicrophoneMessage {
    MicrophoneRecordStart(String), // The argument is the audio device name
    MicrophoneRecordStop,
    ProcessingDone,
}

pub enum ProcessingMessage {
    ProcessAudioFile(String),
    ProcessAudioSamples(Box<Vec<i16>>), // Prefer to use heap across threads to avoid stack overflow
}

pub enum HTTPMessage {
    RecognizeSignature(Box<DecodedSignature>),
}
