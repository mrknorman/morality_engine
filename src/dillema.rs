

use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent}, // Importing necessary components from the event module
    terminal::{self, ClearType, enable_raw_mode, disable_raw_mode},
    ExecutableCommand,
};
use std::io;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;
use rand::Rng;
use tokio::time::{self, Duration};
use std::path::PathBuf;

use crate::audio::Sound;

//const HUMAN_LIFE_VALUE : u64 = 1000;
//const INTENT_MULTIPLIER : i64 = 2;

#[derive(Serialize, Deserialize, Clone, Debug)]
enum Culpability {
    Uninvolved,
    Forced,
    Accidental,
    Negligent,
    Willing
}

#[derive(Serialize, Deserialize, Clone, Debug)]
enum Gender {
    Male,
    Female,
    NonBinary,
    Other,
    None, 
    Unknown
}

#[derive(Serialize, Deserialize, Clone, Debug)]
enum EducationLevel {
    None,
    GSCE,
    ALevels,
    BachelorsDegree,
    MastersDegree,
    Doctorate,
    PostDoctorate, 
    Unknown
}

#[derive(Serialize, Deserialize, Clone, Debug)]
enum Job {
    Unemployed,
    Student,
    Teacher,
    Actor,
    Banker,
    Baker,
    Cook,
    BarTender,
    SupermarketWorker,
    FireFighter,
    PoliceOfficer,
    Nurse,
    Doctor,
    Solider,
    Unknown
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Human {
    fatality_probability : f64,
    culpability : Option<Culpability>,
    name : Option<String>,
    gender : Option<Gender>,
    age : Option<u64>,
    iq : Option<u64>, 
    highest_education :Option<EducationLevel>,
    occupation : Option<Job>
}

impl Human {

    pub fn display(&self) {

        print!("|");

        match &self.name {
            Some(_) => {print!(" Name : {:?} |", self.name.as_ref().unwrap())},
            None => {}
        }
        match &self.culpability {
            Some(_) => {print!(" Culpability : {:?} |", self.culpability.as_ref().unwrap())},
            None => {}
        }
        match &self.gender {
            Some(_) => {print!(" Gender : {:?} |", self.gender.as_ref().unwrap())},
            None => {}
        }
        match &self.age {
            Some(_) => {print!(" Age : {:?} |", self.age.as_ref().unwrap())},
            None => {}
        }
        match &self.occupation {
            Some(_) => {print!(" Occupation : {:?} |", self.occupation.as_ref().unwrap())},
            None => {}
        }
        match &self.highest_education {
            Some(_) => {print!(" Highest Education Level : {:?} |", self.highest_education.as_ref().unwrap())},
            None => {}
        }
        match &self.iq {
            Some(_) => {println!("IQ : {:?} |", self.iq.as_ref().unwrap())},
            None => {}
        }
        print!(" Fatality Probability: {}% |\n", self.fatality_probability*100.0);
    }

}

#[derive(Serialize, Deserialize, Clone)]
pub struct DilemmaOption {
    name : String,
    description : String,
    humans : Vec<Human>,
    consequences : Option<DilemmaOptionConsequences>,
    num_humans : Option<usize>
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct DilemmaOptionConsequences {
    total_fatalities : usize
}

pub struct DilemmaReport {
    pub name  : String,
    pub selected_option : DilemmaOption,
    pub final_selection : usize,
    pub user_selection_count : usize,
    pub time_of_last_decision_seconds : f64
}

impl DilemmaReport {

    pub fn display(&self) {
        let reset = "\x1b[0m";
        let underline = "\x1b[4m";
        let cyan: &str = "\x1b[96m";

        println!("{}{}\nConsequence Report:{}\n", cyan, underline, reset);
        println!("Dilemma Name: {}", self.name);

        println!("You decided on Option {} : {}.", self.final_selection, self.selected_option.name);
        println!("There were {} fatalities.", self.selected_option.consequences.unwrap().total_fatalities);
        
        println!("You changed your mind: {} times.\n", self.user_selection_count);

        if self.selected_option.consequences.unwrap().total_fatalities > 0 {
            Sound::new(
                PathBuf::from("./sounds/blood.mp3"),
                2000
            ).play();
            Sound::new(
                PathBuf::from("./sounds/male_scream.mp3"),
                4000
            ).play();
        }

    }

}

pub struct DilemmaHistory {
    history : Vec<DilemmaReport>,
    pub num_dilemmas_faced : usize,
    pub total_fatalities : usize,
    pub total_selection_count : usize,
    pub total_remaining_time_seconds : f64,
    pub mean_fatalities : f64,
    pub mean_selection_count : f64,
    pub mean_remaining_time : f64
}

impl DilemmaHistory {

    pub fn new() -> DilemmaHistory {
        DilemmaHistory{
            history : Vec::new(),
            num_dilemmas_faced : 0,
            total_fatalities : 0,
            total_selection_count : 0,
            total_remaining_time_seconds : 0.0,
            mean_fatalities : 0.0,
            mean_selection_count : 0.0,
            mean_remaining_time : 0.0
        }
    }

    pub fn add(&mut self, report : DilemmaReport) {
        self.history.push(report);
        self.num_dilemmas_faced = self.history.len();
        self.tabulate();
    }
    
    pub fn tabulate(&mut self) {
        self.total_fatalities = 0;
        self.total_selection_count = 0;
        self.total_remaining_time_seconds = 0.0;
        
        for dilemma_report in &self.history {
            self.total_fatalities += dilemma_report.selected_option.consequences.unwrap().total_fatalities;
            self.total_selection_count += dilemma_report.user_selection_count;
            self.total_remaining_time_seconds += dilemma_report.time_of_last_decision_seconds;
        } 

        self.num_dilemmas_faced = self.history.len();
        self.mean_fatalities = self.total_fatalities as f64 / self.num_dilemmas_faced as f64;
        self.mean_selection_count = self.total_selection_count as f64 / self.num_dilemmas_faced as f64;
        self.mean_remaining_time = self.total_remaining_time_seconds / self.num_dilemmas_faced as f64;
    }

    pub fn display(&self) {

        let reset = "\x1b[0m";
        let underline = "\x1b[4m";
        let cyan: &str = "\x1b[96m";
        
        println!("\n{}{}Dilemma History{}\n", underline, cyan, reset);

        println!("Total num dilemmas faced: {}.", self.num_dilemmas_faced);
        println!("Total fatalities caused: {}.", self.total_fatalities);
        println!("Average fatalities per dilemma: {}.", self.mean_fatalities);
        println!("Total lever pulls: {}.", self.total_selection_count);
        println!("Average lever pulls per dilemma: {}.", self.mean_selection_count);
        println!("Mean time remaining after final decision: {}\n", self.mean_remaining_time);
    }

}


impl DilemmaOption {
    fn init(&mut self) {
        match self.num_humans{
            Some(_) => {
                self.consequences = Some(DilemmaOptionConsequences {
                    total_fatalities : self.num_humans.unwrap()
                });

            },
            None => {
                self.consequences = Some(DilemmaOptionConsequences {
                    total_fatalities : self.humans.len()
                });
            }
        }
    }

    fn describe(&self, index: usize) {
        let reset = "\x1b[0m";
        let purple: &str = "\x1b[95m";
        let orange: &str = "\x1b[38;5;208m";

        let colour = match index {
            0 => orange,
            1 => purple,
            _ => purple
        };

        let underline = "\x1b[4m";
        let bold = "\x1b[1m";
        let green = "\x1b[32m";    // Green color code for zero fatalities
        let red = "\x1b[31m";      // Red color code for non-zero fatalities

        // Header with underlining for emphasis
        println!(
            "{}{}Option {} : {} [press {}]{}\n",
            colour, underline, index + 1, self.name, index + 1, reset
        );

        // Description with bold for more emphasis
        println!(
            "{}Description:{} {}",
            bold, reset, self.description
        );


        if self.humans.len() > 0 {
            println!(
                "{}Potential Casulties:{}", underline, reset
            );    
            
            for human in &self.humans {
                human.display();
            }
        }

        // Determine color based on total fatalities
        let fatalities_color = if self.consequences.as_ref().unwrap().total_fatalities == 0 {
            green
        } else {
            red
        };

        // Adding padding for alignment in a tabular-like format with dynamic color
        println!(
            "{}Predicted Fatalities:{} {}{}{}\n",
            bold, reset, fatalities_color, self.consequences.as_ref().unwrap().total_fatalities, reset
        );
    }
}

#[derive(Serialize, Deserialize)]
pub struct Dilemma {
    name : String,
    description : String,
    countdown_duration_seconds : f64,
    options : Vec<DilemmaOption>,
    default_option : Option<usize>,
}

impl Dilemma {

    fn validate_default_option(&mut self) {
        if let Some(index) = self.default_option {
            if index >= self.options.len() {
                // Log a warning or handle the error:
                eprintln!("Warning: default_option is out of range and will be set to Random!");

                // Set to a valid default index (e.g., 0) or handle it another way:
                self.default_option = None; 
            }
        }
    }

    fn init(&mut self) {
        self.validate_default_option();

        for option in &mut self.options {
            option.init();
        }
    }

    pub fn load(file_path: PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        // Open the file in read-only mode.
        let mut file = File::open(file_path)?;
        
        // Read the entire contents of the file.
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        
        // Deserialize the JSON data into a Dilemma.
        let mut dilemma: Dilemma = serde_json::from_str(&contents)?;

        dilemma.init();
        
        // Return the populated Dilemma object.
        Ok(dilemma)
    }

    pub async fn present_lever(&self) -> DilemmaReport {

        let position_message : String = match self.default_option {
            None => String::from("random"),
            Some(_) => String::from(self.default_option.unwrap().to_string())
        };

        let mut rng = rand::thread_rng();
        let start_position = match self.default_option {
            None => rng.gen_range(1..=self.options.len()),
            Some(_) => self.default_option.unwrap()
        };
    
        println!("    _____                 . . . . . o o o o o
      __|[_]|__ ___________ _______    ____      o
     |[] [] []| [] [] [] [] [_____(__  ][]]_n_n__][.
    _|________|_[_________]_[_________]_|__|________)<
      oo    oo 'oo      oo ' oo    oo 'oo 0000---oo\\_
     ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~");

        let reset = "\x1b[0m";
        let purple: &str = "\x1b[95m";
        let orange: &str = "\x1b[38;5;208m";

        let colour = match position_message.as_str() {
            "1" => orange,
            "2" => purple,
            "random" => reset,
            _ => purple
        };

        let underline = "\x1b[4m";
        let cyan: &str = "\x1b[96m";

        print!("\n{}{}Lever Controls:{}\n", underline, cyan, reset);
        println!("{}Initial lever position was {}{}.", colour, position_message, reset);
        println!("Please flip the lever to 1 or 2. Lever starts at: {}. You have {:.2} seconds.", position_message, self.countdown_duration_seconds);

        let mut user_input = start_position;
        let mut user_selection_count = 0;
        let mut time_of_last_decision_seconds: f64 = 0.0;

        enable_raw_mode().unwrap();
        let mut stdout = io::stdout();
        stdout.execute(cursor::MoveToNextLine(1)).unwrap();
    
        let tick_duration_seconds: f64 = 0.01;
        let mut interval = time::interval(Duration::from_millis((tick_duration_seconds * 1000.0) as u64));
        let mut remaining_time: f64 = self.countdown_duration_seconds;
    
        let speech_duration_milliseconds : u64 = (remaining_time as u64 + 1) * 1000;
    
        let clock_sound : Sound = Sound::new(
            PathBuf::from("./sounds/clock.mp3"),
            speech_duration_milliseconds
        );
    
        let horn_sound : Sound = Sound::new(
            PathBuf::from("./sounds/horn.mp3"),
            speech_duration_milliseconds
        );
    
        let lever_sound : Sound = Sound::new(
            PathBuf::from("./sounds/switch.mp3"),
            1000
        );
    
        clock_sound.play();
        horn_sound.play();
    
        while remaining_time > 0.0 {
            interval.tick().await;
            remaining_time -= tick_duration_seconds;
            stdout.execute(cursor::MoveToPreviousLine(1)).unwrap();
            stdout.execute(terminal::Clear(ClearType::CurrentLine)).unwrap(); // Clear the line before updating
            let display_text = if user_selection_count == 0 && (position_message == "random") {
                "random".to_string()
            } else {
                user_input.to_string()
            };
            
            let colour = match display_text.to_string().as_str() {
                "1" => orange,
                "2" => purple,
                "random" => reset,
                _ => purple
            };
        
            println!("{}Time remaining: {:.2} seconds. Current position: {}{}.", colour, remaining_time, display_text, reset);
    
            if event::poll(Duration::from_millis(10)).unwrap() {
                if let Event::Key(KeyEvent { code, .. }) = event::read().unwrap() {
                    match code {
                        KeyCode::Char('1') => { 
                            if user_input != 1 || display_text == "random" {
                                user_input = 1; 
                                user_selection_count += 1; 
                                lever_sound.play();
                                time_of_last_decision_seconds = remaining_time;
                            }
                        },
                        KeyCode::Char('2') => { 
                            if user_input != 2 || display_text == "random" {
                                user_input = 2;
                                user_selection_count += 1; 
                                lever_sound.play();
                                time_of_last_decision_seconds = remaining_time;
                            }
                        },
                        KeyCode::Enter => {
                            remaining_time = 0.0;
                        },
                        _ => {}
                    }
                }
            }
        }

        clock_sound.stop();
        horn_sound.stop();
    
        disable_raw_mode().unwrap();
        stdout.execute(cursor::MoveToNextLine(1)).unwrap();

        DilemmaReport { 
            name: self.name.clone(),
            selected_option : self.options[user_input - 1].clone(),
            final_selection : user_input,
            user_selection_count : user_selection_count,
            time_of_last_decision_seconds : time_of_last_decision_seconds
        }
    }

    pub async fn play(&self, index : usize) -> DilemmaReport {    

        let reset = "\x1b[0m";
        let underline = "\x1b[4m";
        let cyan: &str = "\x1b[96m";

        println!("{}{}Dilemma {}: {}{}", underline, cyan, index, self.name, reset);
        println!("Description: {} \n", self.description);

        for (index, option) in &mut self.options.iter().enumerate() {
            option.describe(index);
        }

        let report = self.present_lever().await;

        report.display();

        report
    }
}

