use std::io::Write;
use std::process::Command;
use std::process::Stdio;

pub struct MatrixDisplay {
    process: Option<std::process::Child>,
    command: std::process::Command,
}

impl MatrixDisplay {
    const CHARS_PER_LINE: usize = 12;
    const DATA_LINK_ESCAPE: char = '\u{10}';

    pub fn new() -> Self {
        let exe = std::path::Path::new("matrixdisplay");
        let current_exe = std::env::current_exe().unwrap();
        let dir = current_exe.parent().unwrap();
        // exe in same dir
        let mut program = dir.join(exe);
        if let Ok(false) = program.try_exists()  {
            // exe in subdir
            program = dir.join("matrix-display").join(exe);
        }
        if let Ok(false) = program.try_exists()  {
            // exe relativ to target/release/
            program = dir.join("../../matrix-display").join(exe);
        }

        println!("program: {}", program.display());

        Self {
            process: None,
            command: Command::new(program),
        }
    }

    pub fn init(&mut self) {
        let current_exe = std::env::current_exe().unwrap();
        let dir = current_exe.parent().unwrap();
        if let Ok(child) = self.command.current_dir(dir).stdin(Stdio::piped()).spawn() {
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
        let current_exe = std::env::current_exe().unwrap();
        let dir = current_exe.parent().unwrap();
        let mut process = self.command.current_dir(dir).stdin(Stdio::piped()).spawn().unwrap();
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
