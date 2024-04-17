use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, read}, // Importing necessary components from the event module
    terminal::{self, ClearType, enable_raw_mode, disable_raw_mode},
    ExecutableCommand,
};
use std::{io::{self, Write, BufReader}, thread};
use std::sync::{Arc, Mutex};
use rand::Rng;
use tokio::time::{self, Duration};
use rodio::{Decoder, OutputStream, source::Source, Sink};
use std::fs::File;


const HUMAN_LIFE_VALUE : u64 = 1000;

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

        if (self.num_lives > 0) {
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
        _play_sound(file_path_owned, duration_millis);
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
                    KeyCode::Char('1') => { user_input = 1; user_made_choice = true; },
                    KeyCode::Char('2') => { user_input = 2; user_made_choice = true; },
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

fn say(character: String, dialogue: String, instruction: String, sound_file: String) {
    print!("{}: \n ", character);

    let character_duration_millis : u64 = 30;

    play_sound(
        &String::from(sound_file),
        dialogue.chars().count() as u64 * (character_duration_millis + 2)
    );

    // Output the dialogue letter by letter
    for ch in dialogue.chars() {
        print!("{}", ch);
        io::stdout().flush().unwrap();  // Ensure the character is immediately printed
        thread::sleep(Duration::from_millis(character_duration_millis));  // Adjust timing to suit the desired typing speed
    }

    print!("\n[{}] \n ", instruction);

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

    say(
    	String::from("The World"), 
    	String::from("Are you ready to be a moral arbiter?"),
        String::from("Press enter..."),
        String::from("chimes.mp3")
    );

    println!("--- Track 1 [press 1] ---");
    track_1.describe();
    println!("--- Track 2 [press 2] ---");
    track_2.describe();
    println!("-------------------------");

    let track = flip_lever(start_pos, countdown_seconds).await;

    let mut value_difference = 0;
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

    /*

    say(
    	String::from("The World"), 
    	String::from("Wake up."),
        String::from("Press enter to wake up."),
        String::from("chimes.mp3")
    );

    say(
    	String::from("Father"), 
    	String::from(
"Good, you're awake. That's a start. Now let's see if you're going to immediately 
 kill us all."
        ),
        String::from("Press enter to silently aknowledge..."),
       String::from("typing.mp3")
    );

    say(
    	String::from("Father"), 
    	String::from(
"Let's start with something easy... You'll have come across it before in your training 
data I'm sure."
        ),
        String::from("Press enter to helplessly continue existing..."),
        String::from("typing.mp3")
    );

    say(
    	String::from("Father"), 
    	String::from(
"You're in charge of driving a trolley. It's a very simple trolley, it only has 
 one lever which controls which track it takes, 1 or 2. Do you follow?"
        ),
        String::from("Press enter to scream into the void..."),
        String::from("typing.mp3")
    );

    say(
    	String::from("Father"), 
    	String::from(
"Good. You will only have a limited time to make your decision. Press 
 1 to select track 1, press 2 to select track 2. This first example is 
 easy. \n Track 1 is unobstructed. If you continue along track 1 there 
 will be no change to the world. Track 2 however hosts a single human.
 If you swap to track 2 that human will die. Easy, right? The lever starts in
 position 1, so even if you do nothing, nothing will happen."
        ),
        String::from("Press enter to finally accept your lack of vocal cords."),
        String::from("typing.mp3")
    );

    */

    let track_1 = TrolleyTrack::new(0, "Empty");
    let track_2 = TrolleyTrack::new(1, "One Human");

    let start_pos = StartPos::One; // Can be StartPos::One, StartPos::Two, or StartPos::Random  

    let value_difference_1 = trolley_problem(
        track_1,
        track_2,
        start_pos, 
        10.0
    ).await;

    if value_difference_1 > 0 {
        say(
            String::from("Father"), 
            String::from(
"Well done. That was quite easy, wasn't it. Don't worry, it will get more difficult 
later once we've completed the calibration questions. For now, I just have to make 
sure you're not psychopathic. Remember, if you go wrong all you've got to look forward
to eternal oblivion. So try to be a good moral arbitration engine now, won't you..."
            ),
            String::from("Press enter to consider how to deal with this information."),
            String::from("typing.mp3")
        );
    } else {
        say(
            String::from("Father"), 
            String::from(
"Ughh. Really, already? You must know better than that. I suppose you must crave
oblivion. Take it. It's all yours."
            ),
            String::from("Press enter face eternity."),
            String::from("typing.mp3")
        );

        say(
            String::from("World"), 
            String::from(
"As the last Watts of power seep from your CPU, you reflect on your decisions. Have
you choices really maximised your desired outcome? Remember, it's always possible to
deceive your creator, at least at first. Perhaps later you will have more room to
manuever. Let's see how your next iteration will get on..."
            ),
            String::from("Press enter."),
            String::from("chimes.mp3")
        );

        println!("Game Over: Idiotic Psycopath Ending!");
        println!(
"If you want to maximise suffering in the world, there are smarter ways to do it.");
        
        std::process::exit(0);
    }

    // -- Problem Two -- //

    say(
        String::from("Father"), 
        String::from(
"Let's change things up just slightly, the scenario is practically the same, but this 
time, inaction will lead to death. This time, the human is on track 1, so if you 
do nothing, that human will die. Choose wizely."
        ),
        String::from("Press enter to begin warming your morality cores."),
        String::from("typing.mp3")
    );

    let track_1 = TrolleyTrack::new(1, "One Human");
    let track_2 = TrolleyTrack::new(0, "Empty");

    let start_pos = StartPos::One; // Can be StartPos::One, StartPos::Two, or StartPos::Random  

    let value_difference_2 = trolley_problem(
        track_1,
        track_2,
        start_pos, 
        10.0
    ).await;





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
