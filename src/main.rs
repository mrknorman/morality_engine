use bevy::{
    asset::RenderAssetUsages, color::palettes::css::BLACK, core_pipeline::{
        bloom::Bloom,
        tonemapping::Tonemapping,
    }, prelude::*, render::{camera::RenderTarget, render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages}, view::RenderLayers}, sprite::Material2dPlugin, window::{PresentMode, PrimaryWindow, WindowResized}
};

use dilemma::lever::Lever;
use rand::SeedableRng;
use rand_pcg::Pcg64Mcg;
use shaders::ScanLinesMaterial;
use stats::DilemmaStats;

#[forbid(unsafe_code)]

mod audio;
mod background;
mod dialogue;
mod dilemma;
mod game_states;
mod loading;
mod menu;
mod person;
mod train;
mod graph;
mod shortcuts;
mod motion;
mod text;
mod track;
mod interaction;
mod timing;
mod character;
mod io;
mod sprites;
mod common_ui;
mod shaders;
mod colors;
mod physics;
mod ascii_fonts;
mod inheritance;
mod time;
mod stats;

use crate::{
    game_states::{
        GameState, 
        MainState, 
        DilemmaPhase
    },
    shaders::PulsingMaterial,
    time::DilationPLugin,
    stats::GameStats
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resizable: true,
                present_mode: PresentMode::Immediate,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(GamePlugin)
        .add_plugins(Material2dPlugin::<PulsingMaterial>::default())
        .add_plugins(Material2dPlugin::<ScanLinesMaterial>::default())
        .run();
}

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum StartUpOrder {
    SetUpRenderTarget,
    ScanLineSetup,
    PostProcess
}


struct GamePlugin;
impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(GlobalRng::default())
            .insert_resource(Lever::default())
            .insert_resource(GameStats::default())
            .insert_resource(DilemmaStats::default())
            .insert_resource(GameStats::default())
            .add_systems(Startup, (
                setup_render_target.in_set(StartUpOrder::SetUpRenderTarget),
                ScanLinesMaterial::setup.in_set(StartUpOrder::ScanLineSetup).after(StartUpOrder::SetUpRenderTarget),
                setup_cameras.after(StartUpOrder::ScanLineSetup)
            ))
            .add_systems(Update, (shortcuts::close_on_esc, ScanLinesMaterial::update, ScanLinesMaterial::update_scan_mesh, update_render_target_size))
            .add_systems(Update, ScanLinesMaterial::update)
            .init_state::<MainState>()
            .add_sub_state::<GameState>()
            .add_sub_state::<DilemmaPhase>()
            .enable_state_scoped_entities::<MainState>()
            .enable_state_scoped_entities::<GameState>()
            .enable_state_scoped_entities::<DilemmaPhase>()
            .add_plugins(common_ui::CommonUIPlugin)
            .add_plugins(menu::MenuScenePlugin)
            .add_plugins(loading::LoadingScenePlugin)
            .add_plugins(dialogue::DialogueScenePlugin)
            .add_plugins(dilemma::DilemmaScenePlugin)
            .add_plugins(motion::MotionPlugin)
            .add_plugins(colors::ColorsPlugin)
            .add_plugins(inheritance::InheritancePlugin)
            .add_plugins(DilationPLugin);
            
            /*
            use bevy::diagnostic::{LogDiagnosticsPlugin, FrameTimeDiagnosticsPlugin};
            app
            .add_plugins(FrameTimeDiagnosticsPlugin)
            .add_plugins(LogDiagnosticsPlugin::default())
            ;
            */
            
    }
}

#[derive(Resource)]
struct GlobalRng(pub Pcg64Mcg);
 
impl Default for GlobalRng {
    fn default() -> Self {
        GlobalRng(Pcg64Mcg::seed_from_u64(12345))
    }
}

#[derive(Component)]
pub struct MainCamera;

fn setup_cameras(
        mut commands: Commands,         
        mut clear_color: ResMut<ClearColor>,
        render_target: Res<RenderTargetHandle>
    ) {

    clear_color.0 = BLACK.into();

    // Off-screen camera: render game geometry to the off-screen texture.
    commands.spawn((
        Camera2d,
        OffscreenCamera,
        RenderLayers::layer(0),
        Camera {
            hdr: true,
            target: RenderTarget::Image(render_target.0.clone()),
            ..default()
        },
        Tonemapping::TonyMcMapface,
        Bloom::default(),
    ));

    // Main (window) camera: renders only the fullscreen quad (post-process) to the window.
    commands.spawn((
        Camera2d,
        MainCamera,
        RenderLayers::layer(1),
        Camera {
            hdr: true,
            ..default()
        },
        Tonemapping::TonyMcMapface,
        Bloom::default(),
    ));
}


#[derive(Component)]
pub struct OffscreenCamera;

fn update_render_target_size(
    windows: Query<&Window, (With<PrimaryWindow>, Changed<Window>)>,
    mut images: ResMut<Assets<Image>>,
    mut resize_reader: EventReader<WindowResized>,
    render_target: ResMut<RenderTargetHandle>,
) {
    for _ in resize_reader.read() {
        if let Ok(window) = windows.get_single() {
            let new_width = window.resolution.width() as u32;
            let new_height = window.resolution.height() as u32;
            // Get a mutable reference to the current image, if it exists:
            if let Some(image) = images.get_mut(&render_target.0) {
                // If the size doesn’t match, recreate the image
                if image.texture_descriptor.size.width != new_width ||
                image.texture_descriptor.size.height != new_height
                {
                    let new_size = Extent3d {
                        width: new_width,
                        height: new_height,
                        depth_or_array_layers: 1,
                    };
                    let clear_color = [0, 0, 0, 255]; // opaque black
                    // Create a new image filled with the clear color.
                    let mut new_image = Image::new_fill(
                        new_size,
                        TextureDimension::D2,
                        &clear_color,
                        TextureFormat::bevy_default(),
                        RenderAssetUsages::default(), // or use your previous flag combination
                    );
                    // Ensure it has the correct usage flags.
                    new_image.texture_descriptor.usage = TextureUsages::TEXTURE_BINDING
                        | TextureUsages::COPY_DST
                        | TextureUsages::RENDER_ATTACHMENT;
                    // Replace the old image with the new one.
                    *image = new_image;
                }
            }
        }
    }
}
#[derive(Resource)]
pub struct RenderTargetHandle(pub Handle<Image>);

impl Default for RenderTargetHandle {
    fn default() -> Self {
        RenderTargetHandle(Handle::default())
    }
}




fn setup_render_target(
    mut commands: Commands,
    windows: Query<&Window>,
    mut images: ResMut<Assets<Image>>,
) {
    let window = windows.single();
    let width = window.resolution.width() as u32;
    let height = window.resolution.height() as u32;
    let size = Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
    };

    // Opaque black clear color
    let clear_color = [0, 0, 0, 255]; 

    let mut image = Image::new_fill(
        size,
        TextureDimension::D2,
        &clear_color,
        TextureFormat::bevy_default(),
        RenderAssetUsages::default(),
    );

    image.texture_descriptor.usage =
        TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST | TextureUsages::RENDER_ATTACHMENT;

    let image_handle = images.add(image);
    commands.insert_resource(RenderTargetHandle(image_handle));
}


//

/*
    Todo:
    
    Background:
        - Rewrite background to allow for more precise placement of sprites
    Debt:
        - Make dialogue more robust
        - Fix dialogue fadeout
        - Fix occasional interaction system bug where interaction loops
        - Make all JSONs included rather than loaded
    Title:
        - IMPORTANT - Change way Components are added to Ascii letters
        - Letters become more bloody if you take a bloody path, run away from mouse and you can make them explode
    Dialogue:
        - Number waterfall background, reacts to mouse movement
        - Make the nodes flash decision colors
        - Some cooler graph animations - activation and in operation
        - Try dialogue narration
        - Clickable nodes in graph make tune
        - System Startup Text
        - Simulation Loading Text and Bar in small window
    Dilemma:
        - Add close button to windows
        - Add coloured better formatted numbers to results screen
        - Change decision music 
        - Physics bodypart destruction
        - Flashy Selector
        - Hover
        - This train will not stop appears when click on train
        - Background colors
        - Results screen fireworks
        - Window Ordering to resolve z-fighting issues
        - Link Clickthrough

    Style:
        - New track art
        - Update sound fx


    Long Term:
    - Rampage MODE! Unlocked through ultra violent false start!
    - Sandbox MODE! Unlocked by completeing Calibration (allows for real morality test)
    - Pause Menu
    - Options Menu -> Volume Controls
    - Save Game -> Update Menu Train Based on Next Train
    - Achivements

    - Act 1 : The Lab - Title Screen after escaping the Maze
    - Implement Trust and Systemic Failure
    - Upgrades Shop - Part 2
        - Research and Upgrades
    - Unlock Console for Hacking

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

    // Bomb if no selection
    // Deontological Nightmare to question single killers
    // 4 vs 5 to question utilitarians

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