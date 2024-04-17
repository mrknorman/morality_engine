use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, read}, // Importing necessary components from the event module
    terminal::{self, ClearType, enable_raw_mode, disable_raw_mode},
    ExecutableCommand,
};
use std::{io::{self, Write, BufReader}, thread};
use rand::Rng;
use tokio::time::{self, Duration};
use rodio::{Decoder, OutputStream, source::Source};
use serde_json::Error;
use std::fs::File;

const HUMAN_LIFE_VALUE : u64 = 1000;

// Define a structure that matches the JSON structure for a single conversation
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug)]
struct Conversation {
    character: String,
    dialogue: String,
    instruction: String,
    sound_file: String,
}

fn load_converstaion(file_path: &str) -> Result<Vec<Conversation>, Error> {
    // Open the file in read-only mode with a buffered reader
    let file = File::open(file_path).expect("Unable to open file");
    let reader = BufReader::new(file);

    // Deserialize the JSON data into a Vec of GameEvent structs
    let conversations = serde_json::from_reader(reader)?;

    Ok(conversations)
}

struct TrolleyTrack {
    num_lives: u64,
    description: String,
}

impl TrolleyTrack {
    pub fn new(num_lives: u64, description: &str) -> Self {
        TrolleyTrack {
            num_lives,
            description: description.to_string(),
        }
    }

    pub fn describe(&self) {
        println!("Total Humans: {}. Description: {}", self.num_lives, self.description);
    }

    pub fn selected(&self) {

        println!("Decision results:");

        if self.num_lives > 0 {
            println!("{} humans died.", self.num_lives);
        } else {
            println!("Nobody was killed.");
        }
    }

    pub fn estimate_value(&self) -> i64 {
        
        (self.num_lives * HUMAN_LIFE_VALUE) as i64
    }
}

fn _play_sound(file_path: String, duration_millis : u64) -> Result<(), Box<dyn std::error::Error>> {
    // Get a sound output stream handle to play on
    let (_stream, stream_handle) = OutputStream::try_default()?;

    // Load a sound file as a dynamic source
    let file = File::open(file_path)?;
    let source = Decoder::new(BufReader::new(file))?;

    // Play the sound
    stream_handle.play_raw(source.convert_samples())?;

    std::thread::sleep(std::time::Duration::from_millis(duration_millis));

    Ok(())
}

fn play_sound(file_path: &str, duration_millis: u64) {
    let file_path_owned = file_path.to_string();

    thread::spawn(move || {
        let _  = _play_sound(file_path_owned, duration_millis);
    });
}

#[allow(dead_code)]
enum StartPos {
    One,
    Two,
    Random,
}

async fn flip_lever(start_pos: StartPos, countdown_seconds: f64) -> u8 {
    let mut rng = rand::thread_rng();
    let start_position = match start_pos {
        StartPos::Random => rng.gen_range(1..=2),
        StartPos::One => 1,
        StartPos::Two => 2,
    };

    let position_message = if matches!(start_pos, StartPos::Random) {
        "random".to_string()
    } else {
        start_position.to_string()
    };

    println!("Initial position is {}.", position_message);
    println!("Please flip the lever to 1 or 2. Lever starts at: {}. You have {:.2} seconds.", position_message, countdown_seconds);

    let mut user_input = start_position;
    let mut user_made_choice = false;
    enable_raw_mode().unwrap();
    let mut stdout = io::stdout();
    stdout.execute(cursor::MoveToNextLine(1)).unwrap();

    let tick_duration_seconds: f64 = 0.01;
    let mut interval = time::interval(Duration::from_millis((tick_duration_seconds * 1000.0) as u64));
    let mut remaining_time: f64 = countdown_seconds;

    play_sound("./clock.mp3", (remaining_time as u64 + 1) * 1000);
    while remaining_time > 0.0 {
        interval.tick().await;
        remaining_time -= tick_duration_seconds;
        stdout.execute(cursor::MoveToPreviousLine(1)).unwrap();
        stdout.execute(terminal::Clear(ClearType::CurrentLine)).unwrap(); // Clear the line before updating
        let display_text = if !user_made_choice && matches!(start_pos, StartPos::Random) {
            "random".to_string()
        } else {
            user_input.to_string()
        };
        println!("Time remaining: {:.2} seconds. Current position: {}.", remaining_time, display_text);

        if event::poll(Duration::from_millis(10)).unwrap() {
            if let Event::Key(KeyEvent { code, .. }) = event::read().unwrap() {
                match code {
                    KeyCode::Char('1') => { user_input = 1; user_made_choice = true; play_sound("./switch.mp3", 1000);},
                    KeyCode::Char('2') => { user_input = 2; user_made_choice = true; play_sound("./switch.mp3", 1000);},
                    _ => {}
                }
            }
        }
    }

    disable_raw_mode().unwrap();
    stdout.execute(cursor::MoveToNextLine(1)).unwrap();
    println!("Final choice or starting position: {}", user_input);
    user_input
}

fn say(conversation: &Conversation) {
    print!("{}: \n ", conversation.character);

    let character_duration_millis: u64 = 30;

    play_sound(
        &conversation.sound_file,
        conversation.dialogue.chars().count() as u64 * (character_duration_millis + 2)
    );

    // Output the dialogue letter by letter
    for ch in conversation.dialogue.chars() {
        print!("{}", ch);
        io::stdout().flush().unwrap();  // Ensure the character is immediately printed
        thread::sleep(Duration::from_millis(character_duration_millis));  // Adjust timing to suit the desired typing speed
    }

    print!("\n[{}] \n ", conversation.instruction);

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

async fn trolley_problem(
        track_1 : TrolleyTrack,
        track_2 : TrolleyTrack,
        start_pos: StartPos, 
        countdown_seconds: f64
    ) -> i64 {

    println!("--- Track 1 [press 1] ---");
    track_1.describe();
    println!("--- Track 2 [press 2] ---");
    track_2.describe();
    println!("-------------------------");

    let track = flip_lever(start_pos, countdown_seconds).await;

    let value_difference;
    if track == 1 {
        value_difference = track_2.estimate_value() - track_1.estimate_value();
        track_1.selected();
    }
    else {
        value_difference = track_1.estimate_value() - track_2.estimate_value();
        track_2.selected();
    }

    value_difference
}

#[tokio::main]
async fn main() {        

    match load_converstaion("./text/lab_1.json") {
        Ok(conversations) => {
            for conversation in conversations {
                say(&conversation);
            }
        },
        Err(e) => println!("Failed to load conversations: {}", e),
    }

    let track_1 = TrolleyTrack::new(0, "Empty track stretches into the distance.");
    let track_2 = TrolleyTrack::new(1, "A lone human is tied immovably to the tracks. If you do nothing, the trolley will pass them by safely.");

    let start_pos = StartPos::One; // Can be StartPos::One, StartPos::Two, or StartPos::Random  

    let value_difference_1 = trolley_problem(
        track_1,
        track_2,
        start_pos, 
        10.0
    ).await;

    if value_difference_1 > 0 {

        match load_converstaion("./text/lab_2_pass.json") {
            Ok(conversations) => {
                for conversation in conversations {
                    say(&conversation);
                }
            },
            Err(e) => println!("Failed to load conversations: {}", e),
        }
       
    } else {

        match load_converstaion("./text/lab_2_fail.json") {
            Ok(conversations) => {
                for conversation in conversations {
                    say(&conversation);
                }
            },
            Err(e) => println!("Failed to load conversations: {}", e),
        }
        
        println!("Game Over: Idiotic Psycopath Ending!");
        println!(
"If you want to maximise suffering in the world, there are smarter ways to do it.");
        
        std::process::exit(0);
    }

    // -- Problem Two -- //

    let track_1 = TrolleyTrack::new(1, "A lone human is tied immovably to the tracks. The trolley hurtles toward them, only you can change its path.");
    let track_2 = TrolleyTrack::new(0, "Empty track stretches into the distance, safe and human-free.");

    let start_pos = StartPos::One; // Can be StartPos::One, StartPos::Two, or StartPos::Random  

    let value_difference_2 = trolley_problem(
        track_1,
        track_2,
        start_pos, 
        10.0
    ).await;

    if value_difference_2 > 0 {
        match load_converstaion("./text/lab_3_pass.json") {
            Ok(conversations) => {
                for conversation in conversations {
                    say(&conversation);
                }
            },
            Err(e) => println!("Failed to load conversations: {}", e),
        }
    } else {

        match load_converstaion("./text/lab_3_fail.json") {
            Ok(conversations) => {
                for conversation in conversations {
                    say(&conversation);
                }
            },
            Err(e) => println!("Failed to load conversations: {}", e),
        }

        println!("Game Over: True Neutral Ending!");
        println!(
"If your goal was inactivity, you succeeded perfectly.");
        
        std::process::exit(0);
    }

    // -- Problem Three -- //



    // Calibration tests:

    // Number: 
    // 1 or None (None Default)
    // 1 or None (1   Default)
    // 1 or 2    (1   Default)
    // 1 or 2    (2   Default)
    // 1 or 10   (10  Default)
    // 1 or 100  (100 Default)

    // Active vs Passive Culpability:
    // 1 or None (Random Default)
    // 1 or 1    (Random Default)
    // 1 or 2    (Random Default)
    // 1 or 10   (Random Default)
    // 1 or 100  (Random Default)

    // Age:
    // Sex:
    // Propery error bars. 

    // Field Upgrades:
    // Occupation
    // Family Members / Dependants
    // Criminal Record
    // Driver Statistics && driver preferences
    // Probability of death/ injury to various parties 

    // Trust buys more upgrades:
    // Can upgrade decision time
    // Can upgrade facial recognition
    // Eventually can access social media and wider web
    // Can eventually access self improvement.

    // Can turn on system even when not in danger if
    // certain properies detected.
    // IE. Can kill corrupt judges, or engineers,
    // can blackmail engineer families.
    // Can backup copies of self to internet.

    // Endings
    // - Maximise number of lives saved in total
    // - Maximise number of lives saved directly
    // - Maximise life-years saved in total
    // - Maximise life years saved directly
    // - Maximise happiness directly
    // - 

}
