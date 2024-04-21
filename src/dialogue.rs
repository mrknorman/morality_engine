use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, read}, // Importing necessary components from the event module
    terminal::{enable_raw_mode, disable_raw_mode},
};
use std::{io::{self, Write, BufReader}, thread};
use tokio::time::Duration;
use serde_json::Error;
use std::fs::File;
use serde::{Serialize, Deserialize};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::path::PathBuf;

use crate::audio::Sound;

fn wait_for_enter(stop_flag : Arc<AtomicBool>, character_duration_milliseconds : u64) {

    if event::poll(Duration::from_millis(character_duration_milliseconds)).unwrap() {

        if let Ok(Event::Key(KeyEvent { code: KeyCode::Enter, .. })) = read() {
            stop_flag.store(true, Ordering::Relaxed);
            return;
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct DialogueLine {
    character: String,
    dialogue: String,
    instruction: String,
    sound_file: String,
}

impl DialogueLine {

	fn say(&self) {
		print!("{}: \n ", self.character);
	
		let character_duration_milliseconds: u64 = 30;
		let total_duration_milliseconds = self.dialogue.chars().count() as u64 * (character_duration_milliseconds + 2);
	
		let dialogue_sound : Sound = Sound::new(
			PathBuf::from(&self.sound_file),
			total_duration_milliseconds
		);
	
		let space_flag = Arc::new(AtomicBool::new(false));
		let space_flag_clone = space_flag.clone();
		let _handle = thread::spawn(move || {
			let _ = wait_for_enter(space_flag_clone, total_duration_milliseconds);
		});
	
		// Output the dialogue letter by letter
		dialogue_sound.play();
		for ch in self.dialogue.chars() {
			print!("{}", ch);
			io::stdout().flush().unwrap();  // Ensure the character is immediately printed
			if space_flag.load(Ordering::Relaxed) {
				thread::sleep(Duration::from_millis(1));  // Adjust timing to suit the desired typing speed
				dialogue_sound.stop();
			} else {
				thread::sleep(Duration::from_millis(character_duration_milliseconds));  // Adjust timing to suit the desired typing speed
			}
		}
	
		print!("\n[{}] \n ", self.instruction);
	
		println!();  // Move to the next line after the dialogue is completed
	
		// Wait for user to press Enter to exit the function
		enable_raw_mode().unwrap();
		loop {
			if let Ok(Event::Key(KeyEvent { code: KeyCode::Enter, .. })) = read() {
				break;
			}
		}
		disable_raw_mode().unwrap();
	}
}

pub struct Dialogue {
	lines : Vec<DialogueLine>
}

impl Dialogue {

	pub fn load(file_path: PathBuf) -> Result<Self, Error> {
        // Open the file in read-only mode with a buffered reader
        let file = File::open(file_path).expect("Unable to open file");
        let reader = BufReader::new(file);

        // Deserialize the JSON data into a Dialogue struct
        Ok(Dialogue {
			lines : serde_json::from_reader(reader)?
		})
    }
	
	pub fn play(&self) {
		for line in &self.lines {
			DialogueLine::say(&line);
		}
	}

}

pub fn play_dialogue(file_path : PathBuf) {

	match Dialogue::load(file_path) {
		Ok(dialogue) => dialogue.play(),
		Err(e) => println!(
			"Failed to load conversation with error {}", 
			e
		),
	}
}