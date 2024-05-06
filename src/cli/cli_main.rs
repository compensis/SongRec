use std::cell::RefCell;
use std::error::Error;
use std::rc::Rc;
use std::sync::{mpsc, Arc};

use std::io::Write;
use std::process::Command;
use std::process::Stdio;

use chrono::Local;
use gettextrs::gettext;
use glib;
use glib::clone;

use mpris_player::PlaybackStatus;

use crate::core::http_thread::http_thread;
use crate::core::microphone_thread::microphone_thread;
use crate::core::processing_thread::processing_thread;
use crate::core::thread_messages::{GUIMessage, MicrophoneMessage, ProcessingMessage, SongRecognizedMessage};

use crate::utils::csv_song_history::SongHistoryRecord;
use crate::utils::mpris_player::{get_player, update_song};
use crate::utils::thread::spawn_big_thread;

struct MatrixDisplay {
    process: Option<std::process::Child>,
    command: std::process::Command,
    //process: std::process::Child,
}

impl MatrixDisplay {
    const CHARS_PER_LINE: usize = 12;
    const DATA_LINK_ESCAPE: char = '\u{10}';

    pub fn new() -> Self {
        Self {
            process: None,
            command: Command::new("matrix-display/matrixdisplay"),
        }
    }

    pub fn init(&mut self) {
        if let Ok(child) = self.command.stdin(Stdio::piped()).spawn() {
            self.process = Some(child);
        };
    }

    pub fn clear_screen(&self, ) {
        if let Some(process) = &self.process {
            let mut stdin = process.stdin.as_ref().unwrap();
            writeln!(stdin, "").unwrap();
        }
    }

    pub fn writeln(&self, line: &str) {
        if let Some(process) = &self.process {
            let mut stdin = process.stdin.as_ref().unwrap();
            let line = &textwrap::fill(&line, Self::CHARS_PER_LINE);
            writeln!(stdin, "{}", line).unwrap();
        }
    }

    pub fn show_image(&mut self, image: Vec<u8>) {
        if let Some(mut process) = self.process.take() {
            process.kill().unwrap();
        }
        // Start a new matrix display program process, becaus we need to drop stdin aft the
        // image is written so we cant reuse it next time
        let mut process = self.command.stdin(Stdio::piped()).spawn().unwrap();
        let mut stdin = process.stdin.take().unwrap();
        writeln!(stdin, "{},", Self::DATA_LINK_ESCAPE).unwrap();
        stdin.write_all(&image).unwrap();
        // Drop stdin after the image is written, to close stdin,
        // to show that this was all the data
        drop(stdin);
        // Keep process so that the image is still displayed
        self.process = Some(process);
    }
}

pub enum CLIOutputType {
    SongName,
    MatrixDisplay,
    JSON,
    CSV,
}

pub struct CLIParameters {
    pub enable_mpris: bool,
    pub recognize_once: bool,
    pub audio_device: Option<String>,
    pub input_file: Option<String>,
    pub output_type: CLIOutputType,
}

pub fn cli_main(parameters: CLIParameters) -> Result<(), Box<dyn Error>> {
    glib::MainContext::default().acquire();
    let main_loop = Arc::new(glib::MainLoop::new(None, false));

    let (gui_tx, gui_rx) = glib::MainContext::channel(glib::PRIORITY_DEFAULT);
    let (microphone_tx, microphone_rx) = mpsc::channel();
    let (processing_tx, processing_rx) = mpsc::channel();
    let (http_tx, http_rx) = mpsc::channel();

    let processing_microphone_tx = processing_tx.clone();
    let microphone_http_tx = microphone_tx.clone();

    spawn_big_thread(
        clone!(@strong gui_tx => move || { // microphone_rx, processing_tx
            microphone_thread(microphone_rx, processing_microphone_tx, gui_tx);
        }),
    );

    spawn_big_thread(clone!(@strong gui_tx => move || { // processing_rx, http_tx
        processing_thread(processing_rx, http_tx, gui_tx);
    }));

    spawn_big_thread(clone!(@strong gui_tx => move || { // http_rx
        http_thread(http_rx, gui_tx, microphone_http_tx);
    }));

    // recognize once if an input file is provided
    let do_recognize_once = parameters.recognize_once || parameters.input_file.is_some();

    // do not enable mpris if recognizing one song
    let do_enable_mpris = parameters.enable_mpris && !do_recognize_once;

    let mpris_player = if do_enable_mpris { get_player() } else { None };
    let last_track: Rc<RefCell<Option<String>>> = Rc::new(RefCell::new(None));

    let main_loop_cli = main_loop.clone();

    let audio_dev_name = parameters.audio_device.as_ref().map(|dev| dev.to_string());
    let input_file_name = parameters.input_file.as_ref().map(|dev| dev.to_string());

    if let Some(ref filename) = parameters.input_file {
        processing_tx
            .send(ProcessingMessage::ProcessAudioFile(filename.to_string()))
            .unwrap();
    }

    let mut csv_writer = csv::Writer::from_writer(std::io::stdout());

    let mut matrix_display = MatrixDisplay::new();
    if let CLIOutputType::MatrixDisplay = parameters.output_type {
        matrix_display.init();
    }
    
    //let mut matrix_display = MatrixDisplay::new();

    //let mut command = Command::new("matrix-display/matrixdisplay");
    //let mut matrix_display_process = command.stdin(Stdio::piped()).spawn().unwrap();

    gui_rx.attach(None, move |gui_message| {
        match gui_message {
            GUIMessage::DevicesList(device_names) => {
                // no need to start a microphone if recognizing from file
                if input_file_name.is_some() {
                    return glib::Continue(true);
                }
                let dev_name = if let Some(dev) = &audio_dev_name {
                    if !device_names.contains(dev) {
                        eprintln!("{}", gettext("Exiting: audio device not found"));
                        main_loop_cli.quit();
                        return glib::Continue(false);
                    }
                    dev
                } else {
                    if device_names.is_empty() {
                        eprintln!("{}", gettext("Exiting: no audio devices found!"));
                        main_loop_cli.quit();
                        return glib::Continue(false);
                    }
                    &device_names[0]
                };
                eprintln!("{} {}", gettext("Using device"), dev_name);
                microphone_tx
                    .send(MicrophoneMessage::MicrophoneRecordStart(
                        dev_name.to_owned(),
                    ))
                    .unwrap();
            }
            GUIMessage::NetworkStatus(reachable) => {
                let mpris_status = if reachable {
                    PlaybackStatus::Playing
                } else {
                    PlaybackStatus::Paused
                };
                mpris_player
                    .as_ref()
                    .map(|p| p.set_playback_status(mpris_status));

                if !reachable {
                    if input_file_name.is_some() {
                        eprintln!("{}", gettext("Error: Network unreachable"));
                        main_loop_cli.quit();
                        return glib::Continue(false);
                    } else {
                        eprintln!("{}", gettext("Warning: Network unreachable"));
                    }
                }
            }
            GUIMessage::ErrorMessage(string) => {
                if !(string == gettext("No match for this song") && !input_file_name.is_some()) {
                    eprintln!("{} {}", gettext("Error:"), string);
                }
                if input_file_name.is_some() {
                    main_loop_cli.quit();
                    return glib::Continue(false);
                }
            }
            GUIMessage::MicrophoneRecording => {
                if !do_recognize_once {
                    eprintln!("{}", gettext("Recording started!"));
                    if let CLIOutputType::MatrixDisplay = parameters.output_type {
                        matrix_display.writeln("Recording started!");
                    }
                }
            }
            GUIMessage::SongRecognized(message) => {
                let mut last_track_borrow = last_track.borrow_mut();
                let track_key = Some(message.track_key.clone());

                if *last_track_borrow != track_key {
                    mpris_player.as_ref().map(|p| update_song(p, &message));
                    *last_track_borrow = track_key;

                    let song_name = &message.song_name;
                    let artist_name= &message.artist_name;
    
                    #[cfg(feature = "textwrap")]
                    let song_name = &textwrap::fill(&song_name, textwrap::termwidth());
    
                    #[cfg(feature = "textwrap")]
                    let artist_name = &textwrap::fill(&artist_name, textwrap::termwidth());

                    match parameters.output_type {
                        CLIOutputType::JSON => {
                            println!("{}", message.shazam_json);
                        }
                        CLIOutputType::CSV => {
                            csv_writer.serialize(get_song_history_record(message)).unwrap();
                            csv_writer.flush().unwrap();
                        }
                        CLIOutputType::MatrixDisplay => {
                            if let Some(cover) = message.cover_image {
                                matrix_display.show_image(cover);

                                println!("{} - {}", artist_name, song_name);
                            }
                            else {
                                matrix_display.clear_screen();
                                matrix_display.writeln(song_name);
                                matrix_display.writeln(artist_name);
                            }
                        }
                        #[cfg(not(feature = "slowprint"))]
                        CLIOutputType::SongName => {
                            println!("{} - {}", artist_name, song_name);
                        }
                        #[cfg(feature = "slowprint")]
                        CLIOutputType::SongName => {
                            clearscreen::clear().unwrap();
                            let delay = std::time::Duration::from_millis(100);
                            slowprint::slow_println(song_name, delay);
                            slowprint::slow_println(artist_name, delay);
                        }
                    };
                }
                if do_recognize_once {
                    microphone_tx.send(MicrophoneMessage::MicrophoneRecordStop).unwrap();
                    main_loop_cli.quit();
                    return glib::Continue(false);
                }
            }
            _ => {}
        }
        glib::Continue(true)
    });

    main_loop.run();
    Ok(())
}

fn get_song_history_record(message: Box<SongRecognizedMessage>) -> SongHistoryRecord {
    let _song_name = format!("{} - {}", message.artist_name, message.song_name);
    SongHistoryRecord {
        song_name: _song_name,
        album: Some(
            message
                .album_name
                .as_ref()
                .unwrap_or(&"".to_string())
                .to_string(),
        ),
        track_key: Some(message.track_key),
        release_year: Some(
            message
                .release_year
                .as_ref()
                .unwrap_or(&"".to_string())
                .to_string(),
        ),
        genre: Some(
            message
                .genre
                .as_ref()
                .unwrap_or(&"".to_string())
                .to_string(),
        ),
        recognition_date: Local::now().format("%c").to_string(),
    }
}
