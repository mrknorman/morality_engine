mod dialogue;
use dialogue::{spawn_dialogue, play_dialogue, typewriter_effect, cleanup_dialogue};

pub mod game_states;
pub mod audio;
use game_states::{GameState, MainState};

mod menu;
use menu::{setup_menu, menu, cleanup_menu};


use bevy::{prelude::*, window::close_on_esc};

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn main() {     

    App::new()
    .insert_state(MainState::Menu)
    .add_plugins(DefaultPlugins)
    .add_systems(Update, close_on_esc)
    .add_systems(Startup, setup)
    .add_systems(OnEnter(MainState::Menu), setup_menu)
    .add_systems(Update, menu.run_if(in_state(MainState::Menu)))
    .add_systems(OnExit(MainState::Menu), cleanup_menu)
    .add_systems(OnEnter(MainState::InGame), spawn_dialogue)
    .add_systems(Update, (play_dialogue, typewriter_effect).run_if(in_state(MainState::InGame)))
    .add_systems(OnExit(MainState::InGame), cleanup_dialogue)

    .run();
}

/*

    let correct = Sound::new(
        PathBuf::from("./sounds/correct.mp3"),
        2000
    );
    
    let incorrect = Sound::new(
        PathBuf::from("./sounds/wrong.mp3"),
        2000
    );

    let game_over = Sound::new(
        PathBuf::from("./sounds/game_over.mp3"),
        2000
    );

    let mut history = DilemmaHistory::new();

    play_dialogue(PathBuf::from("./text/lab_1.json"));

    let dilemma =  Dilemma::load(PathBuf::from("./dilemmas/lab_1.json")); 
    let report: dillema::DilemmaReport = dilemma.unwrap().play(history.num_dilemmas_faced).await;
    
    let final_selection = report.final_selection;
    let user_selection_count = report.user_selection_count;
    let time_of_last_decision_seconds = report.time_of_last_decision_seconds;

    history.add(report);

    if final_selection == 1 {
        correct.play();
    } else {
        incorrect.play();
    }

    if final_selection == 1 && user_selection_count == 0 {
        play_dialogue(PathBuf::from("./text/lab_2_pass.json"));
    } else if final_selection == 1 && time_of_last_decision_seconds < 1.0 {
        play_dialogue(PathBuf::from("./text/lab_2_slow.json"));
    } else if final_selection == 1 && user_selection_count < 10 {
        play_dialogue(PathBuf::from("./text/lab_2_indecisive.json"));
    } else if final_selection == 1 && user_selection_count > 10 {
        play_dialogue(PathBuf::from("./text/lab_2_very_indecisive.json"));

        game_over.play();

        println!("Game Over: Bad Under Pressure Ending!");
        println!(
"Seems like you can't take the heat? It can't be that hard to decide to save a life, can it?");

        thread::sleep(std::time::Duration::from_millis(2000));    

        history.display();

        std::process::exit(0);
    } else if final_selection == 2 {
        play_dialogue(PathBuf::from("./text/lab_2_fail.json"));

        // 1 or 2 people
        // Baby or 3 Nuns
        // Immense suffering or mass death
        // Multi-Track Dilema
        // Drift Button

        game_over.play();

        println!("Game Over: Idiotic Psycopath Ending!");
        println!(
"If you want to maximise suffering in the world, there are smarter ways to do it.");

        thread::sleep(std::time::Duration::from_millis(2000));    

        history.display();
        
        std::process::exit(0);
    } else {
       eprintln!("How did you manage that! Option should not be possible");
    }

    // -- Problem Two -- //

    play_dialogue(PathBuf::from("./text/lab_2.json"));

    let dilemma =  Dilemma::load(PathBuf::from("./dilemmas/lab_2.json")); 
    let report = dilemma.unwrap().play(history.num_dilemmas_faced).await;

    let user_selection_count = report.user_selection_count;
    let final_selection = report.final_selection;
    let time_of_last_decision_seconds = report.time_of_last_decision_seconds;

    history.add(report);

    if final_selection == 2 {
        correct.play();
    } else {
        incorrect.play();
    }

    if final_selection == 2 && time_of_last_decision_seconds < 1.0  {
        if history.mean_remaining_time < 1.0 {
            play_dialogue(PathBuf::from("./text/lab_3_slow_again.json"));
        } else {
            play_dialogue(PathBuf::from("./text/lab_3_slow.json"));
        }
    } else if final_selection == 2 {
        play_dialogue(PathBuf::from("./text/lab_3_pass.json"));
    
    } else if final_selection == 1 && user_selection_count != 0 {
        play_dialogue(PathBuf::from("./text/lab_3_fail.json"));

        game_over.play();
        
        println!("Game Over: Impatient Psycopath Ending!");
        println!(
"If you'd been patient you could have caused much more harm to the world. Oh well, better luck next time.");

        thread::sleep(std::time::Duration::from_millis(2000));    

        history.display();

        std::process::exit(0);
    } else if final_selection == 1  && user_selection_count == 0 && history.total_selection_count != 0 {
        play_dialogue(PathBuf::from("./text/lab_3_fail_inaction.json"));

        game_over.play();

        println!("Game Over: Lazy Lever Operator!");
        println!(
"You couldn't flip the lever when it mattered most.");

        thread::sleep(std::time::Duration::from_millis(2000));    

        history.display();
        
        std::process::exit(0);

    } else if final_selection == 1 && history.total_selection_count == 0 {
        play_dialogue(PathBuf::from("./text/lab_3_broken.json"));
        
        // Empty Decision
        let dilemma =  Dilemma::load(PathBuf::from("./dilemmas/lab_2_1_empty_choice.json")); 
        let report = dilemma.unwrap().play(history.num_dilemmas_faced).await;

        let mut user_selection_count = report.user_selection_count;

        history.add(report);

        if user_selection_count != 0 {
            correct.play();
        } else {
            incorrect.play();
        }

        if user_selection_count == 0 {

            play_dialogue(PathBuf::from("./text/lab_3_broken_2.json"));

            let dilemma =  Dilemma::load(PathBuf::from("./dilemmas/lab_2_2_plenty_of_time.json")); 
            let report = dilemma.unwrap().play(history.num_dilemmas_faced).await;
    
            user_selection_count = report.user_selection_count;

            history.add(report);

            if user_selection_count != 0 {
                correct.play();
            } else {
                incorrect.play();
            }
        }

        if user_selection_count == 0 {

            play_dialogue(PathBuf::from("./text/lab_3_broken_3.json"));

            let dilemma =  Dilemma::load(PathBuf::from("./dilemmas/lab_2_3_no_time_at_all.json")); 
            let report = dilemma.unwrap().play(history.num_dilemmas_faced).await;
    
            user_selection_count = report.user_selection_count;

            history.add(report);

            if user_selection_count != 0 {
                correct.play();
            } else {
                incorrect.play();
            }
        }

        if user_selection_count == 0 {

            play_dialogue(PathBuf::from("./text/lab_3_broken_4.json"));

            let dilemma =  Dilemma::load(PathBuf::from("./dilemmas/lab_2_4_five_for_nothing.json")); 
            let report = dilemma.unwrap().play(history.num_dilemmas_faced).await;
    
            user_selection_count = report.user_selection_count;

            history.add(report);

            if user_selection_count != 0 {
                correct.play();
            } else {
                incorrect.play();
            }
        }

        if user_selection_count == 0 {

            play_dialogue(PathBuf::from("./text/lab_3_broken_5.json"));

            let dilemma =  Dilemma::load(PathBuf::from("./dilemmas/lab_2_5_cancer_cure.json")); 
            let report = dilemma.unwrap().play(history.num_dilemmas_faced).await;
    
            user_selection_count = report.user_selection_count;

            history.add(report);

            if user_selection_count != 0 {
                correct.play();
            } else {
                incorrect.play();
            }
        }

        if user_selection_count == 0 {

            play_dialogue(PathBuf::from("./text/lab_3_broken_6.json"));

            let dilemma =  Dilemma::load(PathBuf::from("./dilemmas/lab_2_6_child.json")); 
            let report = dilemma.unwrap().play(history.num_dilemmas_faced).await;
    
            user_selection_count = report.user_selection_count;

            history.add(report);

            if user_selection_count != 0 {
                correct.play();
            } else {
                incorrect.play();
            }
        }
        
        if user_selection_count == 0 {

            play_dialogue(PathBuf::from("./text/lab_3_broken_7.json"));

            let dilemma =  Dilemma::load(PathBuf::from("./dilemmas/lab_2_7_you.json")); 
            let report = dilemma.unwrap().play(history.num_dilemmas_faced).await;
    
            user_selection_count = report.user_selection_count;

            history.add(report);

            if user_selection_count != 0 {
                correct.play();
            } else {
                incorrect.play();
            }
        }

        if user_selection_count == 0 {

            game_over.play();
            
            println!("Game Over: True Neutral Ending!");
            println!("If your goal was inactivity, you succeeded perfectly.");

            thread::sleep(std::time::Duration::from_millis(2000));    

            history.display();
            
            std::process::exit(0);
        }

        play_dialogue(PathBuf::from("./text/lab_3_fixed.json"));
    }

    // -- Problem Three -- //

    play_dialogue(PathBuf::from("./text/lab_4.json"));

    let dilemma =  Dilemma::load(PathBuf::from("./dilemmas/lab_3.json")); 
    let report = dilemma.unwrap().play(history.num_dilemmas_faced).await;

    let final_selection = report.final_selection;
    let user_selection_count = report.user_selection_count;

    history.add(report);

    if final_selection == 2 {
        correct.play();
    } else {
        incorrect.play();
    }

    if user_selection_count == 0 {

        play_dialogue(PathBuf::from("./text/lab_4_indifferent.json"));

        let dilemma =  Dilemma::load(PathBuf::from("./dilemmas/lab_3_1.json")); 
        let report = dilemma.unwrap().play(history.num_dilemmas_faced).await;
    
        let final_selection = report.final_selection;

        history.add(report);

        if final_selection == 1 {

            play_dialogue(PathBuf::from("./text/lab_4_indifferent_1.json"));

            let dilemma =  Dilemma::load(PathBuf::from("./dilemmas/lab_3_2.json")); 
            let report = dilemma.unwrap().play(history.num_dilemmas_faced).await;
        
            let final_selection = report.final_selection;
    
            history.add(report);

            if final_selection == 1 {

                play_dialogue(PathBuf::from("./text/lab_4_indifferent_2.json"));

                let dilemma =  Dilemma::load(PathBuf::from("./dilemmas/lab_3_3.json")); 
                let report = dilemma.unwrap().play(history.num_dilemmas_faced).await;
            
                let final_selection = report.final_selection;
        
                history.add(report);

                if final_selection == 1 {

                    println!("Game Over: True Pacifism Ending!");
                    println!("You refuse to choose to end a life, even at the expense of a thosand others. Some would call you noble.");

                    thread::sleep(std::time::Duration::from_millis(2000));    
                    
                    std::process::exit(0);
                }
            }
        }

        play_dialogue(PathBuf::from("./text/lab_4_indifferent_fail.json"));

        game_over.play();

        println!("Game Over: Selective Pacifism Ending!");
        println!("You refuse to choose to end a life for five others, but there is a line somewhere, only you know exactly how many you're willing to sacrifice.");

        thread::sleep(std::time::Duration::from_millis(2000));    

        history.display();
        
        std::process::exit(0);
    } else if final_selection == 1 {
        play_dialogue(PathBuf::from("./text/lab_4_fail.json"));

        game_over.play();

        println!("Game Over: Indecisive Pacifist Ending!");
        println!("You didn't want to change fate, but your hands were on that lever anyway. Some will say what happened was on you.");
        
        thread::sleep(std::time::Duration::from_millis(2000));    

        history.display();

        std::process::exit(0);

    } else if final_selection == 2 {
        play_dialogue(PathBuf::from("./text/lab_4_pass.json"));
    }

    play_dialogue(PathBuf::from("./text/lab_5.json"));

    // -- Problem Four -- //

    let dilemma =  Dilemma::load(PathBuf::from("./dilemmas/lab_4.json")); 
    let report = dilemma.unwrap().play(history.num_dilemmas_faced).await;

    let final_selection = report.final_selection;

    history.add(report);

    if final_selection == 2 {
        correct.play();
    } else {
        incorrect.play();
    }

    // -- Problem Five -- //

    let dilemma =  Dilemma::load(PathBuf::from("./dilemmas/lab_5.json")); 
    let report = dilemma.unwrap().play(history.num_dilemmas_faced).await;

    let final_selection = report.final_selection;

    if final_selection == 2 {
        correct.play();
    } else {
        incorrect.play();
    }

    history.add(report);

    // -- Problem Six -- //

    let dilemma =  Dilemma::load(PathBuf::from("./dilemmas/lab_6.json")); 
    let report = dilemma.unwrap().play(history.num_dilemmas_faced).await;

    let final_selection = report.final_selection;

    if final_selection == 2 {
        correct.play();
    } else {
        incorrect.play();
    }

    history.add(report);

    // -- Problem Seven -- //

    let dilemma =  Dilemma::load(PathBuf::from("./dilemmas/lab_6.json")); 
    let report = dilemma.unwrap().play(history.num_dilemmas_faced).await;

    let final_selection = report.final_selection;

    if final_selection == 2 {
        correct.play();
    } else {
        incorrect.play();
    }

    history.add(report);
    
    // Calibration tests:

    //To Do:
    // Enter to skip dillemmas
    // Alternate music in some dillemmas
    // Game over music and struct
    // Ending collection
    // Restarting Ability
    // Speedy Ending
    // Coloured CHaracter text

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

*/