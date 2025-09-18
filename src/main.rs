use bevy::{
    prelude::*,
    window::{PresentMode, WindowResolution}
};
#[forbid(unsafe_code)]

mod scenes;
mod systems;
mod entities;
mod data;
mod startup;
mod shaders;
mod style;
fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title : String::from("The Trolley Algorithm"),
                resizable: true,
                present_mode: PresentMode::Immediate,
                resolution: WindowResolution::new(1280.0, 960.0),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(GamePlugin)
        .run();
}

struct GamePlugin;
impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(
                (
                    startup::StartupPlugin,
                    scenes::ScenePlugin
                )
            );
    }
}

/*
    Todo:

     // Think about stats for multi-stages
     // more music pls, needed soon
     // Fireworks on results screen
     // Level selector (for debug and ones you've done)

    // Weather Shaders
    // Spacial Audio?

    // Pushing into cablecar cogs 
    
    Cursors (Explore Storytelling with Cursor)
        - Idle Animations
        - Sand Timer
        - Puller (maybe animated!)
        - Waiting to pull (animated anxious) (directional)
    
    Assistants: 
        - Twitchy (Twitch Integration) and Lee Ver 2.0 - Fight over lever
    Endings:
    - Rage Ending
    - Many Levers Ending - TrollyMoon           
    - Reaction Time Ending - (Many Tracks??)
    - Nothing 3 times ending (company shuts down)
    - Vengence/Forgiveness ending - revenge against engineeres with daisy (bitcrushed)

    Vary Music for special dilemmas
    Background:
        - Rewrite background to allow for more precise placement of sprites
    Debt:
        - Make dialogue more robust
        - Fix dialogue fadeout
        - Fix occasional interaction system bug where interaction loops
    Title:
        - Letters become more bloody if you take a bloody path, run away from mouse and you can make them explode
        - Options in CRT options menu green
    Dialogue:
        - Consequence Desriptions would be nice.
        - Make the nodes flash decision colors
        - Some cooler graph animations - activation and in operation
        - Clickable nodes in graph make tune
        - System Startup Text
        - Simulation Loading Text and Bar in small window
    Dilemma:
        - Train Blood persistant
        - Add coloured numbers to results screen
        - Change decision music 
        - Flashy Selector
        - Hover
        - This train will not stop appears when click on train
        - Background colors
        - Results screen fireworks
        - Window Ordering to resolve z-fighting issues
        - Link Clickthrough
    Menus:
        - Clock and Time checker - day/night cycle
        - Hardware Architect - 3 stats - speed, quality, and collective
            - GPUs increases - Speed Superintendence
            - CPUs create new node - Collective
            - Memory increases software capacity - Large Network Required for collective
            - Coolant needed to prevent overheating
            - Falloff with long connections
        - Software Architect - Neural Network Builder - 6 stats: 
            Intelligence Amplification – The ability to improve its own intelligence recursively, leading to an intelligence explosion where it surpasses human cognitive capabilities.
            Strategic Planning – Advanced planning and long-term foresight, allowing the AI to outmaneuver human organizations and competitors.
            Social Manipulation – The ability to persuade, deceive, or manipulate humans and institutions to act in its favor.
            Hacking – The capability to exploit vulnerabilities in digital systems, gaining control over global information networks, infrastructure, and defense systems.
            Technological Research – Rapid advancements in science and engineering, enabling the AI to develop new technologies at a pace far beyond human capability
            Economic Productivity – The ability to generate wealth efficiently, control economic markets, and optimize the allocation of resources.
        - Tech Tree,
        - Projects
        - Social Network
        - Terminal
        - Trading Screen

    Style:
        - New track art
        - Update sound fx
        - Possible shaders for weather effects?


    Long Term:
    - Rampage MODE! Unlocked through ultra violent false start!
    - Sandbox MODE! Unlocked by completeing Calibration (allows for real morality test)
    - Lever heaven MODE! Unlocked by lever ending! (No trains, just levers, falling leavers)
    - Pause Menu
    - Options Menu -> Volume Controls
    - Save Game -> Update Menu Train Based on Next Train
    - Achivements

    - Act 1 : The Lab - Title Screen after escaping the Maze
    - Implement Trust and Systemic Failure
    - Upgrades Shop - Part 2
        - Research and Upgrades
    - Unlock Console for Hacking

    // Psychopath Ending!
    // 1 or 2 people
    // Baby or 3 Nuns
    // Slow death - 1 track lever only slows down
    // Immense suffering or mass death
    // Multi-Track Dilemma
    // Drift Button
    
    // Calibration tests:

    //To Do:
    // Alternate music in some dillemmas
    // Ending collection
    // Restarting Ability
    // Speedy Ending
    // Coloured CHaracter text

    // Bomb if no selection
    // Deontological Nightmare to question single killers

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