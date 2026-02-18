use bevy::{prelude::*, sprite::Anchor};

use crate::{
    data::states::{MainState, PauseState},
    entities::text::{scaled_font_size, Cell, Column, Row, Table, TextContent},
    startup::system_menu::{SystemMenuLayout, PANEL_WIDTH},
    systems::{colors::SYSTEM_MENU_COLOR, interaction::InteractionGate},
};

pub(super) const PAUSE_MENU_TITLE: &str = "PAUSED";
pub(super) const PAUSE_MENU_HINT: &str = "[ESCAPE TO RESUME]\n[ARROW UP/ARROW DOWN + ENTER]";
pub(super) const PAUSE_MENU_RESUME_TEXT: &str = "RESUME [⌫]";
pub(super) const PAUSE_MENU_OPTIONS_TEXT: &str = "OPTIONS [o]";
pub(super) const PAUSE_MENU_EXIT_TO_MENU_TEXT: &str = "EXIT TO MENU [m]";
pub(super) const PAUSE_MENU_EXIT_TO_DESKTOP_TEXT: &str = "EXIT TO DESKTOP [d]";

pub(super) const DEBUG_MENU_TITLE: &str = "DEBUG OVERLAY MENU";
pub(super) const DEBUG_MENU_HINT: &str = "[ARROW UP/ARROW DOWN + ENTER]\n[CLICK ALSO WORKS]";
pub(super) const DEBUG_MENU_RESUME_TEXT: &str = "[CLOSE DEBUG MENU]";
pub(super) const DEBUG_MENU_MAIN_MENU_TEXT: &str = "[RETURN TO MAIN MENU]";

pub(super) const OPTIONS_MENU_TITLE: &str = "OPTIONS";
pub(super) const OPTIONS_MENU_HINT: &str = "[ARROW UP/ARROW DOWN + ENTER]";
pub(super) const OPTIONS_MENU_VIDEO_TEXT: &str = "VIDEO";
pub(super) const OPTIONS_MENU_AUDIO_TEXT: &str = "AUDIO";
pub(super) const OPTIONS_MENU_CONTROLS_TEXT: &str = "CONTROLS";
pub(super) const OPTIONS_MENU_BACK_TEXT: &str = "BACK [⌫]";
pub(super) const VIDEO_MENU_TITLE: &str = "VIDEO";
pub(super) const VIDEO_MENU_HINT: &str = "[ARROW UP/ARROW DOWN + ENTER]";
pub(super) const VIDEO_MENU_DISPLAY_MODE_TEXT: &str = "DISPLAY MODE [f]";
pub(super) const VIDEO_MENU_RESOLUTION_TEXT: &str = "RESOLUTION [r]";
pub(super) const VIDEO_MENU_VSYNC_TEXT: &str = "VSYNC [v]";
pub(super) const VIDEO_MENU_APPLY_TEXT: &str = "APPLY [a]";
pub(super) const VIDEO_MENU_RESET_TEXT: &str = "RESET TO DEFAULTS [z]";
pub(super) const VIDEO_MENU_BACK_TEXT: &str = "BACK [⌫]";
pub(super) const VIDEO_MENU_VALUE_PLACEHOLDER: &str = "---";
pub(super) const VIDEO_TABLE_LABEL_COLUMN_WIDTH: f32 = 390.0;
pub(super) const VIDEO_TABLE_VALUE_COLUMN_WIDTH: f32 = 390.0;
pub(super) const VIDEO_TABLE_TOTAL_WIDTH: f32 =
    VIDEO_TABLE_LABEL_COLUMN_WIDTH + VIDEO_TABLE_VALUE_COLUMN_WIDTH;
pub(super) const VIDEO_TABLE_ROW_HEIGHT: f32 = 40.0;
pub(super) const VIDEO_TABLE_TEXT_SIZE: f32 = scaled_font_size(14.0);
pub(super) const VIDEO_TABLE_TEXT_SELECTED_SIZE: f32 = scaled_font_size(21.0);
pub(super) const VIDEO_TABLE_TEXT_Z: f32 = 0.3;
pub(super) const VIDEO_TABLE_X: f32 = -VIDEO_TABLE_TOTAL_WIDTH * 0.5;
pub(super) const VIDEO_TABLE_Y: f32 = 80.0;
pub(super) const VIDEO_OPTION_SELECTOR_X: f32 = 0.0;
pub(super) const VIDEO_OPTION_REGION_WIDTH: f32 = VIDEO_TABLE_TOTAL_WIDTH + 80.0;
pub(super) const VIDEO_OPTION_REGION_HEIGHT: f32 = 38.0;
// Center of the value column relative to the option entity (at x=0):
// VIDEO_TABLE_X + LABEL_WIDTH + VALUE_WIDTH/2 = -390 + 390 + 195 = 195
pub(super) const VIDEO_VALUE_COLUMN_CENTER_X: f32 =
    VIDEO_TABLE_X + VIDEO_TABLE_LABEL_COLUMN_WIDTH + VIDEO_TABLE_VALUE_COLUMN_WIDTH * 0.5;
pub(super) const VIDEO_NAME_COLUMN_CENTER_X: f32 = VIDEO_TABLE_X + VIDEO_TABLE_LABEL_COLUMN_WIDTH * 0.5;
// Keep the name highlight above table cell backgrounds (z≈0.95 world)
// while staying below text (text z is boosted separately).
pub(super) const VIDEO_NAME_HIGHLIGHT_Z: f32 = 0.04;
pub(super) const VIDEO_NAME_HIGHLIGHT_WIDTH: f32 = VIDEO_TABLE_LABEL_COLUMN_WIDTH - 6.0;
pub(super) const VIDEO_NAME_HIGHLIGHT_HEIGHT: f32 = VIDEO_OPTION_REGION_HEIGHT - 2.0;
pub(super) const VIDEO_RESOLUTION_DROPDOWN_WIDTH: f32 = VIDEO_TABLE_VALUE_COLUMN_WIDTH;
pub(super) const VIDEO_RESOLUTION_DROPDOWN_ROW_HEIGHT: f32 = VIDEO_TABLE_ROW_HEIGHT;
pub(super) const VIDEO_RESOLUTION_DROPDOWN_Z: f32 = 1.45;
pub(super) const VIDEO_RESOLUTION_DROPDOWN_BORDER_Z: f32 = 0.02;
pub(super) const VIDEO_RESOLUTION_DROPDOWN_ITEM_Z: f32 = 0.04;
pub(super) const VIDEO_RESOLUTION_DROPDOWN_ARROW_Z: f32 = 0.2;
pub(super) const VIDEO_RESOLUTION_DROPDOWN_ARROW_WIDTH: f32 = 16.0;
pub(super) const VIDEO_RESOLUTION_DROPDOWN_ARROW_HEIGHT: f32 = 20.0;
pub(super) const VIDEO_RESOLUTION_DROPDOWN_ARROW_INSET: f32 = 18.0;
pub(super) const VIDEO_RESOLUTION_DROPDOWN_ARROW_SPREAD: f32 =
    VIDEO_RESOLUTION_DROPDOWN_WIDTH * 0.5 - VIDEO_RESOLUTION_DROPDOWN_ARROW_INSET;
pub(super) const VIDEO_RESOLUTION_OPTION_INDEX: usize = 1;
pub(super) const VIDEO_MODAL_PANEL_SIZE: Vec2 = Vec2::new(540.0, 230.0);
pub(super) const VIDEO_MODAL_PANEL_Z: f32 = 2.0;
pub(super) const VIDEO_MODAL_BORDER_Z: f32 = 2.1;
pub(super) const VIDEO_MODAL_TEXT_Z: f32 = 2.2;
pub(super) const VIDEO_MODAL_OPTIONS_Y: f32 = -56.0;
pub(super) const VIDEO_MODAL_OPTIONS_SPREAD_X: f32 = 120.0;
pub(super) const VIDEO_MODAL_OPTION_REGION: Vec2 = Vec2::new(230.0, 38.0);
pub(super) const VIDEO_MODAL_OPTION_INDICATOR_X: f32 = 82.0;
pub(super) const PAUSE_MENU_MUSIC_PATH: &str = "./audio/music/suspended_systems.ogg";

pub(super) const RESOLUTIONS: [(f32, f32); 5] = [
    (1280.0, 720.0),
    (1600.0, 900.0),
    (1920.0, 1080.0),
    (2560.0, 1440.0),
    (3840.0, 2160.0),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MenuHost {
    Pause,
    Debug,
    Main,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MenuPage {
    PauseRoot,
    DebugRoot,
    Options,
    Video,
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct MenuRoot {
    pub host: MenuHost,
    pub gate: InteractionGate,
}

#[derive(Clone, Copy, Debug)]
struct MenuFrame {
    page: MenuPage,
    selected_index: usize,
}

#[derive(Component, Debug)]
pub struct MenuStack {
    frames: Vec<MenuFrame>,
}

impl MenuStack {
    pub fn new(initial_page: MenuPage) -> Self {
        Self {
            frames: vec![MenuFrame {
                page: initial_page,
                selected_index: 0,
            }],
        }
    }

    pub fn current_page(&self) -> Option<MenuPage> {
        self.frames.last().map(|frame| frame.page)
    }

    pub fn remember_selected_index(&mut self, selected_index: usize) {
        if let Some(frame) = self.frames.last_mut() {
            frame.selected_index = selected_index;
        }
    }

    pub fn current_selected_index(&self) -> usize {
        self.frames
            .last()
            .map(|frame| frame.selected_index)
            .unwrap_or(0)
    }

    pub fn push(&mut self, page: MenuPage) {
        self.frames.push(MenuFrame {
            page,
            selected_index: 0,
        });
    }

    pub fn pop(&mut self) -> Option<usize> {
        if self.frames.len() <= 1 {
            return None;
        }
        self.frames.pop();
        Some(self.current_selected_index())
    }
}

#[derive(Component, Clone, Copy)]
pub struct MenuPageContent;

#[derive(Component)]
pub struct PauseMenuAudio;

#[derive(Component, Clone)]
pub struct MenuOptionCommand(pub MenuCommand);

#[derive(Component, Clone, Copy)]
pub(super) struct VideoOptionsTable;

#[derive(Component, Clone, Copy)]
pub(super) struct VideoOptionRow {
    pub(super) index: usize,
}

#[derive(Component)]
pub(super) struct VideoResolutionDropdown;

#[derive(Component)]
pub(super) struct VideoModalRoot;

#[derive(Component)]
pub(super) struct VideoApplyConfirmModal;

#[derive(Component)]
pub(super) struct VideoExitUnsavedModal;

#[derive(Component)]
pub(super) struct VideoApplyCountdownText;

#[derive(Component, Clone, Copy)]
pub(super) struct VideoResolutionDropdownItem {
    pub(super) index: usize,
}

#[derive(Component, Clone, Copy)]
pub(super) struct VideoResolutionDropdownItemBaseY(pub(super) f32);

#[derive(Component)]
pub(super) struct VideoResolutionDropdownValueArrow;

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub(super) enum VideoResolutionDropdownValueArrowSide {
    Left,
    Right,
}

#[derive(Component)]
pub(super) struct VideoResolutionDropdownValueArrowAttached;

#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum VideoModalButton {
    ApplyKeep,
    ApplyRevert,
    ExitWithoutSaving,
    ExitCancel,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum VideoDisplayMode {
    Windowed,
    Borderless,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct VideoSettingsSnapshot {
    pub(super) display_mode: VideoDisplayMode,
    pub(super) resolution_index: usize,
    pub(super) vsync_enabled: bool,
}

impl Default for VideoSettingsSnapshot {
    fn default() -> Self {
        Self {
            display_mode: VideoDisplayMode::Windowed,
            resolution_index: 0,
            vsync_enabled: true,
        }
    }
}

#[derive(Resource, Debug)]
pub(super) struct VideoSettingsState {
    pub(super) initialized: bool,
    pub(super) saved: VideoSettingsSnapshot,
    pub(super) pending: VideoSettingsSnapshot,
    pub(super) revert_snapshot: Option<VideoSettingsSnapshot>,
    pub(super) apply_timer: Option<Timer>,
}

impl Default for VideoSettingsState {
    fn default() -> Self {
        let snapshot = VideoSettingsSnapshot::default();
        Self {
            initialized: false,
            saved: snapshot,
            pending: snapshot,
            revert_snapshot: None,
            apply_timer: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MenuCommand {
    None,
    Push(MenuPage),
    Pop,
    ToggleResolutionDropdown,
    ToggleDisplayMode,
    ToggleVsync,
    ApplyVideoSettings,
    ResetVideoDefaults,
    SetPause(PauseState),
    SetMain(MainState),
    SetPauseAndMain(PauseState, MainState),
    ExitApplication,
    CloseMenu,
}

#[derive(Clone)]
pub(super) struct MenuOptionDef {
    pub(super) name: &'static str,
    pub(super) label: &'static str,
    pub(super) y: f32,
    pub(super) command: MenuCommand,
    pub(super) shortcut: Option<KeyCode>,
    pub(super) cyclable: bool,
}

pub(super) struct MenuPageDef {
    pub(super) name_prefix: &'static str,
    pub(super) title: &'static str,
    pub(super) hint: &'static str,
    pub(super) layout: SystemMenuLayout,
    pub(super) options: &'static [MenuOptionDef],
}

const PAUSE_ROOT_OPTIONS: [MenuOptionDef; 4] = [
    MenuOptionDef {
        name: "pause_menu_options_option",
        label: PAUSE_MENU_OPTIONS_TEXT,
        y: 50.0,
        command: MenuCommand::Push(MenuPage::Options),
        shortcut: Some(KeyCode::KeyO),
        cyclable: false,
    },
    MenuOptionDef {
        name: "pause_menu_exit_to_menu_option",
        label: PAUSE_MENU_EXIT_TO_MENU_TEXT,
        y: 5.0,
        command: MenuCommand::SetPauseAndMain(PauseState::Unpaused, MainState::Menu),
        shortcut: Some(KeyCode::KeyM),
        cyclable: false,
    },
    MenuOptionDef {
        name: "pause_menu_exit_to_desktop_option",
        label: PAUSE_MENU_EXIT_TO_DESKTOP_TEXT,
        y: -40.0,
        command: MenuCommand::ExitApplication,
        shortcut: Some(KeyCode::KeyD),
        cyclable: false,
    },
    MenuOptionDef {
        name: "pause_menu_resume_option",
        label: PAUSE_MENU_RESUME_TEXT,
        y: -85.0,
        command: MenuCommand::SetPause(PauseState::Unpaused),
        shortcut: Some(KeyCode::Backspace),
        cyclable: false,
    },
];

const DEBUG_ROOT_OPTIONS: [MenuOptionDef; 2] = [
    MenuOptionDef {
        name: "debug_menu_close_option",
        label: DEBUG_MENU_RESUME_TEXT,
        y: 25.0,
        command: MenuCommand::CloseMenu,
        shortcut: None,
        cyclable: false,
    },
    MenuOptionDef {
        name: "debug_menu_main_menu_option",
        label: DEBUG_MENU_MAIN_MENU_TEXT,
        y: -25.0,
        command: MenuCommand::SetMain(MainState::Menu),
        shortcut: None,
        cyclable: false,
    },
];

const OPTIONS_MENU_OPTIONS: [MenuOptionDef; 4] = [
    MenuOptionDef {
        name: "system_options_video_option",
        label: OPTIONS_MENU_VIDEO_TEXT,
        y: 60.0,
        command: MenuCommand::Push(MenuPage::Video),
        shortcut: None,
        cyclable: false,
    },
    MenuOptionDef {
        name: "system_options_audio_option",
        label: OPTIONS_MENU_AUDIO_TEXT,
        y: 20.0,
        command: MenuCommand::None,
        shortcut: None,
        cyclable: false,
    },
    MenuOptionDef {
        name: "system_options_controls_option",
        label: OPTIONS_MENU_CONTROLS_TEXT,
        y: -20.0,
        command: MenuCommand::None,
        shortcut: None,
        cyclable: false,
    },
    MenuOptionDef {
        name: "system_options_back_option",
        label: OPTIONS_MENU_BACK_TEXT,
        y: -65.0,
        command: MenuCommand::Pop,
        shortcut: Some(KeyCode::Backspace),
        cyclable: false,
    },
];

const VIDEO_MENU_OPTIONS: [MenuOptionDef; 6] = [
    MenuOptionDef {
        name: "system_video_display_mode_option",
        label: VIDEO_MENU_DISPLAY_MODE_TEXT,
        y: 90.0,
        command: MenuCommand::ToggleDisplayMode,
        shortcut: Some(KeyCode::KeyF),
        cyclable: true,
    },
    MenuOptionDef {
        name: "system_video_resolution_option",
        label: VIDEO_MENU_RESOLUTION_TEXT,
        y: 50.0,
        command: MenuCommand::ToggleResolutionDropdown,
        shortcut: Some(KeyCode::KeyR),
        cyclable: false,
    },
    MenuOptionDef {
        name: "system_video_vsync_option",
        label: VIDEO_MENU_VSYNC_TEXT,
        y: 10.0,
        command: MenuCommand::ToggleVsync,
        shortcut: Some(KeyCode::KeyV),
        cyclable: true,
    },
    MenuOptionDef {
        name: "system_video_apply_option",
        label: VIDEO_MENU_APPLY_TEXT,
        y: -30.0,
        command: MenuCommand::ApplyVideoSettings,
        shortcut: Some(KeyCode::KeyA),
        cyclable: false,
    },
    MenuOptionDef {
        name: "system_video_reset_option",
        label: VIDEO_MENU_RESET_TEXT,
        y: -70.0,
        command: MenuCommand::ResetVideoDefaults,
        shortcut: Some(KeyCode::KeyZ),
        cyclable: false,
    },
    MenuOptionDef {
        name: "system_video_back_option",
        label: VIDEO_MENU_BACK_TEXT,
        y: -110.0,
        command: MenuCommand::Pop,
        shortcut: Some(KeyCode::Backspace),
        cyclable: false,
    },
];

pub(super) fn page_definition(page: MenuPage) -> MenuPageDef {
    match page {
        MenuPage::PauseRoot => MenuPageDef {
            name_prefix: "pause_menu",
            title: PAUSE_MENU_TITLE,
            hint: PAUSE_MENU_HINT,
            layout: SystemMenuLayout::new(Vec2::new(PANEL_WIDTH, 630.0), 182.0, 140.0),
            options: &PAUSE_ROOT_OPTIONS,
        },
        MenuPage::DebugRoot => MenuPageDef {
            name_prefix: "debug_menu",
            title: DEBUG_MENU_TITLE,
            hint: DEBUG_MENU_HINT,
            layout: SystemMenuLayout::new(Vec2::new(PANEL_WIDTH, PANEL_WIDTH * 1.5), 150.0, 110.0),
            options: &DEBUG_ROOT_OPTIONS,
        },
        MenuPage::Options => MenuPageDef {
            name_prefix: "system_options_menu",
            title: OPTIONS_MENU_TITLE,
            hint: OPTIONS_MENU_HINT,
            layout: SystemMenuLayout::new(Vec2::new(PANEL_WIDTH, 630.0), 182.0, 140.0),
            options: &OPTIONS_MENU_OPTIONS,
        },
        MenuPage::Video => MenuPageDef {
            name_prefix: "system_video_menu",
            title: VIDEO_MENU_TITLE,
            hint: VIDEO_MENU_HINT,
            layout: SystemMenuLayout::new(Vec2::new(PANEL_WIDTH * 2.0, 630.0), 182.0, 140.0),
            options: &VIDEO_MENU_OPTIONS,
        },
    }
}

pub(super) fn video_options_table() -> Table {
    let left_cells = vec![
        Cell::new(TextContent::new(
            VIDEO_MENU_DISPLAY_MODE_TEXT.to_string(),
            SYSTEM_MENU_COLOR,
            VIDEO_TABLE_TEXT_SIZE,
        )),
        Cell::new(TextContent::new(
            VIDEO_MENU_RESOLUTION_TEXT.to_string(),
            SYSTEM_MENU_COLOR,
            VIDEO_TABLE_TEXT_SIZE,
        )),
        Cell::new(TextContent::new(
            VIDEO_MENU_VSYNC_TEXT.to_string(),
            SYSTEM_MENU_COLOR,
            VIDEO_TABLE_TEXT_SIZE,
        )),
        Cell::new(TextContent::new(
            VIDEO_MENU_APPLY_TEXT.to_string(),
            SYSTEM_MENU_COLOR,
            VIDEO_TABLE_TEXT_SIZE,
        )),
        Cell::new(TextContent::new(
            VIDEO_MENU_RESET_TEXT.to_string(),
            SYSTEM_MENU_COLOR,
            VIDEO_TABLE_TEXT_SIZE,
        )),
        Cell::new(TextContent::new(
            VIDEO_MENU_BACK_TEXT.to_string(),
            SYSTEM_MENU_COLOR,
            VIDEO_TABLE_TEXT_SIZE,
        )),
    ];

    let right_cells = vec![
        Cell::new(TextContent::new(
            VIDEO_MENU_VALUE_PLACEHOLDER.to_string(),
            SYSTEM_MENU_COLOR,
            VIDEO_TABLE_TEXT_SIZE,
        )),
        Cell::new(TextContent::new(
            VIDEO_MENU_VALUE_PLACEHOLDER.to_string(),
            SYSTEM_MENU_COLOR,
            VIDEO_TABLE_TEXT_SIZE,
        )),
        Cell::new(TextContent::new(
            VIDEO_MENU_VALUE_PLACEHOLDER.to_string(),
            SYSTEM_MENU_COLOR,
            VIDEO_TABLE_TEXT_SIZE,
        )),
        Cell::new(TextContent::new(
            String::new(),
            SYSTEM_MENU_COLOR,
            VIDEO_TABLE_TEXT_SIZE,
        )),
        Cell::new(TextContent::new(
            String::new(),
            SYSTEM_MENU_COLOR,
            VIDEO_TABLE_TEXT_SIZE,
        )),
        Cell::new(TextContent::new(
            String::new(),
            SYSTEM_MENU_COLOR,
            VIDEO_TABLE_TEXT_SIZE,
        )),
    ];

    let rows = vec![
        Row {
            height: VIDEO_TABLE_ROW_HEIGHT,
        };
        left_cells.len()
    ];

    let left_column = Column::new(
        left_cells,
        VIDEO_TABLE_LABEL_COLUMN_WIDTH,
        Vec2::new(10.0, 8.0),
        Anchor::CENTER_RIGHT,
        false,
    );
    let right_column = Column::new(
        right_cells,
        VIDEO_TABLE_VALUE_COLUMN_WIDTH,
        Vec2::new(10.0, 8.0),
        Anchor::CENTER,
        false,
    );

    Table {
        columns: vec![left_column, right_column],
        rows,
    }
}

pub(super) fn display_mode_text(mode: VideoDisplayMode) -> &'static str {
    match mode {
        VideoDisplayMode::Windowed => "Windowed",
        VideoDisplayMode::Borderless => "Borderless",
    }
}

pub(super) fn video_row_center_y(index: usize) -> f32 {
    VIDEO_TABLE_Y - (index as f32 + 0.5) * VIDEO_TABLE_ROW_HEIGHT
}

pub(super) fn video_row_top_y(index: usize) -> f32 {
    VIDEO_TABLE_Y - index as f32 * VIDEO_TABLE_ROW_HEIGHT
}

pub(super) fn dropdown_center_y_from_row_top(row_index: usize, num_rows: usize) -> f32 {
    let dropdown_height = num_rows as f32 * VIDEO_RESOLUTION_DROPDOWN_ROW_HEIGHT;
    video_row_top_y(row_index) - dropdown_height * 0.5
}

pub(super) fn dropdown_item_local_center_y(index: usize, num_rows: usize) -> f32 {
    let dropdown_height = num_rows as f32 * VIDEO_RESOLUTION_DROPDOWN_ROW_HEIGHT;
    dropdown_height * 0.5 - (index as f32 + 0.5) * VIDEO_RESOLUTION_DROPDOWN_ROW_HEIGHT
}

pub(super) fn dropdown_resolution_shortcut_index(
    keyboard_input: &ButtonInput<KeyCode>,
) -> Option<usize> {
    let digit_keys = [
        KeyCode::Digit1,
        KeyCode::Digit2,
        KeyCode::Digit3,
        KeyCode::Digit4,
        KeyCode::Digit5,
        KeyCode::Digit6,
        KeyCode::Digit7,
        KeyCode::Digit8,
        KeyCode::Digit9,
        KeyCode::Digit0,
    ];

    for (digit_index, keycode) in digit_keys.iter().copied().enumerate() {
        if keyboard_input.just_pressed(keycode) {
            let resolution_index = if keycode == KeyCode::Digit0 {
                9
            } else {
                digit_index
            };
            if resolution_index < RESOLUTIONS.len() {
                return Some(resolution_index);
            }
            break;
        }
    }

    None
}

pub(super) fn video_resolution_option_index() -> usize {
    VIDEO_MENU_OPTIONS
        .iter()
        .position(|option| matches!(option.command, MenuCommand::ToggleResolutionDropdown))
        .unwrap_or(VIDEO_RESOLUTION_OPTION_INDEX)
}

pub(super) fn present_mode_text(vsync_enabled: bool) -> &'static str {
    if vsync_enabled {
        "On"
    } else {
        "Off"
    }
}

pub(super) fn default_video_settings() -> VideoSettingsSnapshot {
    VideoSettingsSnapshot::default()
}

pub(super) fn video_settings_dirty(state: &VideoSettingsState) -> bool {
    state.pending != state.saved
}
