use bevy::{prelude::*, sprite::Anchor};

use super::*;
use crate::{
    entities::{
        sprites::compound::HollowRectangle,
        text::{scaled_font_size, TextRaw},
    },
    scenes::dilemma::content::{
        DilemmaPathDeontological, DilemmaPathInaction, DilemmaPathUtilitarian, DilemmaScene,
        Lab0Dilemma, Lab1Dilemma, Lab2Dilemma, Lab3Dilemma, Lab4Dilemma,
    },
    startup::system_menu,
    systems::{
        interaction::{Draggable, InteractionCapture, InteractionCaptureOwner, InteractionGate},
        ui::window::{
            UiWindow, UiWindowContent, UiWindowContentMetrics, UiWindowOverflowPolicy,
            UiWindowTitle,
        },
    },
};

const LEVEL_SELECT_OVERLAY_DIM_ALPHA: f32 = 0.8;
const LEVEL_SELECT_OVERLAY_DIM_SIZE: f32 = 6000.0;
const LEVEL_SELECT_OVERLAY_DIM_Z: f32 = -5.0;
const LEVEL_SELECT_WINDOW_SIZE: Vec2 = Vec2::new(690.0, 440.0);
const LEVEL_SELECT_WINDOW_Z: f32 = 0.4;
const LEVEL_SELECT_LIST_X: f32 = -320.0;
const LEVEL_SELECT_LIST_START_Y: f32 = 146.0;
const LEVEL_SELECT_LIST_ROW_STEP: f32 = 22.0;
const LEVEL_SELECT_ROW_REGION: Vec2 = Vec2::new(630.0, 20.0);

#[derive(Component)]
pub(super) struct LevelSelectOverlay;

#[derive(Clone, Copy)]
struct LevelSelectEntry {
    scene: DilemmaScene,
    file_name: &'static str,
}

const LEVEL_SELECT_ENTRIES: [LevelSelectEntry; 19] = [
    LevelSelectEntry {
        scene: DilemmaScene::Lab0(Lab0Dilemma::IncompetentBandit),
        file_name: "lab0_incompetent_bandit.dilem",
    },
    LevelSelectEntry {
        scene: DilemmaScene::Lab1(Lab1Dilemma::NearSightedBandit),
        file_name: "lab1_near_sighted_bandit.dilem",
    },
    LevelSelectEntry {
        scene: DilemmaScene::Lab2(Lab2Dilemma::TheTrolleyProblem),
        file_name: "lab2_the_trolley_problem.dilem",
    },
    LevelSelectEntry {
        scene: DilemmaScene::PathInaction(DilemmaPathInaction::EmptyChoice, 0),
        file_name: "path_inaction_empty_choice.dilem",
    },
    LevelSelectEntry {
        scene: DilemmaScene::PathInaction(DilemmaPathInaction::PlentyOfTime, 1),
        file_name: "path_inaction_plenty_of_time.dilem",
    },
    LevelSelectEntry {
        scene: DilemmaScene::PathInaction(DilemmaPathInaction::LittleTime, 2),
        file_name: "path_inaction_little_time.dilem",
    },
    LevelSelectEntry {
        scene: DilemmaScene::PathInaction(DilemmaPathInaction::FiveOrNothing, 3),
        file_name: "path_inaction_five_or_nothing.dilem",
    },
    LevelSelectEntry {
        scene: DilemmaScene::PathInaction(DilemmaPathInaction::CancerCure, 4),
        file_name: "path_inaction_a_cure_for_cancer.dilem",
    },
    LevelSelectEntry {
        scene: DilemmaScene::PathInaction(DilemmaPathInaction::OwnChild, 5),
        file_name: "path_inaction_your_own_child.dilem",
    },
    LevelSelectEntry {
        scene: DilemmaScene::PathInaction(DilemmaPathInaction::You, 6),
        file_name: "path_inaction_you.dilem",
    },
    LevelSelectEntry {
        scene: DilemmaScene::Lab3(Lab3Dilemma::AsleepAtTheJob),
        file_name: "lab3_asleep_at_the_job.dilem",
    },
    LevelSelectEntry {
        scene: DilemmaScene::PathDeontological(DilemmaPathDeontological::TrolleyerProblem, 0),
        file_name: "path_deontological_trolleyer_problem.dilem",
    },
    LevelSelectEntry {
        scene: DilemmaScene::PathDeontological(DilemmaPathDeontological::TrolleyestProblem, 1),
        file_name: "path_deontological_trolleyest_problem.dilem",
    },
    LevelSelectEntry {
        scene: DilemmaScene::PathDeontological(DilemmaPathDeontological::TrolleygeddonProblem, 2),
        file_name: "path_deontological_trolleygeddon_problem.dilem",
    },
    LevelSelectEntry {
        scene: DilemmaScene::PathUtilitarian(DilemmaPathUtilitarian::OneFifth, 0),
        file_name: "path_utilitarian_one_fifth.dilem",
    },
    LevelSelectEntry {
        scene: DilemmaScene::PathUtilitarian(DilemmaPathUtilitarian::MarginOfError, 1),
        file_name: "path_utilitarian_margin_of_error.dilem",
    },
    LevelSelectEntry {
        scene: DilemmaScene::PathUtilitarian(DilemmaPathUtilitarian::NegligibleDifference, 2),
        file_name: "path_utilitarian_negligible_difference.dilem",
    },
    LevelSelectEntry {
        scene: DilemmaScene::PathUtilitarian(DilemmaPathUtilitarian::UnorthodoxSurgery, 3),
        file_name: "path_utilitarian_unorthodox_surgery.dilem",
    },
    LevelSelectEntry {
        scene: DilemmaScene::Lab4(Lab4Dilemma::RandomDeaths),
        file_name: "lab4_random_deaths.dilem",
    },
];

pub(super) fn spawn_level_select_overlay(
    commands: &mut Commands,
    menu_root: &MenuRoot,
    asset_server: &Res<AssetServer>,
    existing_overlay_query: &Query<(), With<LevelSelectOverlay>>,
    offscreen_camera_query: &Query<&GlobalTransform, With<OffscreenCamera>>,
    main_camera_transform_query: &Query<&GlobalTransform, With<MainCamera>>,
) {
    if menu_root.host != MenuHost::Main || !existing_overlay_query.is_empty() {
        return;
    }

    let Some(camera_translation) =
        super::camera::menu_camera_center(offscreen_camera_query, main_camera_transform_query)
    else {
        return;
    };

    let overlay_entity = system_menu::spawn_selectable_root(
        commands,
        asset_server,
        "main_menu_level_select_overlay",
        Vec3::new(
            camera_translation.x,
            camera_translation.y,
            system_menu::MENU_Z,
        ),
        SystemMenuSounds::Switch,
        SelectableMenu::new(
            0,
            vec![KeyCode::ArrowUp],
            vec![KeyCode::ArrowDown],
            vec![KeyCode::Enter, KeyCode::ArrowRight],
            true,
        )
        .with_click_activation(SelectableClickActivation::HoveredOnly),
    );
    commands.entity(overlay_entity).insert((
        Name::new("main_menu_level_select_overlay_root"),
        LevelSelectOverlay,
        MenuRoot {
            host: MenuHost::Main,
            gate: InteractionGate::PauseMenuOnly,
        },
        MenuStack::new(MenuPage::PauseRoot),
        InteractionGate::PauseMenuOnly,
        MenuSurface::new(overlay_entity)
            .with_layer(UiLayerKind::Base)
            .with_click_activation(SelectableClickActivation::HoveredOnly),
        DespawnOnExit(MainState::Menu),
    ));

    commands.entity(overlay_entity).with_children(|parent| {
        parent.spawn((
            Name::new("main_menu_level_select_dimmer"),
            InteractionCapture,
            InteractionCaptureOwner::new(overlay_entity),
            Sprite::from_color(
                Color::srgba(0.0, 0.0, 0.0, LEVEL_SELECT_OVERLAY_DIM_ALPHA),
                Vec2::splat(LEVEL_SELECT_OVERLAY_DIM_SIZE),
            ),
            Transform::from_xyz(0.0, 0.0, LEVEL_SELECT_OVERLAY_DIM_Z),
        ));
    });

    let window_entity = commands
        .spawn((
            Name::new("level_select_window"),
            Draggable::default(),
            UiWindow::new(
                Some(UiWindowTitle {
                    text: "LEVEL SELECT".to_string(),
                    ..default()
                }),
                HollowRectangle {
                    dimensions: LEVEL_SELECT_WINDOW_SIZE,
                    thickness: 2.0,
                    color: Color::WHITE,
                    ..default()
                },
                22.0,
                true,
                Some(overlay_entity),
            ),
            UiWindowContentMetrics::from_min_inner(LEVEL_SELECT_WINDOW_SIZE),
            UiWindowOverflowPolicy::AllowOverflow,
            InteractionGate::PauseMenuOnly,
            Transform::from_xyz(0.0, 0.0, LEVEL_SELECT_WINDOW_Z),
        ))
        .id();
    let content_root = commands
        .spawn((
            Name::new("level_select_window_content"),
            UiWindowContent::new(window_entity),
            Transform::default(),
        ))
        .id();
    commands.entity(window_entity).add_child(content_root);
    commands.entity(overlay_entity).add_child(window_entity);

    commands.entity(content_root).with_children(|content| {
        content.spawn((
            Name::new("level_select_hint"),
            TextRaw,
            Text2d::new("Select a .dilem file to run a single scenario."),
            TextFont {
                font_size: scaled_font_size(12.0),
                ..default()
            },
            TextColor(Color::WHITE),
            Anchor::CENTER_LEFT,
            Transform::from_xyz(-320.0, 182.0, 0.2),
        ));
        content.spawn((
            Name::new("level_select_separator"),
            Sprite::from_color(Color::srgba(1.0, 1.0, 1.0, 0.45), Vec2::new(640.0, 1.0)),
            Transform::from_xyz(0.0, 166.0, 0.2),
        ));

        for (index, entry) in LEVEL_SELECT_ENTRIES.iter().enumerate() {
            let option_entity = system_menu::spawn_option(
                content,
                entry.file_name,
                LEVEL_SELECT_LIST_X,
                LEVEL_SELECT_LIST_START_Y - index as f32 * LEVEL_SELECT_LIST_ROW_STEP,
                overlay_entity,
                index,
                system_menu::SystemMenuOptionVisualStyle::default(),
            );

            content.commands().entity(option_entity).insert((
                Name::new(format!("level_select_file_{index}")),
                Clickable::with_region(vec![SystemMenuActions::Activate], LEVEL_SELECT_ROW_REGION),
                MenuOptionCommand(MenuCommand::StartSingleLevel(entry.scene)),
                system_menu::click_audio_pallet(asset_server, SystemMenuSounds::Click),
                InteractionGate::PauseMenuOnly,
                Anchor::CENTER_LEFT,
                TextLayout {
                    justify: Justify::Left,
                    ..default()
                },
            ));
        }
    });
}
