use bevy::{prelude::*, sprite::Anchor};

use crate::{
    data::states::{MainState, PauseState},
    entities::{
        sprites::compound::RectangleSides,
        text::{scaled_font_size, Cell, Column, Row, Table, TextContent},
    },
    startup::system_menu::{SystemMenuLayout, CYCLE_ARROW_HALF_WIDTH, PANEL_WIDTH},
    systems::{
        colors::SYSTEM_MENU_COLOR,
        interaction::InteractionGate,
        ui::discrete_slider::{slot_center_x, slot_span_bounds},
    },
};

pub(super) const PAUSE_MENU_TITLE: &str = "PAUSED";
pub(super) const PAUSE_MENU_HINT: &str = "[ESCAPE TO RESUME]\n[ARROW UP/ARROW DOWN + ENTER]";
pub(super) const PAUSE_MENU_RESUME_TEXT: &str = "RESUME [⌫]";
pub(super) const PAUSE_MENU_OPTIONS_TEXT: &str = "OPTIONS [o]";
pub(super) const PAUSE_MENU_EXIT_TO_MENU_TEXT: &str = "EXIT TO MENU [m]";
pub(super) const PAUSE_MENU_EXIT_TO_DESKTOP_TEXT: &str = "EXIT TO DESKTOP [d]";

pub(super) const DEBUG_MENU_TITLE: &str = "DEBUG OVERLAY MENU";
pub(super) const DEBUG_MENU_HINT: &str = "[ARROW UP/ARROW DOWN + ENTER]\n[CLICK ALSO WORKS]";
pub(super) const DEBUG_MENU_UI_SHOWCASE_TEXT: &str = "UI DEBUG WINDOWS";
pub(super) const DEBUG_MENU_RESUME_TEXT: &str = "CLOSE DEBUG MENU";
pub(super) const DEBUG_MENU_MAIN_MENU_TEXT: &str = "RETURN TO MAIN MENU";

pub(super) const OPTIONS_MENU_TITLE: &str = "OPTIONS";
pub(super) const OPTIONS_MENU_HINT: &str = "[ARROW UP/ARROW DOWN + ENTER]";
pub(super) const OPTIONS_MENU_VIDEO_TEXT: &str = "VIDEO";
pub(super) const OPTIONS_MENU_AUDIO_TEXT: &str = "AUDIO";
pub(super) const OPTIONS_MENU_CONTROLS_TEXT: &str = "CONTROLS";
pub(super) const OPTIONS_MENU_BACK_TEXT: &str = "BACK [⌫]";
pub(super) const VIDEO_MENU_TITLE: &str = "VIDEO";
pub(super) const VIDEO_MENU_HINT: &str = "[ARROW UP/ARROW DOWN + ENTER]\n[TAB TO CYCLE TABS]";
pub(super) const VIDEO_MENU_DISPLAY_MODE_TEXT: &str = "DISPLAY MODE [f]";
pub(super) const VIDEO_MENU_RESOLUTION_TEXT: &str = "RESOLUTION [r]";
pub(super) const VIDEO_MENU_PRESENT_MODE_TEXT: &str = "PRESENT MODE [p]";
pub(super) const VIDEO_MENU_MSAA_TEXT: &str = "MSAA [m]";
pub(super) const VIDEO_MENU_RESIZABLE_TEXT: &str = "WINDOW RESIZABLE";
pub(super) const VIDEO_MENU_DECORATIONS_TEXT: &str = "WINDOW BORDER";
pub(super) const VIDEO_MENU_TRANSPARENT_TEXT: &str = "WINDOW TRANSPARENT";
pub(super) const VIDEO_MENU_COMPOSITE_ALPHA_TEXT: &str = "COMPOSITE ALPHA";
pub(super) const VIDEO_MENU_BLOOM_ENABLED_TEXT: &str = "BLOOM ENABLED";
pub(super) const VIDEO_MENU_BLOOM_STYLE_TEXT: &str = "BLOOM STYLE";
pub(super) const VIDEO_MENU_BLOOM_INTENSITY_TEXT: &str = "BLOOM INTENSITY";
pub(super) const VIDEO_MENU_BLOOM_SCATTER_TEXT: &str = "BLOOM SCATTER";
pub(super) const VIDEO_MENU_BLOOM_THRESHOLD_TEXT: &str = "BLOOM THRESHOLD";
pub(super) const VIDEO_MENU_BLOOM_COMPOSITE_TEXT: &str = "BLOOM COMPOSITE";
pub(super) const VIDEO_MENU_ANAMORPHIC_TEXT: &str = "ANAMORPHIC";
pub(super) const VIDEO_MENU_BLOOM_BOOST_TEXT: &str = "BLOOM BOOST";
pub(super) const VIDEO_MENU_TONEMAP_TEXT: &str = "TONEMAPPING";
pub(super) const VIDEO_MENU_DEBAND_TEXT: &str = "DEBAND DITHER";
pub(super) const VIDEO_MENU_FXAA_TEXT: &str = "FXAA";
pub(super) const VIDEO_MENU_FXAA_QUALITY_TEXT: &str = "FXAA QUALITY";
pub(super) const VIDEO_MENU_CAS_TEXT: &str = "CAS SHARPEN";
pub(super) const VIDEO_MENU_CAS_STRENGTH_TEXT: &str = "CAS STRENGTH";
pub(super) const VIDEO_MENU_CHROMATIC_TEXT: &str = "CHROMATIC";
pub(super) const VIDEO_MENU_CHROMATIC_INTENSITY_TEXT: &str = "CHROMATIC AMOUNT";
pub(super) const VIDEO_MENU_CRT_ENABLED_TEXT: &str = "CRT ENABLED";
pub(super) const VIDEO_MENU_SCAN_SPACING_TEXT: &str = "SCAN LINES";
pub(super) const VIDEO_MENU_SCAN_THICKNESS_TEXT: &str = "CURVATURE";
pub(super) const VIDEO_MENU_SCAN_DARKNESS_TEXT: &str = "STATIC";
pub(super) const VIDEO_MENU_SCAN_JITTER_TEXT: &str = "JITTER";
pub(super) const VIDEO_MENU_SCAN_ABERRATION_TEXT: &str = "RGB SPLIT";
pub(super) const VIDEO_MENU_SCAN_PHOSPHOR_TEXT: &str = "PHOSPHOR MASK";
pub(super) const VIDEO_MENU_SCAN_VIGNETTE_TEXT: &str = "VIGNETTE";
pub(super) const VIDEO_MENU_APPLY_TEXT: &str = "APPLY [a]";
pub(super) const VIDEO_MENU_RESET_TEXT: &str = "RESET [z]";
pub(super) const VIDEO_MENU_BACK_TEXT: &str = "BACK [⌫]";
pub(super) const VIDEO_MENU_VALUE_PLACEHOLDER: &str = "---";
pub(super) const VIDEO_TABS: [&str; 4] = ["DISPLAY", "BLOOM", "ADVANCED", "CRT"];
pub(super) const VIDEO_TOP_OPTION_COUNT: usize = 8;
pub(super) const VIDEO_FOOTER_OPTION_COUNT: usize = 3;
pub(super) const VIDEO_FOOTER_OPTION_START_INDEX: usize = VIDEO_TOP_OPTION_COUNT;
pub(super) const VIDEO_TABLE_TOTAL_WIDTH: f32 = 780.0;
pub(super) const VIDEO_TABLE_LABEL_COLUMN_WIDTH: f32 = VIDEO_TABLE_TOTAL_WIDTH / 3.0;
pub(super) const VIDEO_TABLE_VALUE_COLUMN_WIDTH: f32 =
    VIDEO_TABLE_TOTAL_WIDTH - VIDEO_TABLE_LABEL_COLUMN_WIDTH;
pub(super) const VIDEO_TABLE_ROW_HEIGHT: f32 = 40.0;
pub(super) const VIDEO_TABS_ROW_HEIGHT: f32 = 40.0;
pub(super) const VIDEO_FOOTER_ROW_HEIGHT: f32 = 42.0;
pub(super) const VIDEO_TABLE_TEXT_SIZE: f32 = scaled_font_size(14.0);
pub(super) const VIDEO_TABLE_TEXT_SELECTED_SIZE: f32 = scaled_font_size(21.0);
pub(super) const VIDEO_TABS_TEXT_SIZE: f32 = scaled_font_size(14.0);
pub(super) const VIDEO_TABS_TEXT_SELECTED_SIZE: f32 = VIDEO_TABLE_TEXT_SELECTED_SIZE;
pub(super) const VIDEO_TABLE_TEXT_Z: f32 = 0.3;
pub(super) const VIDEO_TABLE_X: f32 = -VIDEO_TABLE_TOTAL_WIDTH * 0.5;
pub(super) const VIDEO_TABLE_Y: f32 = 72.0;
pub(super) const VIDEO_TABS_TABLE_Y: f32 = 152.0;
pub(super) const VIDEO_FOOTER_TABLE_Y: f32 = -152.0;
pub(super) const VIDEO_FOOTER_SEPARATOR_THICKNESS: f32 = 2.0;
pub(super) const VIDEO_FOOTER_SEPARATOR_Y: f32 = VIDEO_FOOTER_TABLE_Y;
pub(super) const VIDEO_TOP_SCROLL_VIEWPORT_BASE_TOP_Y: f32 =
    VIDEO_TABS_TABLE_Y - VIDEO_TABS_ROW_HEIGHT - 1.0;
pub(super) const VIDEO_TOP_SCROLL_VIEWPORT_BASE_BOTTOM_Y: f32 =
    VIDEO_FOOTER_SEPARATOR_Y + VIDEO_FOOTER_SEPARATOR_THICKNESS * 0.5;
pub(super) const VIDEO_TOP_SCROLL_VIEWPORT_BASE_HEIGHT: f32 =
    VIDEO_TOP_SCROLL_VIEWPORT_BASE_TOP_Y - VIDEO_TOP_SCROLL_VIEWPORT_BASE_BOTTOM_Y;
pub(super) const VIDEO_TOP_SCROLL_VIEWPORT_VERTICAL_INSET: f32 =
    VIDEO_TOP_SCROLL_VIEWPORT_BASE_HEIGHT * 0.10;
pub(super) const VIDEO_TOP_SCROLL_VIEWPORT_TOP_Y: f32 =
    VIDEO_TOP_SCROLL_VIEWPORT_BASE_TOP_Y - VIDEO_TOP_SCROLL_VIEWPORT_VERTICAL_INSET;
pub(super) const VIDEO_TOP_SCROLL_VIEWPORT_BOTTOM_Y: f32 =
    VIDEO_TOP_SCROLL_VIEWPORT_BASE_BOTTOM_Y + VIDEO_TOP_SCROLL_VIEWPORT_VERTICAL_INSET;
pub(super) const VIDEO_TOP_SCROLL_VIEWPORT_HEIGHT: f32 =
    VIDEO_TOP_SCROLL_VIEWPORT_TOP_Y - VIDEO_TOP_SCROLL_VIEWPORT_BOTTOM_Y;
pub(super) const VIDEO_TOP_SCROLL_CENTER_Y: f32 =
    (VIDEO_TOP_SCROLL_VIEWPORT_TOP_Y + VIDEO_TOP_SCROLL_VIEWPORT_BOTTOM_Y) * 0.5;
pub(super) const VIDEO_TOP_SCROLL_LEADING_PADDING: f32 =
    VIDEO_TOP_SCROLL_VIEWPORT_TOP_Y - VIDEO_TABLE_Y;
pub(super) const VIDEO_TOP_SCROLL_CONTENT_HEIGHT: f32 =
    VIDEO_TOP_SCROLL_LEADING_PADDING + VIDEO_TOP_OPTION_COUNT as f32 * VIDEO_TABLE_ROW_HEIGHT;
pub(super) const VIDEO_TOP_SCROLL_Z: f32 = 0.94;
pub(super) const VIDEO_OPTION_SELECTOR_X: f32 = 0.0;
pub(super) const VIDEO_OPTION_REGION_WIDTH: f32 = VIDEO_TABLE_TOTAL_WIDTH;
pub(super) const VIDEO_OPTION_REGION_HEIGHT: f32 = 38.0;
pub(super) const VIDEO_DISCRETE_SLIDER_MAX_STEPS: usize = 4;
pub(super) const VIDEO_DISCRETE_SLIDER_SLOT_SIZE: Vec2 = Vec2::new(18.0, 18.0);
pub(super) const VIDEO_DISCRETE_SLIDER_SLOT_SIZE_SELECTED: Vec2 = Vec2::new(22.0, 22.0);
pub(super) const VIDEO_DISCRETE_SLIDER_ZERO_CUTOFF_LEFT_BIAS_PX: f32 = 2.0;
pub(super) const VIDEO_DISCRETE_SLIDER_GAP: f32 = 7.0;
pub(super) const VIDEO_DISCRETE_SLIDER_ROOT_X: f32 =
    VIDEO_TABLE_X + VIDEO_TABLE_LABEL_COLUMN_WIDTH + VIDEO_TABLE_VALUE_COLUMN_WIDTH * 0.5 - 24.0;
pub(super) const VIDEO_DISCRETE_SLIDER_LABEL_OFFSET_X: f32 = 160.0;
pub(super) const VIDEO_DISCRETE_SLIDER_Z: f32 = VIDEO_TABLE_TEXT_Z;
pub(super) const VIDEO_SLIDER_ARROW_MARGIN: f32 = 18.0;
pub(super) const VIDEO_SELECTOR_ARROW_SPREAD: f32 = 88.0;
pub(super) const VIDEO_FOOTER_COLUMN_WIDTH: f32 = VIDEO_TABLE_TOTAL_WIDTH / 3.0;
pub(super) const VIDEO_FOOTER_OPTION_REGION_WIDTH: f32 = VIDEO_FOOTER_COLUMN_WIDTH - 2.0;
pub(super) const VIDEO_FOOTER_OPTION_REGION_HEIGHT: f32 = VIDEO_FOOTER_ROW_HEIGHT + 16.0;
pub(super) const VIDEO_FOOTER_INDICATOR_X: f32 = VIDEO_FOOTER_COLUMN_WIDTH * 0.42;
// Center of the value column relative to the option entity (at x=0):
// VIDEO_TABLE_X + LABEL_WIDTH + VALUE_WIDTH/2 = -390 + 390 + 195 = 195
pub(super) const VIDEO_VALUE_COLUMN_CENTER_X: f32 =
    VIDEO_TABLE_X + VIDEO_TABLE_LABEL_COLUMN_WIDTH + VIDEO_TABLE_VALUE_COLUMN_WIDTH * 0.5;
pub(super) const VIDEO_NAME_COLUMN_CENTER_X: f32 = VIDEO_TABLE_X + VIDEO_TABLE_LABEL_COLUMN_WIDTH * 0.5;
// Keep the name highlight above table cell backgrounds (z≈0.95 world)
// while staying below text (text z is boosted separately).
pub(super) const VIDEO_NAME_HIGHLIGHT_Z: f32 = -0.02;
pub(super) const VIDEO_NAME_HIGHLIGHT_WIDTH: f32 = VIDEO_TABLE_LABEL_COLUMN_WIDTH - 6.0;
pub(super) const VIDEO_NAME_HIGHLIGHT_HEIGHT: f32 = VIDEO_OPTION_REGION_HEIGHT - 2.0;
pub(super) const VIDEO_HOVER_BOX_DELAY_SECONDS: f32 = 0.5;
pub(super) const VIDEO_HOVER_BOX_SIZE: Vec2 = Vec2::new(340.0, 72.0);
pub(super) const VIDEO_HOVER_BOX_TEXT_SIZE: f32 = scaled_font_size(12.0);
pub(super) const VIDEO_HOVER_BOX_CLAMP_INSET: Vec2 = Vec2::new(28.0, 28.0);
// Dropdown entries render above base option rows, so their tooltip root needs
// a higher z to avoid appearing under the dropdown panel/items.
pub(super) const VIDEO_DROPDOWN_HOVER_BOX_Z: f32 = 2.2;
pub(super) const VIDEO_RESOLUTION_DROPDOWN_WIDTH: f32 = VIDEO_TABLE_VALUE_COLUMN_WIDTH;
pub(super) const VIDEO_RESOLUTION_DROPDOWN_BACKGROUND_PAD_X: f32 = 8.0;
pub(super) const VIDEO_RESOLUTION_DROPDOWN_ROW_HEIGHT: f32 = VIDEO_TABLE_ROW_HEIGHT;
pub(super) const VIDEO_RESOLUTION_DROPDOWN_Z: f32 = 1.8;
pub(super) const VIDEO_RESOLUTION_DROPDOWN_BORDER_Z: f32 = 0.04;
pub(super) const VIDEO_RESOLUTION_DROPDOWN_ITEM_Z: f32 = 0.08;
pub(super) const VIDEO_RESOLUTION_DROPDOWN_ARROW_Z: f32 = 0.28;
pub(super) const VIDEO_RESOLUTION_DROPDOWN_ARROW_WIDTH: f32 = 16.0;
pub(super) const VIDEO_RESOLUTION_DROPDOWN_ARROW_HEIGHT: f32 = 20.0;
pub(super) const VIDEO_RESOLUTION_DROPDOWN_ARROW_INSET: f32 = 18.0;
pub(super) const VIDEO_RESOLUTION_DROPDOWN_ARROW_SPREAD: f32 =
    VIDEO_RESOLUTION_DROPDOWN_WIDTH * 0.5 - VIDEO_RESOLUTION_DROPDOWN_ARROW_INSET;
pub(super) const VIDEO_DROPDOWN_MAX_ROWS: usize = 10;
pub(super) const VIDEO_RESOLUTION_OPTION_INDEX: usize = 1;
pub(super) const VIDEO_MODAL_PANEL_SIZE: Vec2 = Vec2::new(540.0, 230.0);
pub(super) const VIDEO_MODAL_DIM_ALPHA: f32 = 0.72;
pub(super) const VIDEO_MODAL_DIM_SIZE: f32 = 6000.0;
pub(super) const VIDEO_MODAL_DIM_Z: f32 = -0.05;
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

#[derive(Component)]
pub struct MainMenuOptionsOverlay;

#[derive(Component, Clone)]
pub struct MenuOptionCommand(pub MenuCommand);

#[derive(Component, Clone, Copy)]
pub(super) struct VideoTopOptionsTable;

#[derive(Component, Clone, Copy)]
pub(super) struct VideoTopOptionsScrollRoot;

#[derive(Component, Clone, Copy)]
pub(super) struct VideoTopOptionsScrollContent;

#[derive(Component, Clone, Copy)]
pub(super) struct VideoFooterOptionsTable;

#[derive(Component, Clone, Copy)]
pub(super) struct VideoTabsTable;

#[derive(Component)]
pub(super) struct VideoTabsInteractionRoot;

#[derive(Component, Clone, Copy)]
pub(super) struct VideoOptionHoverBoxRoot;

#[derive(Component, Clone, Copy)]
pub(super) struct VideoDropdownHoverBoxRoot;

#[derive(Component, Clone, Copy)]
pub(super) struct VideoTabOption {
    pub(super) index: usize,
}

#[derive(Component, Clone, Copy)]
pub(super) struct VideoOptionRow {
    pub(super) index: usize,
}

#[derive(Component, Clone, Copy)]
pub(super) struct VideoOptionDiscreteSlider {
    pub(super) menu_entity: Entity,
    pub(super) row: usize,
}

#[derive(Component)]
pub(super) struct VideoOptionDiscreteSliderLabel;

#[derive(Component)]
pub(super) struct VideoResolutionDropdown;

#[derive(Component)]
pub(super) struct VideoResolutionDropdownBorder;

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
    Fullscreen,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum VideoPresentMode {
    AutoVsync,
    AutoNoVsync,
    Fifo,
    FifoRelaxed,
    Immediate,
    Mailbox,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum VideoMsaa {
    Off,
    Sample2,
    Sample4,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum VideoCompositeAlpha {
    Auto,
    Opaque,
    PreMultiplied,
    PostMultiplied,
    Inherit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum VideoTabKind {
    Display,
    Bloom,
    Advanced,
    Crt,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum BloomStyle {
    Natural,
    OldSchool,
    ScreenBlur,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum BloomIntensity {
    Low,
    Medium,
    High,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum BloomScatter {
    Tight,
    Normal,
    Wide,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum BloomComposite {
    EnergyConserving,
    Additive,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum BloomThreshold {
    Off,
    Soft,
    Hard,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum AnamorphicScale {
    Off,
    Wide,
    UltraWide,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum BloomBoost {
    Off,
    Low,
    Medium,
    High,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum VideoTonemapping {
    None,
    Reinhard,
    ReinhardLuminance,
    AcesFitted,
    AgX,
    SomewhatBoringDisplayTransform,
    TonyMcMapface,
    BlenderFilmic,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum FxaaQuality {
    Low,
    Medium,
    High,
    Ultra,
    Extreme,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum CasStrength {
    Off,
    Low,
    Medium,
    High,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ScanSpacing {
    Off,
    Fine,
    Balanced,
    Sparse,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ScanThickness {
    Off,
    Low,
    Medium,
    High,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ScanDarkness {
    Off,
    Low,
    Medium,
    High,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum CrtEffectLevel {
    Off,
    Low,
    Medium,
    High,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct VideoSettingsSnapshot {
    pub(super) display_mode: VideoDisplayMode,
    pub(super) resolution_index: usize,
    pub(super) present_mode: VideoPresentMode,
    pub(super) msaa: VideoMsaa,
    pub(super) window_resizable: bool,
    pub(super) window_decorations: bool,
    pub(super) window_transparent: bool,
    pub(super) composite_alpha: VideoCompositeAlpha,
    pub(super) bloom_enabled: bool,
    pub(super) bloom_style: BloomStyle,
    pub(super) bloom_intensity: BloomIntensity,
    pub(super) bloom_scatter: BloomScatter,
    pub(super) bloom_composite: BloomComposite,
    pub(super) bloom_threshold: BloomThreshold,
    pub(super) anamorphic_scale: AnamorphicScale,
    pub(super) bloom_boost: BloomBoost,
    pub(super) tonemapping: VideoTonemapping,
    pub(super) deband_dither_enabled: bool,
    pub(super) fxaa_enabled: bool,
    pub(super) fxaa_quality: FxaaQuality,
    pub(super) cas_enabled: bool,
    pub(super) cas_strength: CasStrength,
    pub(super) chromatic_enabled: bool,
    pub(super) chromatic_intensity: CrtEffectLevel,
    pub(super) crt_enabled: bool,
    pub(super) scan_spacing: ScanSpacing,
    pub(super) scan_thickness: ScanThickness,
    pub(super) scan_darkness: ScanDarkness,
    pub(super) scan_jitter: CrtEffectLevel,
    pub(super) scan_aberration: CrtEffectLevel,
    pub(super) scan_phosphor: CrtEffectLevel,
    pub(super) scan_vignette: CrtEffectLevel,
}

impl Default for VideoSettingsSnapshot {
    fn default() -> Self {
        Self {
            display_mode: VideoDisplayMode::Windowed,
            resolution_index: 0,
            present_mode: VideoPresentMode::AutoNoVsync,
            msaa: VideoMsaa::Sample4,
            window_resizable: true,
            window_decorations: true,
            window_transparent: false,
            composite_alpha: VideoCompositeAlpha::Auto,
            bloom_enabled: true,
            bloom_style: BloomStyle::Natural,
            bloom_intensity: BloomIntensity::Medium,
            bloom_scatter: BloomScatter::Normal,
            bloom_composite: BloomComposite::EnergyConserving,
            bloom_threshold: BloomThreshold::Off,
            anamorphic_scale: AnamorphicScale::Off,
            bloom_boost: BloomBoost::Medium,
            tonemapping: VideoTonemapping::TonyMcMapface,
            deband_dither_enabled: false,
            fxaa_enabled: false,
            fxaa_quality: FxaaQuality::High,
            cas_enabled: false,
            cas_strength: CasStrength::Medium,
            chromatic_enabled: false,
            chromatic_intensity: CrtEffectLevel::Medium,
            crt_enabled: true,
            scan_spacing: ScanSpacing::Balanced,
            scan_thickness: ScanThickness::Medium,
            scan_darkness: ScanDarkness::Medium,
            scan_jitter: CrtEffectLevel::Medium,
            scan_aberration: CrtEffectLevel::Medium,
            scan_phosphor: CrtEffectLevel::High,
            scan_vignette: CrtEffectLevel::High,
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
    OpenMainMenuOptionsOverlay,
    NextScene,
    ToggleDebugUiShowcase,
    ToggleVideoTopOption(usize),
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

const DEBUG_ROOT_OPTIONS: [MenuOptionDef; 3] = [
    MenuOptionDef {
        name: "debug_menu_ui_showcase_option",
        label: DEBUG_MENU_UI_SHOWCASE_TEXT,
        y: 50.0,
        command: MenuCommand::ToggleDebugUiShowcase,
        shortcut: None,
        cyclable: false,
    },
    MenuOptionDef {
        name: "debug_menu_close_option",
        label: DEBUG_MENU_RESUME_TEXT,
        y: 0.0,
        command: MenuCommand::CloseMenu,
        shortcut: None,
        cyclable: false,
    },
    MenuOptionDef {
        name: "debug_menu_main_menu_option",
        label: DEBUG_MENU_MAIN_MENU_TEXT,
        y: -50.0,
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

const VIDEO_MENU_OPTIONS: [MenuOptionDef; VIDEO_TOP_OPTION_COUNT + VIDEO_FOOTER_OPTION_COUNT] = [
    MenuOptionDef {
        name: "system_video_display_mode_option",
        label: VIDEO_MENU_DISPLAY_MODE_TEXT,
        y: 90.0,
        command: MenuCommand::ToggleVideoTopOption(0),
        shortcut: Some(KeyCode::KeyF),
        cyclable: true,
    },
    MenuOptionDef {
        name: "system_video_resolution_option",
        label: VIDEO_MENU_RESOLUTION_TEXT,
        y: 50.0,
        command: MenuCommand::ToggleVideoTopOption(1),
        shortcut: Some(KeyCode::KeyR),
        cyclable: true,
    },
    MenuOptionDef {
        name: "system_video_present_mode_option",
        label: VIDEO_MENU_PRESENT_MODE_TEXT,
        y: 10.0,
        command: MenuCommand::ToggleVideoTopOption(2),
        shortcut: Some(KeyCode::KeyP),
        cyclable: true,
    },
    MenuOptionDef {
        name: "system_video_msaa_option",
        label: VIDEO_MENU_MSAA_TEXT,
        y: -30.0,
        command: MenuCommand::ToggleVideoTopOption(3),
        shortcut: Some(KeyCode::KeyM),
        cyclable: true,
    },
    MenuOptionDef {
        name: "system_video_resizable_option",
        label: VIDEO_MENU_RESIZABLE_TEXT,
        y: -70.0,
        command: MenuCommand::ToggleVideoTopOption(4),
        shortcut: None,
        cyclable: true,
    },
    MenuOptionDef {
        name: "system_video_decorations_option",
        label: VIDEO_MENU_DECORATIONS_TEXT,
        y: -110.0,
        command: MenuCommand::ToggleVideoTopOption(5),
        shortcut: None,
        cyclable: true,
    },
    MenuOptionDef {
        name: "system_video_transparent_option",
        label: VIDEO_MENU_TRANSPARENT_TEXT,
        y: -150.0,
        command: MenuCommand::ToggleVideoTopOption(6),
        shortcut: None,
        cyclable: true,
    },
    MenuOptionDef {
        name: "system_video_composite_alpha_option",
        label: VIDEO_MENU_COMPOSITE_ALPHA_TEXT,
        y: -190.0,
        command: MenuCommand::ToggleVideoTopOption(7),
        shortcut: None,
        cyclable: true,
    },
    MenuOptionDef {
        name: "system_video_apply_option",
        label: VIDEO_MENU_APPLY_TEXT,
        y: -230.0,
        command: MenuCommand::ApplyVideoSettings,
        shortcut: Some(KeyCode::KeyA),
        cyclable: false,
    },
    MenuOptionDef {
        name: "system_video_reset_option",
        label: VIDEO_MENU_RESET_TEXT,
        y: -270.0,
        command: MenuCommand::ResetVideoDefaults,
        shortcut: Some(KeyCode::KeyZ),
        cyclable: false,
    },
    MenuOptionDef {
        name: "system_video_back_option",
        label: VIDEO_MENU_BACK_TEXT,
        y: -310.0,
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
            layout: SystemMenuLayout::new(Vec2::new(PANEL_WIDTH * 2.0, 630.0), 214.0, 186.0),
            options: &VIDEO_MENU_OPTIONS,
        },
    }
}

pub(super) fn video_top_options_table() -> Table {
    let left_cells: Vec<Cell> = (0..VIDEO_TOP_OPTION_COUNT)
        .map(|_| {
            Cell::new(TextContent::new(
                VIDEO_MENU_VALUE_PLACEHOLDER.to_string(),
                SYSTEM_MENU_COLOR,
                VIDEO_TABLE_TEXT_SIZE,
            ))
        })
        .collect();

    let right_cells: Vec<Cell> = (0..VIDEO_TOP_OPTION_COUNT)
        .map(|_| {
            Cell::new(TextContent::new(
                VIDEO_MENU_VALUE_PLACEHOLDER.to_string(),
                SYSTEM_MENU_COLOR,
                VIDEO_TABLE_TEXT_SIZE,
            ))
        })
        .collect();

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

pub(super) fn video_tabs_table() -> Table {
    let tab_column_width = VIDEO_TABLE_TOTAL_WIDTH / VIDEO_TABS.len() as f32;
    let tab_border_sides = RectangleSides {
        top: true,
        bottom: true,
        left: true,
        right: true,
    };
    let columns = VIDEO_TABS
        .iter()
        .map(|label| {
            Column::new(
                vec![Cell::new(TextContent::new(
                    (*label).to_string(),
                    SYSTEM_MENU_COLOR,
                    VIDEO_TABS_TEXT_SIZE,
                ))],
                tab_column_width,
                Vec2::new(8.0, 6.0),
                Anchor::CENTER,
                false,
            )
            .with_cell_boundary_sides(tab_border_sides)
            .with_cell_boundary_color(SYSTEM_MENU_COLOR)
        })
        .collect();

    Table {
        columns,
        rows: vec![Row {
            height: VIDEO_TABS_ROW_HEIGHT,
        }],
    }
}

pub(super) fn video_footer_options_table() -> Table {
    let columns = [VIDEO_MENU_APPLY_TEXT, VIDEO_MENU_RESET_TEXT, VIDEO_MENU_BACK_TEXT]
        .iter()
        .map(|label| {
            Column::new(
                vec![Cell::new(TextContent::new(
                    (*label).to_string(),
                    SYSTEM_MENU_COLOR,
                    VIDEO_TABLE_TEXT_SIZE,
                ))],
                VIDEO_FOOTER_COLUMN_WIDTH,
                Vec2::new(10.0, 8.0),
                Anchor::CENTER,
                false,
            )
        })
        .collect();

    Table {
        columns,
        rows: vec![Row {
            height: VIDEO_FOOTER_ROW_HEIGHT,
        }],
    }
}

pub(super) fn display_mode_text(mode: VideoDisplayMode) -> &'static str {
    match mode {
        VideoDisplayMode::Windowed => "Windowed",
        VideoDisplayMode::Borderless => "Borderless",
        VideoDisplayMode::Fullscreen => "Fullscreen",
    }
}

pub(super) fn video_tab_kind(tab_index: usize) -> VideoTabKind {
    match tab_index {
        0 => VideoTabKind::Display,
        1 => VideoTabKind::Bloom,
        2 => VideoTabKind::Advanced,
        _ => VideoTabKind::Crt,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum VideoTopOptionKey {
    DisplayMode,
    Resolution,
    PresentMode,
    Msaa,
    WindowResizable,
    WindowDecorations,
    WindowTransparent,
    CompositeAlpha,
    BloomEnabled,
    BloomStyle,
    BloomIntensity,
    BloomScatter,
    BloomThreshold,
    BloomComposite,
    AnamorphicScale,
    BloomBoost,
    Tonemapping,
    DebandDither,
    FxaaEnabled,
    FxaaQuality,
    CasEnabled,
    CasStrength,
    ChromaticEnabled,
    ChromaticIntensity,
    CrtEnabled,
    ScanSpacing,
    ScanThickness,
    ScanDarkness,
    ScanJitter,
    ScanAberration,
    ScanPhosphor,
    ScanVignette,
}

pub(super) fn video_top_option_keys(tab: VideoTabKind) -> [VideoTopOptionKey; VIDEO_TOP_OPTION_COUNT] {
    match tab {
        VideoTabKind::Display => [
            VideoTopOptionKey::DisplayMode,
            VideoTopOptionKey::Resolution,
            VideoTopOptionKey::PresentMode,
            VideoTopOptionKey::Msaa,
            VideoTopOptionKey::WindowResizable,
            VideoTopOptionKey::WindowDecorations,
            VideoTopOptionKey::WindowTransparent,
            VideoTopOptionKey::CompositeAlpha,
        ],
        VideoTabKind::Bloom => [
            VideoTopOptionKey::BloomEnabled,
            VideoTopOptionKey::BloomStyle,
            VideoTopOptionKey::BloomIntensity,
            VideoTopOptionKey::BloomScatter,
            VideoTopOptionKey::BloomThreshold,
            VideoTopOptionKey::BloomComposite,
            VideoTopOptionKey::AnamorphicScale,
            VideoTopOptionKey::BloomBoost,
        ],
        VideoTabKind::Advanced => [
            VideoTopOptionKey::Tonemapping,
            VideoTopOptionKey::DebandDither,
            VideoTopOptionKey::FxaaEnabled,
            VideoTopOptionKey::FxaaQuality,
            VideoTopOptionKey::CasEnabled,
            VideoTopOptionKey::CasStrength,
            VideoTopOptionKey::ChromaticEnabled,
            VideoTopOptionKey::ChromaticIntensity,
        ],
        VideoTabKind::Crt => [
            VideoTopOptionKey::CrtEnabled,
            VideoTopOptionKey::ScanSpacing,
            VideoTopOptionKey::ScanThickness,
            VideoTopOptionKey::ScanDarkness,
            VideoTopOptionKey::ScanJitter,
            VideoTopOptionKey::ScanAberration,
            VideoTopOptionKey::ScanPhosphor,
            VideoTopOptionKey::ScanVignette,
        ],
    }
}

pub(super) fn video_top_option_key(tab: VideoTabKind, row: usize) -> Option<VideoTopOptionKey> {
    video_top_option_keys(tab).get(row).copied()
}

impl VideoTopOptionKey {
    pub(super) fn label(self) -> &'static str {
        match self {
            VideoTopOptionKey::DisplayMode => VIDEO_MENU_DISPLAY_MODE_TEXT,
            VideoTopOptionKey::Resolution => VIDEO_MENU_RESOLUTION_TEXT,
            VideoTopOptionKey::PresentMode => VIDEO_MENU_PRESENT_MODE_TEXT,
            VideoTopOptionKey::Msaa => VIDEO_MENU_MSAA_TEXT,
            VideoTopOptionKey::WindowResizable => VIDEO_MENU_RESIZABLE_TEXT,
            VideoTopOptionKey::WindowDecorations => VIDEO_MENU_DECORATIONS_TEXT,
            VideoTopOptionKey::WindowTransparent => VIDEO_MENU_TRANSPARENT_TEXT,
            VideoTopOptionKey::CompositeAlpha => VIDEO_MENU_COMPOSITE_ALPHA_TEXT,
            VideoTopOptionKey::BloomEnabled => VIDEO_MENU_BLOOM_ENABLED_TEXT,
            VideoTopOptionKey::BloomStyle => VIDEO_MENU_BLOOM_STYLE_TEXT,
            VideoTopOptionKey::BloomIntensity => VIDEO_MENU_BLOOM_INTENSITY_TEXT,
            VideoTopOptionKey::BloomScatter => VIDEO_MENU_BLOOM_SCATTER_TEXT,
            VideoTopOptionKey::BloomThreshold => VIDEO_MENU_BLOOM_THRESHOLD_TEXT,
            VideoTopOptionKey::BloomComposite => VIDEO_MENU_BLOOM_COMPOSITE_TEXT,
            VideoTopOptionKey::AnamorphicScale => VIDEO_MENU_ANAMORPHIC_TEXT,
            VideoTopOptionKey::BloomBoost => VIDEO_MENU_BLOOM_BOOST_TEXT,
            VideoTopOptionKey::Tonemapping => VIDEO_MENU_TONEMAP_TEXT,
            VideoTopOptionKey::DebandDither => VIDEO_MENU_DEBAND_TEXT,
            VideoTopOptionKey::FxaaEnabled => VIDEO_MENU_FXAA_TEXT,
            VideoTopOptionKey::FxaaQuality => VIDEO_MENU_FXAA_QUALITY_TEXT,
            VideoTopOptionKey::CasEnabled => VIDEO_MENU_CAS_TEXT,
            VideoTopOptionKey::CasStrength => VIDEO_MENU_CAS_STRENGTH_TEXT,
            VideoTopOptionKey::ChromaticEnabled => VIDEO_MENU_CHROMATIC_TEXT,
            VideoTopOptionKey::ChromaticIntensity => VIDEO_MENU_CHROMATIC_INTENSITY_TEXT,
            VideoTopOptionKey::CrtEnabled => VIDEO_MENU_CRT_ENABLED_TEXT,
            VideoTopOptionKey::ScanSpacing => VIDEO_MENU_SCAN_SPACING_TEXT,
            VideoTopOptionKey::ScanThickness => VIDEO_MENU_SCAN_THICKNESS_TEXT,
            VideoTopOptionKey::ScanDarkness => VIDEO_MENU_SCAN_DARKNESS_TEXT,
            VideoTopOptionKey::ScanJitter => VIDEO_MENU_SCAN_JITTER_TEXT,
            VideoTopOptionKey::ScanAberration => VIDEO_MENU_SCAN_ABERRATION_TEXT,
            VideoTopOptionKey::ScanPhosphor => VIDEO_MENU_SCAN_PHOSPHOR_TEXT,
            VideoTopOptionKey::ScanVignette => VIDEO_MENU_SCAN_VIGNETTE_TEXT,
        }
    }

    pub(super) fn description(self) -> &'static str {
        match self {
            VideoTopOptionKey::DisplayMode => {
                "Choose windowed, borderless fullscreen, or exclusive fullscreen mode."
            }
            VideoTopOptionKey::Resolution => {
                "Controls render pixel count. Higher looks sharper but can reduce performance."
            }
            VideoTopOptionKey::PresentMode => {
                "Controls monitor sync behavior, affecting tearing, smoothness, and latency."
            }
            VideoTopOptionKey::Msaa => {
                "MSAA anti-aliasing smooths jagged edges. Higher levels cost more performance."
            }
            VideoTopOptionKey::WindowResizable => {
                "If enabled, you can resize the game window while running."
            }
            VideoTopOptionKey::WindowDecorations => {
                "Show or hide the OS title bar and window buttons."
            }
            VideoTopOptionKey::WindowTransparent => {
                "Allow transparent window areas on platforms that support it."
            }
            VideoTopOptionKey::CompositeAlpha => {
                "How transparent window pixels blend with your desktop compositor."
            }
            VideoTopOptionKey::BloomEnabled => {
                "Bloom adds a soft glow around bright lights and highlights."
            }
            VideoTopOptionKey::BloomStyle => "Choose the visual style of bloom glow.",
            VideoTopOptionKey::BloomIntensity => "How strong the bloom glow appears.",
            VideoTopOptionKey::BloomScatter => "How far the bloom glow spreads outward.",
            VideoTopOptionKey::BloomThreshold => {
                "How bright a pixel must be before bloom is applied."
            }
            VideoTopOptionKey::BloomComposite => {
                "How bloom is combined back into the final image."
            }
            VideoTopOptionKey::AnamorphicScale => {
                "Stretches bloom sideways for a cinematic lens-streak look."
            }
            VideoTopOptionKey::BloomBoost => {
                "Adds extra weight to larger, softer bloom glow."
            }
            VideoTopOptionKey::Tonemapping => {
                "Converts HDR brightness into displayable colors while keeping detail."
            }
            VideoTopOptionKey::DebandDither => {
                "Adds subtle noise to reduce visible color banding in gradients."
            }
            VideoTopOptionKey::FxaaEnabled => {
                "FXAA smooths edge aliasing as a fast post-process filter."
            }
            VideoTopOptionKey::FxaaQuality => {
                "Higher FXAA quality smooths more edges but can soften detail."
            }
            VideoTopOptionKey::CasEnabled => {
                "CAS sharpens the image to recover detail after filtering."
            }
            VideoTopOptionKey::CasStrength => "How strong the CAS sharpening is.",
            VideoTopOptionKey::ChromaticEnabled => {
                "Adds lens-style color fringing near image edges."
            }
            VideoTopOptionKey::ChromaticIntensity => {
                "How strong the color fringing effect appears."
            }
            VideoTopOptionKey::CrtEnabled => "Master switch for all CRT screen effects.",
            VideoTopOptionKey::ScanSpacing => "Space between CRT-style scan lines.",
            VideoTopOptionKey::ScanThickness => "Thickness of each CRT scan line.",
            VideoTopOptionKey::ScanDarkness => "How dark the scan-line overlay appears.",
            VideoTopOptionKey::ScanJitter => "Amount of horizontal wobble and instability.",
            VideoTopOptionKey::ScanAberration => {
                "Amount of RGB channel separation (color bleed)."
            }
            VideoTopOptionKey::ScanPhosphor => {
                "Strength of phosphor mask pattern and glow response."
            }
            VideoTopOptionKey::ScanVignette => "How much the image darkens near edges.",
        }
    }

    pub(super) fn value_description(self, index: usize) -> Option<&'static str> {
        match self {
            VideoTopOptionKey::DisplayMode => match index {
                0 => Some("Runs in a regular window with borders and title bar."),
                1 => Some("Fullscreen-looking window with quick alt-tab behavior."),
                2 => Some("True fullscreen mode for maximum display control."),
                _ => None,
            },
            VideoTopOptionKey::Resolution => None,
            VideoTopOptionKey::PresentMode => match index {
                0 => Some("Game picks a synced mode automatically."),
                1 => Some("Game picks a low-latency unsynced mode automatically."),
                2 => Some("Classic VSync: smooth, no tearing, can add input delay."),
                3 => Some("VSync that may tear if frames arrive late."),
                4 => Some("Shows frames immediately: lowest latency, tearing likely."),
                5 => Some("Low-latency VSync with buffering when supported."),
                _ => None,
            },
            VideoTopOptionKey::CompositeAlpha => match index {
                0 => Some("Let the system choose the transparency blend mode."),
                1 => Some("Treat window as fully opaque (no transparency)."),
                2 => Some("Premultiplied alpha blend mode for transparent windows."),
                3 => Some("Straight alpha blend mode for transparent windows."),
                4 => Some("Use parent/window-system alpha mode when available."),
                _ => None,
            },
            VideoTopOptionKey::BloomStyle => match index {
                0 => Some("Balanced glow for a natural look."),
                1 => Some("Punchier retro-style glow."),
                2 => Some("Soft, blurrier glow."),
                _ => None,
            },
            VideoTopOptionKey::Tonemapping => match index {
                0 => Some("No tonemapping. Bright highlights may clip hard."),
                1 => Some("Simple classic filmic curve."),
                2 => Some("Reinhard variant that better preserves brightness."),
                3 => Some("Popular cinematic ACES-style look."),
                4 => Some("Modern filmic look with gentle roll-off."),
                5 => Some("Neutral transform with moderate contrast."),
                6 => Some("Stylized high-contrast artistic transform."),
                7 => Some("Blender Filmic curve for smooth highlights."),
                _ => None,
            },
            _ => None,
        }
    }

    pub(super) fn choice_count(self) -> usize {
        match self {
            VideoTopOptionKey::Resolution => RESOLUTIONS.len(),
            VideoTopOptionKey::DisplayMode => 3,
            VideoTopOptionKey::Msaa => 3,
            VideoTopOptionKey::PresentMode => 6,
            VideoTopOptionKey::CompositeAlpha => 5,
            VideoTopOptionKey::Tonemapping => 8,
            VideoTopOptionKey::FxaaQuality => 5,
            VideoTopOptionKey::WindowResizable
            | VideoTopOptionKey::WindowDecorations
            | VideoTopOptionKey::WindowTransparent
            | VideoTopOptionKey::BloomEnabled
            | VideoTopOptionKey::BloomComposite
            | VideoTopOptionKey::DebandDither
            | VideoTopOptionKey::FxaaEnabled
            | VideoTopOptionKey::CasEnabled
            | VideoTopOptionKey::ChromaticEnabled
            | VideoTopOptionKey::CrtEnabled => 2,
            VideoTopOptionKey::BloomStyle
            | VideoTopOptionKey::BloomIntensity
            | VideoTopOptionKey::BloomScatter
            | VideoTopOptionKey::BloomThreshold
            | VideoTopOptionKey::AnamorphicScale => 3,
            VideoTopOptionKey::BloomBoost
            | VideoTopOptionKey::CasStrength
            | VideoTopOptionKey::ChromaticIntensity => 4,
            VideoTopOptionKey::ScanSpacing
            | VideoTopOptionKey::ScanThickness
            | VideoTopOptionKey::ScanDarkness
            | VideoTopOptionKey::ScanJitter
            | VideoTopOptionKey::ScanAberration
            | VideoTopOptionKey::ScanPhosphor
            | VideoTopOptionKey::ScanVignette => 4,
        }
    }

    pub(super) fn uses_dropdown(self) -> bool {
        matches!(
            self,
            VideoTopOptionKey::DisplayMode
                | VideoTopOptionKey::Resolution
                | VideoTopOptionKey::PresentMode
                | VideoTopOptionKey::CompositeAlpha
                | VideoTopOptionKey::BloomStyle
                | VideoTopOptionKey::Tonemapping
        )
    }

    pub(super) fn slider_has_zero_state(self) -> bool {
        matches!(
            self,
            VideoTopOptionKey::Msaa
                | VideoTopOptionKey::BloomThreshold
                | VideoTopOptionKey::AnamorphicScale
                | VideoTopOptionKey::BloomBoost
                | VideoTopOptionKey::CasStrength
                | VideoTopOptionKey::ChromaticIntensity
                | VideoTopOptionKey::ScanSpacing
                | VideoTopOptionKey::ScanThickness
                | VideoTopOptionKey::ScanDarkness
                | VideoTopOptionKey::ScanJitter
                | VideoTopOptionKey::ScanAberration
                | VideoTopOptionKey::ScanPhosphor
                | VideoTopOptionKey::ScanVignette
        )
    }

    pub(super) fn slider_steps(self) -> Option<usize> {
        match self {
            VideoTopOptionKey::Msaa
            | VideoTopOptionKey::BloomIntensity
            | VideoTopOptionKey::BloomScatter
            | VideoTopOptionKey::BloomThreshold
            | VideoTopOptionKey::AnamorphicScale
            | VideoTopOptionKey::BloomBoost
            | VideoTopOptionKey::FxaaQuality
            | VideoTopOptionKey::CasStrength
            | VideoTopOptionKey::ChromaticIntensity
            | VideoTopOptionKey::ScanSpacing
            | VideoTopOptionKey::ScanThickness
            | VideoTopOptionKey::ScanDarkness
            | VideoTopOptionKey::ScanJitter
            | VideoTopOptionKey::ScanAberration
            | VideoTopOptionKey::ScanPhosphor
            | VideoTopOptionKey::ScanVignette => {
                let choice_count = self.choice_count();
                let slot_count = if self.slider_has_zero_state() {
                    choice_count.saturating_sub(1)
                } else {
                    choice_count
                };
                Some(slot_count.max(1))
            }
            _ => None,
        }
    }

    pub(super) fn uses_slider(self) -> bool {
        self.slider_steps().is_some()
    }

    pub(super) fn slider_filled_slots(self, snapshot: VideoSettingsSnapshot) -> Option<usize> {
        let slot_count = self.slider_steps()?;
        let selected_index = self.selected_index(snapshot);
        let filled = if self.slider_has_zero_state() {
            selected_index
        } else {
            selected_index.saturating_add(1)
        };
        Some(filled.min(slot_count))
    }

    pub(super) fn slider_selected_index_from_slot(self, slot_index: usize) -> Option<usize> {
        let slot_count = self.slider_steps()?;
        let max_selected = self.choice_count().saturating_sub(1);
        let clamped_slot = slot_index.min(slot_count.saturating_sub(1));
        let selected = if self.slider_has_zero_state() {
            clamped_slot.saturating_add(1)
        } else {
            clamped_slot
        };
        Some(selected.min(max_selected))
    }

    pub(super) fn slider_selected_index_from_local_x(
        self,
        local_x: f32,
        slot_width: f32,
        slot_gap: f32,
        layout_steps: usize,
    ) -> Option<usize> {
        let slot_count = self.slider_steps()?;
        let layout_steps = layout_steps.max(slot_count);
        let slot_width = slot_width.max(1.0);
        let slot_gap = slot_gap.max(0.0);
        let max_selected = self.choice_count().saturating_sub(1);

        let mut nearest_slot = 0usize;
        let mut nearest_distance = f32::INFINITY;
        for slot_index in 0..slot_count {
            let center = slot_center_x(slot_index, layout_steps, slot_width, slot_gap);
            let distance = (local_x - center).abs();
            if distance < nearest_distance {
                nearest_distance = distance;
                nearest_slot = slot_index;
            }
        }

        if self.slider_has_zero_state() {
            let first_center = slot_center_x(0, layout_steps, slot_width, slot_gap);
            let off_cutoff = first_center - VIDEO_DISCRETE_SLIDER_ZERO_CUTOFF_LEFT_BIAS_PX;
            if local_x <= off_cutoff {
                return Some(0);
            }
            return Some(nearest_slot.saturating_add(1).min(max_selected));
        }

        Some(nearest_slot.min(max_selected))
    }

    pub(super) fn values(self) -> Vec<String> {
        match self {
            VideoTopOptionKey::DisplayMode => vec![
                display_mode_text(VideoDisplayMode::Windowed).to_string(),
                display_mode_text(VideoDisplayMode::Borderless).to_string(),
                display_mode_text(VideoDisplayMode::Fullscreen).to_string(),
            ],
            VideoTopOptionKey::Resolution => RESOLUTIONS
                .iter()
                .map(|(w, h)| format!("{}x{}", *w as i32, *h as i32))
                .collect(),
            VideoTopOptionKey::PresentMode => vec![
                present_mode_text(VideoPresentMode::AutoVsync).to_string(),
                present_mode_text(VideoPresentMode::AutoNoVsync).to_string(),
                present_mode_text(VideoPresentMode::Fifo).to_string(),
                present_mode_text(VideoPresentMode::FifoRelaxed).to_string(),
                present_mode_text(VideoPresentMode::Immediate).to_string(),
                present_mode_text(VideoPresentMode::Mailbox).to_string(),
            ],
            VideoTopOptionKey::Msaa => vec![
                msaa_text(VideoMsaa::Off).to_string(),
                msaa_text(VideoMsaa::Sample2).to_string(),
                msaa_text(VideoMsaa::Sample4).to_string(),
            ],
            VideoTopOptionKey::WindowResizable => vec![
                toggle_text(true).to_string(),
                toggle_text(false).to_string(),
            ],
            VideoTopOptionKey::WindowDecorations => vec![
                toggle_text(true).to_string(),
                toggle_text(false).to_string(),
            ],
            VideoTopOptionKey::WindowTransparent => vec![
                toggle_text(true).to_string(),
                toggle_text(false).to_string(),
            ],
            VideoTopOptionKey::CompositeAlpha => vec![
                composite_alpha_text(VideoCompositeAlpha::Auto).to_string(),
                composite_alpha_text(VideoCompositeAlpha::Opaque).to_string(),
                composite_alpha_text(VideoCompositeAlpha::PreMultiplied).to_string(),
                composite_alpha_text(VideoCompositeAlpha::PostMultiplied).to_string(),
                composite_alpha_text(VideoCompositeAlpha::Inherit).to_string(),
            ],
            VideoTopOptionKey::BloomEnabled => vec![
                toggle_text(true).to_string(),
                toggle_text(false).to_string(),
            ],
            VideoTopOptionKey::BloomStyle => vec![
                bloom_style_text(BloomStyle::Natural).to_string(),
                bloom_style_text(BloomStyle::OldSchool).to_string(),
                bloom_style_text(BloomStyle::ScreenBlur).to_string(),
            ],
            VideoTopOptionKey::BloomIntensity => vec![
                bloom_intensity_text(BloomIntensity::Low).to_string(),
                bloom_intensity_text(BloomIntensity::Medium).to_string(),
                bloom_intensity_text(BloomIntensity::High).to_string(),
            ],
            VideoTopOptionKey::BloomScatter => vec![
                bloom_scatter_text(BloomScatter::Tight).to_string(),
                bloom_scatter_text(BloomScatter::Normal).to_string(),
                bloom_scatter_text(BloomScatter::Wide).to_string(),
            ],
            VideoTopOptionKey::BloomComposite => vec![
                bloom_composite_text(BloomComposite::EnergyConserving).to_string(),
                bloom_composite_text(BloomComposite::Additive).to_string(),
            ],
            VideoTopOptionKey::BloomThreshold => vec![
                bloom_threshold_text(BloomThreshold::Off).to_string(),
                bloom_threshold_text(BloomThreshold::Soft).to_string(),
                bloom_threshold_text(BloomThreshold::Hard).to_string(),
            ],
            VideoTopOptionKey::AnamorphicScale => vec![
                anamorphic_scale_text(AnamorphicScale::Off).to_string(),
                anamorphic_scale_text(AnamorphicScale::Wide).to_string(),
                anamorphic_scale_text(AnamorphicScale::UltraWide).to_string(),
            ],
            VideoTopOptionKey::BloomBoost => vec![
                bloom_boost_text(BloomBoost::Off).to_string(),
                bloom_boost_text(BloomBoost::Low).to_string(),
                bloom_boost_text(BloomBoost::Medium).to_string(),
                bloom_boost_text(BloomBoost::High).to_string(),
            ],
            VideoTopOptionKey::Tonemapping => vec![
                tonemapping_text(VideoTonemapping::None).to_string(),
                tonemapping_text(VideoTonemapping::Reinhard).to_string(),
                tonemapping_text(VideoTonemapping::ReinhardLuminance).to_string(),
                tonemapping_text(VideoTonemapping::AcesFitted).to_string(),
                tonemapping_text(VideoTonemapping::AgX).to_string(),
                tonemapping_text(VideoTonemapping::SomewhatBoringDisplayTransform).to_string(),
                tonemapping_text(VideoTonemapping::TonyMcMapface).to_string(),
                tonemapping_text(VideoTonemapping::BlenderFilmic).to_string(),
            ],
            VideoTopOptionKey::DebandDither => vec![
                toggle_text(true).to_string(),
                toggle_text(false).to_string(),
            ],
            VideoTopOptionKey::FxaaEnabled => vec![
                toggle_text(true).to_string(),
                toggle_text(false).to_string(),
            ],
            VideoTopOptionKey::FxaaQuality => vec![
                fxaa_quality_text(FxaaQuality::Low).to_string(),
                fxaa_quality_text(FxaaQuality::Medium).to_string(),
                fxaa_quality_text(FxaaQuality::High).to_string(),
                fxaa_quality_text(FxaaQuality::Ultra).to_string(),
                fxaa_quality_text(FxaaQuality::Extreme).to_string(),
            ],
            VideoTopOptionKey::CasEnabled => vec![
                toggle_text(true).to_string(),
                toggle_text(false).to_string(),
            ],
            VideoTopOptionKey::CasStrength => vec![
                cas_strength_text(CasStrength::Off).to_string(),
                cas_strength_text(CasStrength::Low).to_string(),
                cas_strength_text(CasStrength::Medium).to_string(),
                cas_strength_text(CasStrength::High).to_string(),
            ],
            VideoTopOptionKey::ChromaticEnabled => vec![
                toggle_text(true).to_string(),
                toggle_text(false).to_string(),
            ],
            VideoTopOptionKey::ChromaticIntensity => vec![
                crt_effect_level_text(CrtEffectLevel::Off).to_string(),
                crt_effect_level_text(CrtEffectLevel::Low).to_string(),
                crt_effect_level_text(CrtEffectLevel::Medium).to_string(),
                crt_effect_level_text(CrtEffectLevel::High).to_string(),
            ],
            VideoTopOptionKey::CrtEnabled => vec![
                toggle_text(true).to_string(),
                toggle_text(false).to_string(),
            ],
            VideoTopOptionKey::ScanSpacing => vec![
                scan_spacing_text(ScanSpacing::Off).to_string(),
                scan_spacing_text(ScanSpacing::Fine).to_string(),
                scan_spacing_text(ScanSpacing::Balanced).to_string(),
                scan_spacing_text(ScanSpacing::Sparse).to_string(),
            ],
            VideoTopOptionKey::ScanThickness => vec![
                scan_thickness_text(ScanThickness::Off).to_string(),
                scan_thickness_text(ScanThickness::Low).to_string(),
                scan_thickness_text(ScanThickness::Medium).to_string(),
                scan_thickness_text(ScanThickness::High).to_string(),
            ],
            VideoTopOptionKey::ScanDarkness => vec![
                scan_darkness_text(ScanDarkness::Off).to_string(),
                scan_darkness_text(ScanDarkness::Low).to_string(),
                scan_darkness_text(ScanDarkness::Medium).to_string(),
                scan_darkness_text(ScanDarkness::High).to_string(),
            ],
            VideoTopOptionKey::ScanJitter => vec![
                crt_effect_level_text(CrtEffectLevel::Off).to_string(),
                crt_effect_level_text(CrtEffectLevel::Low).to_string(),
                crt_effect_level_text(CrtEffectLevel::Medium).to_string(),
                crt_effect_level_text(CrtEffectLevel::High).to_string(),
            ],
            VideoTopOptionKey::ScanAberration => vec![
                crt_effect_level_text(CrtEffectLevel::Off).to_string(),
                crt_effect_level_text(CrtEffectLevel::Low).to_string(),
                crt_effect_level_text(CrtEffectLevel::Medium).to_string(),
                crt_effect_level_text(CrtEffectLevel::High).to_string(),
            ],
            VideoTopOptionKey::ScanPhosphor => vec![
                crt_effect_level_text(CrtEffectLevel::Off).to_string(),
                crt_effect_level_text(CrtEffectLevel::Low).to_string(),
                crt_effect_level_text(CrtEffectLevel::Medium).to_string(),
                crt_effect_level_text(CrtEffectLevel::High).to_string(),
            ],
            VideoTopOptionKey::ScanVignette => vec![
                crt_effect_level_text(CrtEffectLevel::Off).to_string(),
                crt_effect_level_text(CrtEffectLevel::Low).to_string(),
                crt_effect_level_text(CrtEffectLevel::Medium).to_string(),
                crt_effect_level_text(CrtEffectLevel::High).to_string(),
            ],
        }
    }

    pub(super) fn selected_index(self, snapshot: VideoSettingsSnapshot) -> usize {
        match self {
            VideoTopOptionKey::DisplayMode => match snapshot.display_mode {
                VideoDisplayMode::Windowed => 0,
                VideoDisplayMode::Borderless => 1,
                VideoDisplayMode::Fullscreen => 2,
            },
            VideoTopOptionKey::Resolution => {
                let max_index = RESOLUTIONS.len().saturating_sub(1);
                snapshot.resolution_index.min(max_index)
            }
            VideoTopOptionKey::PresentMode => match snapshot.present_mode {
                VideoPresentMode::AutoVsync => 0,
                VideoPresentMode::AutoNoVsync => 1,
                VideoPresentMode::Fifo => 2,
                VideoPresentMode::FifoRelaxed => 3,
                VideoPresentMode::Immediate => 4,
                VideoPresentMode::Mailbox => 5,
            },
            VideoTopOptionKey::Msaa => match snapshot.msaa {
                VideoMsaa::Off => 0,
                VideoMsaa::Sample2 => 1,
                VideoMsaa::Sample4 => 2,
            },
            VideoTopOptionKey::WindowResizable => {
                if snapshot.window_resizable { 0 } else { 1 }
            }
            VideoTopOptionKey::WindowDecorations => {
                if snapshot.window_decorations { 0 } else { 1 }
            }
            VideoTopOptionKey::WindowTransparent => {
                if snapshot.window_transparent { 0 } else { 1 }
            }
            VideoTopOptionKey::CompositeAlpha => match snapshot.composite_alpha {
                VideoCompositeAlpha::Auto => 0,
                VideoCompositeAlpha::Opaque => 1,
                VideoCompositeAlpha::PreMultiplied => 2,
                VideoCompositeAlpha::PostMultiplied => 3,
                VideoCompositeAlpha::Inherit => 4,
            },
            VideoTopOptionKey::BloomEnabled => {
                if snapshot.bloom_enabled { 0 } else { 1 }
            }
            VideoTopOptionKey::BloomStyle => match snapshot.bloom_style {
                BloomStyle::Natural => 0,
                BloomStyle::OldSchool => 1,
                BloomStyle::ScreenBlur => 2,
            },
            VideoTopOptionKey::BloomIntensity => match snapshot.bloom_intensity {
                BloomIntensity::Low => 0,
                BloomIntensity::Medium => 1,
                BloomIntensity::High => 2,
            },
            VideoTopOptionKey::BloomScatter => match snapshot.bloom_scatter {
                BloomScatter::Tight => 0,
                BloomScatter::Normal => 1,
                BloomScatter::Wide => 2,
            },
            VideoTopOptionKey::BloomComposite => match snapshot.bloom_composite {
                BloomComposite::EnergyConserving => 0,
                BloomComposite::Additive => 1,
            },
            VideoTopOptionKey::BloomThreshold => match snapshot.bloom_threshold {
                BloomThreshold::Off => 0,
                BloomThreshold::Soft => 1,
                BloomThreshold::Hard => 2,
            },
            VideoTopOptionKey::AnamorphicScale => match snapshot.anamorphic_scale {
                AnamorphicScale::Off => 0,
                AnamorphicScale::Wide => 1,
                AnamorphicScale::UltraWide => 2,
            },
            VideoTopOptionKey::BloomBoost => match snapshot.bloom_boost {
                BloomBoost::Off => 0,
                BloomBoost::Low => 1,
                BloomBoost::Medium => 2,
                BloomBoost::High => 3,
            },
            VideoTopOptionKey::Tonemapping => match snapshot.tonemapping {
                VideoTonemapping::None => 0,
                VideoTonemapping::Reinhard => 1,
                VideoTonemapping::ReinhardLuminance => 2,
                VideoTonemapping::AcesFitted => 3,
                VideoTonemapping::AgX => 4,
                VideoTonemapping::SomewhatBoringDisplayTransform => 5,
                VideoTonemapping::TonyMcMapface => 6,
                VideoTonemapping::BlenderFilmic => 7,
            },
            VideoTopOptionKey::DebandDither => {
                if snapshot.deband_dither_enabled { 0 } else { 1 }
            }
            VideoTopOptionKey::FxaaEnabled => {
                if snapshot.fxaa_enabled { 0 } else { 1 }
            }
            VideoTopOptionKey::FxaaQuality => match snapshot.fxaa_quality {
                FxaaQuality::Low => 0,
                FxaaQuality::Medium => 1,
                FxaaQuality::High => 2,
                FxaaQuality::Ultra => 3,
                FxaaQuality::Extreme => 4,
            },
            VideoTopOptionKey::CasEnabled => {
                if snapshot.cas_enabled { 0 } else { 1 }
            }
            VideoTopOptionKey::CasStrength => match snapshot.cas_strength {
                CasStrength::Off => 0,
                CasStrength::Low => 1,
                CasStrength::Medium => 2,
                CasStrength::High => 3,
            },
            VideoTopOptionKey::ChromaticEnabled => {
                if snapshot.chromatic_enabled { 0 } else { 1 }
            }
            VideoTopOptionKey::ChromaticIntensity => match snapshot.chromatic_intensity {
                CrtEffectLevel::Off => 0,
                CrtEffectLevel::Low => 1,
                CrtEffectLevel::Medium => 2,
                CrtEffectLevel::High => 3,
            },
            VideoTopOptionKey::CrtEnabled => {
                if snapshot.crt_enabled { 0 } else { 1 }
            }
            VideoTopOptionKey::ScanSpacing => match snapshot.scan_spacing {
                ScanSpacing::Off => 0,
                ScanSpacing::Fine => 1,
                ScanSpacing::Balanced => 2,
                ScanSpacing::Sparse => 3,
            },
            VideoTopOptionKey::ScanThickness => match snapshot.scan_thickness {
                ScanThickness::Off => 0,
                ScanThickness::Low => 1,
                ScanThickness::Medium => 2,
                ScanThickness::High => 3,
            },
            VideoTopOptionKey::ScanDarkness => match snapshot.scan_darkness {
                ScanDarkness::Off => 0,
                ScanDarkness::Low => 1,
                ScanDarkness::Medium => 2,
                ScanDarkness::High => 3,
            },
            VideoTopOptionKey::ScanJitter => match snapshot.scan_jitter {
                CrtEffectLevel::Off => 0,
                CrtEffectLevel::Low => 1,
                CrtEffectLevel::Medium => 2,
                CrtEffectLevel::High => 3,
            },
            VideoTopOptionKey::ScanAberration => match snapshot.scan_aberration {
                CrtEffectLevel::Off => 0,
                CrtEffectLevel::Low => 1,
                CrtEffectLevel::Medium => 2,
                CrtEffectLevel::High => 3,
            },
            VideoTopOptionKey::ScanPhosphor => match snapshot.scan_phosphor {
                CrtEffectLevel::Off => 0,
                CrtEffectLevel::Low => 1,
                CrtEffectLevel::Medium => 2,
                CrtEffectLevel::High => 3,
            },
            VideoTopOptionKey::ScanVignette => match snapshot.scan_vignette {
                CrtEffectLevel::Off => 0,
                CrtEffectLevel::Low => 1,
                CrtEffectLevel::Medium => 2,
                CrtEffectLevel::High => 3,
            },
        }
    }

    pub(super) fn apply_selected_index(
        self,
        snapshot: &mut VideoSettingsSnapshot,
        index: usize,
    ) -> bool {
        match self {
            VideoTopOptionKey::DisplayMode => {
                let next = match index {
                    0 => VideoDisplayMode::Windowed,
                    1 => VideoDisplayMode::Borderless,
                    _ => VideoDisplayMode::Fullscreen,
                };
                if snapshot.display_mode == next {
                    false
                } else {
                    snapshot.display_mode = next;
                    true
                }
            }
            VideoTopOptionKey::Resolution => {
                let max_index = RESOLUTIONS.len().saturating_sub(1);
                let clamped = index.min(max_index);
                if snapshot.resolution_index == clamped {
                    false
                } else {
                    snapshot.resolution_index = clamped;
                    true
                }
            }
            VideoTopOptionKey::PresentMode => {
                let next = match index {
                    0 => VideoPresentMode::AutoVsync,
                    1 => VideoPresentMode::AutoNoVsync,
                    2 => VideoPresentMode::Fifo,
                    3 => VideoPresentMode::FifoRelaxed,
                    4 => VideoPresentMode::Immediate,
                    _ => VideoPresentMode::Mailbox,
                };
                if snapshot.present_mode == next {
                    false
                } else {
                    snapshot.present_mode = next;
                    true
                }
            }
            VideoTopOptionKey::Msaa => {
                let next = match index {
                    0 => VideoMsaa::Off,
                    1 => VideoMsaa::Sample2,
                    _ => VideoMsaa::Sample4,
                };
                if snapshot.msaa == next {
                    false
                } else {
                    snapshot.msaa = next;
                    true
                }
            }
            VideoTopOptionKey::WindowResizable => {
                let next = index == 0;
                if snapshot.window_resizable == next {
                    false
                } else {
                    snapshot.window_resizable = next;
                    true
                }
            }
            VideoTopOptionKey::WindowDecorations => {
                let next = index == 0;
                if snapshot.window_decorations == next {
                    false
                } else {
                    snapshot.window_decorations = next;
                    true
                }
            }
            VideoTopOptionKey::WindowTransparent => {
                let next = index == 0;
                if snapshot.window_transparent == next {
                    false
                } else {
                    snapshot.window_transparent = next;
                    true
                }
            }
            VideoTopOptionKey::CompositeAlpha => {
                let next = match index {
                    0 => VideoCompositeAlpha::Auto,
                    1 => VideoCompositeAlpha::Opaque,
                    2 => VideoCompositeAlpha::PreMultiplied,
                    3 => VideoCompositeAlpha::PostMultiplied,
                    _ => VideoCompositeAlpha::Inherit,
                };
                if snapshot.composite_alpha == next {
                    false
                } else {
                    snapshot.composite_alpha = next;
                    true
                }
            }
            VideoTopOptionKey::BloomEnabled => {
                let next = index == 0;
                if snapshot.bloom_enabled == next {
                    false
                } else {
                    snapshot.bloom_enabled = next;
                    true
                }
            }
            VideoTopOptionKey::BloomStyle => {
                let next = match index {
                    0 => BloomStyle::Natural,
                    1 => BloomStyle::OldSchool,
                    _ => BloomStyle::ScreenBlur,
                };
                if snapshot.bloom_style == next {
                    false
                } else {
                    snapshot.bloom_style = next;
                    true
                }
            }
            VideoTopOptionKey::BloomIntensity => {
                let next = match index {
                    0 => BloomIntensity::Low,
                    1 => BloomIntensity::Medium,
                    _ => BloomIntensity::High,
                };
                if snapshot.bloom_intensity == next {
                    false
                } else {
                    snapshot.bloom_intensity = next;
                    true
                }
            }
            VideoTopOptionKey::BloomScatter => {
                let next = match index {
                    0 => BloomScatter::Tight,
                    1 => BloomScatter::Normal,
                    _ => BloomScatter::Wide,
                };
                if snapshot.bloom_scatter == next {
                    false
                } else {
                    snapshot.bloom_scatter = next;
                    true
                }
            }
            VideoTopOptionKey::BloomComposite => {
                let next = if index == 0 {
                    BloomComposite::EnergyConserving
                } else {
                    BloomComposite::Additive
                };
                if snapshot.bloom_composite == next {
                    false
                } else {
                    snapshot.bloom_composite = next;
                    true
                }
            }
            VideoTopOptionKey::BloomThreshold => {
                let next = match index {
                    0 => BloomThreshold::Off,
                    1 => BloomThreshold::Soft,
                    _ => BloomThreshold::Hard,
                };
                if snapshot.bloom_threshold == next {
                    false
                } else {
                    snapshot.bloom_threshold = next;
                    true
                }
            }
            VideoTopOptionKey::AnamorphicScale => {
                let next = match index {
                    0 => AnamorphicScale::Off,
                    1 => AnamorphicScale::Wide,
                    _ => AnamorphicScale::UltraWide,
                };
                if snapshot.anamorphic_scale == next {
                    false
                } else {
                    snapshot.anamorphic_scale = next;
                    true
                }
            }
            VideoTopOptionKey::BloomBoost => {
                let next = match index {
                    0 => BloomBoost::Off,
                    1 => BloomBoost::Low,
                    2 => BloomBoost::Medium,
                    _ => BloomBoost::High,
                };
                if snapshot.bloom_boost == next {
                    false
                } else {
                    snapshot.bloom_boost = next;
                    true
                }
            }
            VideoTopOptionKey::Tonemapping => {
                let next = match index {
                    0 => VideoTonemapping::None,
                    1 => VideoTonemapping::Reinhard,
                    2 => VideoTonemapping::ReinhardLuminance,
                    3 => VideoTonemapping::AcesFitted,
                    4 => VideoTonemapping::AgX,
                    5 => VideoTonemapping::SomewhatBoringDisplayTransform,
                    6 => VideoTonemapping::TonyMcMapface,
                    _ => VideoTonemapping::BlenderFilmic,
                };
                if snapshot.tonemapping == next {
                    false
                } else {
                    snapshot.tonemapping = next;
                    true
                }
            }
            VideoTopOptionKey::DebandDither => {
                let next = index == 0;
                if snapshot.deband_dither_enabled == next {
                    false
                } else {
                    snapshot.deband_dither_enabled = next;
                    true
                }
            }
            VideoTopOptionKey::FxaaEnabled => {
                let next = index == 0;
                if snapshot.fxaa_enabled == next {
                    false
                } else {
                    snapshot.fxaa_enabled = next;
                    true
                }
            }
            VideoTopOptionKey::FxaaQuality => {
                let next = match index {
                    0 => FxaaQuality::Low,
                    1 => FxaaQuality::Medium,
                    2 => FxaaQuality::High,
                    3 => FxaaQuality::Ultra,
                    _ => FxaaQuality::Extreme,
                };
                if snapshot.fxaa_quality == next {
                    false
                } else {
                    snapshot.fxaa_quality = next;
                    true
                }
            }
            VideoTopOptionKey::CasEnabled => {
                let next = index == 0;
                if snapshot.cas_enabled == next {
                    false
                } else {
                    snapshot.cas_enabled = next;
                    true
                }
            }
            VideoTopOptionKey::CasStrength => {
                let next = match index {
                    0 => CasStrength::Off,
                    1 => CasStrength::Low,
                    2 => CasStrength::Medium,
                    _ => CasStrength::High,
                };
                if snapshot.cas_strength == next {
                    false
                } else {
                    snapshot.cas_strength = next;
                    true
                }
            }
            VideoTopOptionKey::ChromaticEnabled => {
                let next = index == 0;
                if snapshot.chromatic_enabled == next {
                    false
                } else {
                    snapshot.chromatic_enabled = next;
                    true
                }
            }
            VideoTopOptionKey::ChromaticIntensity => {
                let next = match index {
                    0 => CrtEffectLevel::Off,
                    1 => CrtEffectLevel::Low,
                    2 => CrtEffectLevel::Medium,
                    _ => CrtEffectLevel::High,
                };
                if snapshot.chromatic_intensity == next {
                    false
                } else {
                    snapshot.chromatic_intensity = next;
                    true
                }
            }
            VideoTopOptionKey::CrtEnabled => {
                let next = index == 0;
                if snapshot.crt_enabled == next {
                    false
                } else {
                    snapshot.crt_enabled = next;
                    true
                }
            }
            VideoTopOptionKey::ScanSpacing => {
                let next = match index {
                    0 => ScanSpacing::Off,
                    1 => ScanSpacing::Fine,
                    2 => ScanSpacing::Balanced,
                    _ => ScanSpacing::Sparse,
                };
                if snapshot.scan_spacing == next {
                    false
                } else {
                    snapshot.scan_spacing = next;
                    true
                }
            }
            VideoTopOptionKey::ScanThickness => {
                let next = match index {
                    0 => ScanThickness::Off,
                    1 => ScanThickness::Low,
                    2 => ScanThickness::Medium,
                    _ => ScanThickness::High,
                };
                if snapshot.scan_thickness == next {
                    false
                } else {
                    snapshot.scan_thickness = next;
                    true
                }
            }
            VideoTopOptionKey::ScanDarkness => {
                let next = match index {
                    0 => ScanDarkness::Off,
                    1 => ScanDarkness::Low,
                    2 => ScanDarkness::Medium,
                    _ => ScanDarkness::High,
                };
                if snapshot.scan_darkness == next {
                    false
                } else {
                    snapshot.scan_darkness = next;
                    true
                }
            }
            VideoTopOptionKey::ScanJitter => {
                let next = match index {
                    0 => CrtEffectLevel::Off,
                    1 => CrtEffectLevel::Low,
                    2 => CrtEffectLevel::Medium,
                    _ => CrtEffectLevel::High,
                };
                if snapshot.scan_jitter == next {
                    false
                } else {
                    snapshot.scan_jitter = next;
                    true
                }
            }
            VideoTopOptionKey::ScanAberration => {
                let next = match index {
                    0 => CrtEffectLevel::Off,
                    1 => CrtEffectLevel::Low,
                    2 => CrtEffectLevel::Medium,
                    _ => CrtEffectLevel::High,
                };
                if snapshot.scan_aberration == next {
                    false
                } else {
                    snapshot.scan_aberration = next;
                    true
                }
            }
            VideoTopOptionKey::ScanPhosphor => {
                let next = match index {
                    0 => CrtEffectLevel::Off,
                    1 => CrtEffectLevel::Low,
                    2 => CrtEffectLevel::Medium,
                    _ => CrtEffectLevel::High,
                };
                if snapshot.scan_phosphor == next {
                    false
                } else {
                    snapshot.scan_phosphor = next;
                    true
                }
            }
            VideoTopOptionKey::ScanVignette => {
                let next = match index {
                    0 => CrtEffectLevel::Off,
                    1 => CrtEffectLevel::Low,
                    2 => CrtEffectLevel::Medium,
                    _ => CrtEffectLevel::High,
                };
                if snapshot.scan_vignette == next {
                    false
                } else {
                    snapshot.scan_vignette = next;
                    true
                }
            }
        }
    }

    pub(super) fn cycle(self, snapshot: &mut VideoSettingsSnapshot, forward: bool) -> bool {
        let count = self.choice_count();
        if count == 0 {
            return false;
        }

        let current = self.selected_index(*snapshot).min(count - 1);
        let next = if forward {
            (current + 1) % count
        } else {
            (current + count - 1) % count
        };
        self.apply_selected_index(snapshot, next)
    }

    pub(super) fn value_text(self, snapshot: VideoSettingsSnapshot) -> String {
        let values = self.values();
        if values.is_empty() {
            return String::new();
        }
        let index = self.selected_index(snapshot).min(values.len() - 1);
        values[index].clone()
    }
}

pub(super) fn video_top_option_labels(tab: VideoTabKind) -> [&'static str; VIDEO_TOP_OPTION_COUNT] {
    let keys = video_top_option_keys(tab);
    [
        keys[0].label(),
        keys[1].label(),
        keys[2].label(),
        keys[3].label(),
        keys[4].label(),
        keys[5].label(),
        keys[6].label(),
        keys[7].label(),
    ]
}

pub(super) fn video_top_option_choice_count(tab: VideoTabKind, row: usize) -> usize {
    video_top_option_key(tab, row)
        .map(VideoTopOptionKey::choice_count)
        .unwrap_or(0)
}

pub(super) fn video_top_option_uses_dropdown(tab: VideoTabKind, row: usize) -> bool {
    video_top_option_key(tab, row)
        .is_some_and(VideoTopOptionKey::uses_dropdown)
}

pub(super) fn video_top_option_uses_slider(tab: VideoTabKind, row: usize) -> bool {
    video_top_option_key(tab, row)
        .is_some_and(VideoTopOptionKey::uses_slider)
}

pub(super) fn video_top_option_values(tab: VideoTabKind, row: usize) -> Vec<String> {
    video_top_option_key(tab, row)
        .map(VideoTopOptionKey::values)
        .unwrap_or_default()
}

pub(super) fn video_top_row_for_command(command: &MenuCommand) -> Option<usize> {
    match command {
        MenuCommand::ToggleVideoTopOption(row) if *row < VIDEO_TOP_OPTION_COUNT => Some(*row),
        _ => None,
    }
}

pub(super) fn video_selector_arrow_positions() -> (f32, f32) {
    (
        VIDEO_VALUE_COLUMN_CENTER_X - VIDEO_SELECTOR_ARROW_SPREAD,
        VIDEO_VALUE_COLUMN_CENTER_X + VIDEO_SELECTOR_ARROW_SPREAD,
    )
}

pub(super) fn video_slider_cycle_arrow_positions(steps: usize, slot_width: f32) -> (f32, f32) {
    let steps = steps.max(1);
    let layout_steps = steps.max(VIDEO_DISCRETE_SLIDER_MAX_STEPS);
    let (left_edge, right_edge) =
        slot_span_bounds(steps, layout_steps, slot_width, VIDEO_DISCRETE_SLIDER_GAP);
    (
        VIDEO_DISCRETE_SLIDER_ROOT_X
            + left_edge
            - VIDEO_SLIDER_ARROW_MARGIN
            - CYCLE_ARROW_HALF_WIDTH,
        VIDEO_DISCRETE_SLIDER_ROOT_X
            + right_edge
            + VIDEO_SLIDER_ARROW_MARGIN
            + CYCLE_ARROW_HALF_WIDTH,
    )
}

pub(super) fn video_value_cycle_arrow_positions(key: Option<VideoTopOptionKey>) -> (f32, f32) {
    if let Some(key) = key.filter(|key| key.uses_slider()) {
        let steps = key.slider_steps().unwrap_or(1);
        video_slider_cycle_arrow_positions(steps, VIDEO_DISCRETE_SLIDER_SLOT_SIZE_SELECTED.x)
    } else {
        video_selector_arrow_positions()
    }
}

pub(super) fn video_top_option_selected_index(
    snapshot: VideoSettingsSnapshot,
    tab: VideoTabKind,
    row: usize,
) -> Option<usize> {
    video_top_option_key(tab, row).map(|key| key.selected_index(snapshot))
}

pub(super) fn apply_video_top_option_selected_index(
    snapshot: &mut VideoSettingsSnapshot,
    tab: VideoTabKind,
    row: usize,
    index: usize,
) -> bool {
    video_top_option_key(tab, row)
        .map(|key| key.apply_selected_index(snapshot, index))
        .unwrap_or(false)
}

pub(super) fn video_top_value_strings(
    snapshot: VideoSettingsSnapshot,
    tab: VideoTabKind,
) -> [String; VIDEO_TOP_OPTION_COUNT] {
    let keys = video_top_option_keys(tab);
    [
        keys[0].value_text(snapshot),
        keys[1].value_text(snapshot),
        keys[2].value_text(snapshot),
        keys[3].value_text(snapshot),
        keys[4].value_text(snapshot),
        keys[5].value_text(snapshot),
        keys[6].value_text(snapshot),
        keys[7].value_text(snapshot),
    ]
}

pub(super) fn bloom_style_text(style: BloomStyle) -> &'static str {
    match style {
        BloomStyle::Natural => "Natural",
        BloomStyle::OldSchool => "Old School",
        BloomStyle::ScreenBlur => "Screen Blur",
    }
}

pub(super) fn toggle_text(enabled: bool) -> &'static str {
    if enabled {
        "On"
    } else {
        "Off"
    }
}

pub(super) fn present_mode_text(present_mode: VideoPresentMode) -> &'static str {
    match present_mode {
        VideoPresentMode::AutoVsync => "Auto Vsync",
        VideoPresentMode::AutoNoVsync => "Auto",
        VideoPresentMode::Fifo => "Fifo",
        VideoPresentMode::FifoRelaxed => "Fifo Relaxed",
        VideoPresentMode::Immediate => "Immediate",
        VideoPresentMode::Mailbox => "Mailbox",
    }
}

pub(super) fn msaa_text(msaa: VideoMsaa) -> &'static str {
    match msaa {
        VideoMsaa::Off => "Off",
        VideoMsaa::Sample2 => "2x",
        VideoMsaa::Sample4 => "4x",
    }
}

pub(super) fn composite_alpha_text(mode: VideoCompositeAlpha) -> &'static str {
    match mode {
        VideoCompositeAlpha::Auto => "Auto",
        VideoCompositeAlpha::Opaque => "Opaque",
        VideoCompositeAlpha::PreMultiplied => "PreMul",
        VideoCompositeAlpha::PostMultiplied => "PostMul",
        VideoCompositeAlpha::Inherit => "Inherit",
    }
}

pub(super) fn bloom_intensity_text(intensity: BloomIntensity) -> &'static str {
    match intensity {
        BloomIntensity::Low => "Low",
        BloomIntensity::Medium => "Medium",
        BloomIntensity::High => "High",
    }
}

pub(super) fn bloom_scatter_text(scatter: BloomScatter) -> &'static str {
    match scatter {
        BloomScatter::Tight => "Tight",
        BloomScatter::Normal => "Normal",
        BloomScatter::Wide => "Wide",
    }
}

pub(super) fn bloom_composite_text(composite: BloomComposite) -> &'static str {
    match composite {
        BloomComposite::EnergyConserving => "Energy",
        BloomComposite::Additive => "Additive",
    }
}

pub(super) fn bloom_threshold_text(threshold: BloomThreshold) -> &'static str {
    match threshold {
        BloomThreshold::Off => "Off",
        BloomThreshold::Soft => "Soft",
        BloomThreshold::Hard => "Hard",
    }
}

pub(super) fn anamorphic_scale_text(scale: AnamorphicScale) -> &'static str {
    match scale {
        AnamorphicScale::Off => "Off",
        AnamorphicScale::Wide => "Wide",
        AnamorphicScale::UltraWide => "Ultra",
    }
}

pub(super) fn bloom_boost_text(boost: BloomBoost) -> &'static str {
    match boost {
        BloomBoost::Off => "Off",
        BloomBoost::Low => "Low",
        BloomBoost::Medium => "Medium",
        BloomBoost::High => "High",
    }
}

pub(super) fn tonemapping_text(tonemapping: VideoTonemapping) -> &'static str {
    match tonemapping {
        VideoTonemapping::None => "None",
        VideoTonemapping::Reinhard => "Reinhard",
        VideoTonemapping::ReinhardLuminance => "Reinhard Lum",
        VideoTonemapping::AcesFitted => "Aces Fitted",
        VideoTonemapping::AgX => "AgX",
        VideoTonemapping::SomewhatBoringDisplayTransform => "SBDT",
        VideoTonemapping::TonyMcMapface => "TonyMcMap",
        VideoTonemapping::BlenderFilmic => "Filmic",
    }
}

pub(super) fn fxaa_quality_text(quality: FxaaQuality) -> &'static str {
    match quality {
        FxaaQuality::Low => "Low",
        FxaaQuality::Medium => "Medium",
        FxaaQuality::High => "High",
        FxaaQuality::Ultra => "Ultra",
        FxaaQuality::Extreme => "Extreme",
    }
}

pub(super) fn cas_strength_text(strength: CasStrength) -> &'static str {
    match strength {
        CasStrength::Off => "Off",
        CasStrength::Low => "Low",
        CasStrength::Medium => "Medium",
        CasStrength::High => "High",
    }
}

pub(super) fn crt_effect_level_text(level: CrtEffectLevel) -> &'static str {
    match level {
        CrtEffectLevel::Off => "Off",
        CrtEffectLevel::Low => "Low",
        CrtEffectLevel::Medium => "Medium",
        CrtEffectLevel::High => "High",
    }
}

pub(super) fn scan_spacing_text(spacing: ScanSpacing) -> &'static str {
    match spacing {
        ScanSpacing::Off => "Off",
        ScanSpacing::Fine => "Fine",
        ScanSpacing::Balanced => "Balanced",
        ScanSpacing::Sparse => "Sparse",
    }
}

pub(super) fn scan_thickness_text(thickness: ScanThickness) -> &'static str {
    match thickness {
        ScanThickness::Off => "Off",
        ScanThickness::Low => "Low",
        ScanThickness::Medium => "Medium",
        ScanThickness::High => "High",
    }
}

pub(super) fn scan_darkness_text(darkness: ScanDarkness) -> &'static str {
    match darkness {
        ScanDarkness::Off => "Off",
        ScanDarkness::Low => "Low",
        ScanDarkness::Medium => "Medium",
        ScanDarkness::High => "High",
    }
}

pub(super) fn step_video_top_option_clamped(
    snapshot: &mut VideoSettingsSnapshot,
    tab: VideoTabKind,
    row: usize,
    forward: bool,
) -> bool {
    let Some(key) = video_top_option_key(tab, row) else {
        return false;
    };
    let count = key.choice_count();
    if count == 0 {
        return false;
    }
    let current = key.selected_index(*snapshot).min(count - 1);
    let next = if forward {
        (current + 1).min(count - 1)
    } else {
        current.saturating_sub(1)
    };
    if next == current {
        return false;
    }
    key.apply_selected_index(snapshot, next)
}

pub(super) fn step_video_top_option_for_input(
    snapshot: &mut VideoSettingsSnapshot,
    tab: VideoTabKind,
    row: usize,
    forward: bool,
) -> bool {
    let Some(key) = video_top_option_key(tab, row) else {
        return false;
    };
    if key.uses_slider() {
        step_video_top_option_clamped(snapshot, tab, row, forward)
    } else {
        key.cycle(snapshot, forward)
    }
}

pub(super) fn video_top_row_center_y(index: usize) -> f32 {
    VIDEO_TABLE_Y - (index as f32 + 0.5) * VIDEO_TABLE_ROW_HEIGHT
}

pub(super) fn video_top_scroll_local_y(world_y: f32) -> f32 {
    world_y - VIDEO_TOP_SCROLL_CENTER_Y
}

pub(super) fn video_top_row_top_y(index: usize) -> f32 {
    VIDEO_TABLE_Y - index as f32 * VIDEO_TABLE_ROW_HEIGHT
}

pub(super) fn video_tabs_row_center_y() -> f32 {
    VIDEO_TABS_TABLE_Y - 0.5 * VIDEO_TABS_ROW_HEIGHT
}

pub(super) fn video_tab_column_center_x(index: usize) -> f32 {
    let tab_column_width = VIDEO_TABLE_TOTAL_WIDTH / VIDEO_TABS.len() as f32;
    VIDEO_TABLE_X + tab_column_width * (index as f32 + 0.5)
}

pub(super) fn video_footer_row_center_y() -> f32 {
    VIDEO_FOOTER_TABLE_Y - 0.5 * VIDEO_FOOTER_ROW_HEIGHT
}

pub(super) fn video_footer_column_center_x(index: usize) -> f32 {
    VIDEO_TABLE_X + VIDEO_FOOTER_COLUMN_WIDTH * (index as f32 + 0.5)
}

pub(super) fn video_option_position(index: usize) -> Vec2 {
    if index < VIDEO_TOP_OPTION_COUNT {
        Vec2::new(VIDEO_OPTION_SELECTOR_X, video_top_row_center_y(index))
    } else {
        let footer_index = index - VIDEO_FOOTER_OPTION_START_INDEX;
        Vec2::new(
            video_footer_column_center_x(footer_index),
            video_footer_row_center_y(),
        )
    }
}

pub(super) fn video_option_region(index: usize) -> Vec2 {
    if index < VIDEO_TOP_OPTION_COUNT {
        Vec2::new(VIDEO_OPTION_REGION_WIDTH, VIDEO_OPTION_REGION_HEIGHT)
    } else {
        Vec2::new(
            VIDEO_FOOTER_OPTION_REGION_WIDTH,
            VIDEO_FOOTER_OPTION_REGION_HEIGHT,
        )
    }
}

pub(super) fn dropdown_center_y_from_row_top(row_index: usize, num_rows: usize) -> f32 {
    let dropdown_height = num_rows as f32 * VIDEO_RESOLUTION_DROPDOWN_ROW_HEIGHT;
    video_top_row_top_y(row_index) - dropdown_height * 0.5
}

pub(super) fn dropdown_item_local_center_y(index: usize, num_rows: usize) -> f32 {
    let dropdown_height = num_rows as f32 * VIDEO_RESOLUTION_DROPDOWN_ROW_HEIGHT;
    dropdown_height * 0.5 - (index as f32 + 0.5) * VIDEO_RESOLUTION_DROPDOWN_ROW_HEIGHT
}

pub(super) fn dropdown_digit_shortcut_index(keyboard_input: &ButtonInput<KeyCode>) -> Option<usize> {
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
            let selected_index = if keycode == KeyCode::Digit0 {
                9
            } else {
                digit_index
            };
            return Some(selected_index);
        }
    }

    None
}

pub(super) fn video_resolution_option_index() -> usize {
    VIDEO_MENU_OPTIONS
        .iter()
        .position(|option| {
            matches!(
                option.command,
                MenuCommand::ToggleVideoTopOption(row) if row == VIDEO_RESOLUTION_OPTION_INDEX
            )
        })
        .unwrap_or(VIDEO_RESOLUTION_OPTION_INDEX)
}

pub(super) fn default_video_settings() -> VideoSettingsSnapshot {
    VideoSettingsSnapshot::default()
}

pub(super) fn video_settings_dirty(state: &VideoSettingsState) -> bool {
    state.pending != state.saved
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn video_top_rows_are_derived_from_menu_option_defs() {
        assert_eq!(
            video_top_row_for_command(&MenuCommand::ToggleVideoTopOption(0)),
            Some(0)
        );
        assert_eq!(
            video_top_row_for_command(&MenuCommand::ToggleVideoTopOption(1)),
            Some(1)
        );
        assert_eq!(
            video_top_row_for_command(&MenuCommand::ToggleVideoTopOption(7)),
            Some(7)
        );
        assert_eq!(
            video_top_row_for_command(&MenuCommand::ApplyVideoSettings),
            None
        );
    }

    #[test]
    fn zero_state_slider_uses_zero_filled_slots_and_first_slot_maps_to_one() {
        let mut snapshot = default_video_settings();
        snapshot.scan_spacing = ScanSpacing::Off;
        let key = VideoTopOptionKey::ScanSpacing;

        assert_eq!(key.slider_filled_slots(snapshot), Some(0));
        assert_eq!(key.slider_selected_index_from_slot(0), Some(1));
    }

    #[test]
    fn non_zero_slider_maps_slot_to_matching_index() {
        let key = VideoTopOptionKey::BloomIntensity;
        assert_eq!(key.slider_selected_index_from_slot(0), Some(0));
        assert_eq!(key.slider_selected_index_from_slot(1), Some(1));
        assert_eq!(key.slider_selected_index_from_slot(2), Some(2));
    }

    #[test]
    fn zero_state_slider_local_x_maps_left_half_to_off() {
        let key = VideoTopOptionKey::ScanSpacing;
        let first_center = slot_center_x(0, 4, 22.0, VIDEO_DISCRETE_SLIDER_GAP);
        let cutoff = first_center - VIDEO_DISCRETE_SLIDER_ZERO_CUTOFF_LEFT_BIAS_PX;

        assert_eq!(
            key.slider_selected_index_from_local_x(first_center - 6.0, 22.0, VIDEO_DISCRETE_SLIDER_GAP, 4),
            Some(0)
        );
        assert_eq!(
            key.slider_selected_index_from_local_x(first_center + 6.0, 22.0, VIDEO_DISCRETE_SLIDER_GAP, 4),
            Some(1)
        );
        assert_eq!(
            key.slider_selected_index_from_local_x(cutoff + 0.25, 22.0, VIDEO_DISCRETE_SLIDER_GAP, 4),
            Some(1)
        );
    }

    #[test]
    fn non_zero_slider_local_x_uses_midpoints_between_slots() {
        let key = VideoTopOptionKey::BloomIntensity;
        let c0 = slot_center_x(0, 4, 22.0, VIDEO_DISCRETE_SLIDER_GAP);
        let c1 = slot_center_x(1, 4, 22.0, VIDEO_DISCRETE_SLIDER_GAP);
        let midpoint = (c0 + c1) * 0.5;

        assert_eq!(
            key.slider_selected_index_from_local_x(midpoint - 0.25, 22.0, VIDEO_DISCRETE_SLIDER_GAP, 4),
            Some(0)
        );
        assert_eq!(
            key.slider_selected_index_from_local_x(midpoint + 0.25, 22.0, VIDEO_DISCRETE_SLIDER_GAP, 4),
            Some(1)
        );
    }

    #[test]
    fn slider_cycle_arrow_positions_share_left_anchor_and_expand_rightward() {
        let (left_two, right_two) = video_slider_cycle_arrow_positions(2, 22.0);
        let (left_three, right_three) = video_slider_cycle_arrow_positions(3, 22.0);

        assert_eq!(left_two, left_three);
        assert!(right_three > right_two);
        assert!(left_two < right_two);
    }

    #[test]
    fn selector_inputs_wrap_at_edges() {
        let mut snapshot = default_video_settings();
        snapshot.present_mode = VideoPresentMode::Mailbox;

        let changed = step_video_top_option_for_input(
            &mut snapshot,
            VideoTabKind::Display,
            2,
            true,
        );

        assert!(changed);
        assert_eq!(snapshot.present_mode, VideoPresentMode::AutoVsync);
    }

    #[test]
    fn slider_inputs_clamp_at_edges() {
        let mut snapshot = default_video_settings();
        snapshot.scan_spacing = ScanSpacing::Sparse;

        let changed = step_video_top_option_for_input(
            &mut snapshot,
            VideoTabKind::Crt,
            1,
            true,
        );

        assert!(!changed);
        assert_eq!(snapshot.scan_spacing, ScanSpacing::Sparse);
    }

    #[test]
    fn every_video_top_option_has_non_empty_hover_description() {
        let tabs = [
            VideoTabKind::Display,
            VideoTabKind::Bloom,
            VideoTabKind::Advanced,
            VideoTabKind::Crt,
        ];
        for tab in tabs {
            for key in video_top_option_keys(tab) {
                assert!(!key.description().trim().is_empty());
            }
        }
    }

    #[test]
    fn dropdown_value_descriptions_exist_for_tonemapping() {
        let key = VideoTopOptionKey::Tonemapping;
        for index in 0..key.choice_count() {
            assert!(key.value_description(index).is_some());
        }
    }

    #[test]
    fn dropdown_value_descriptions_exclude_resolution() {
        let key = VideoTopOptionKey::Resolution;
        for index in 0..key.choice_count() {
            assert!(key.value_description(index).is_none());
        }
    }

    #[test]
    fn non_resolution_dropdown_values_have_descriptions() {
        let tabs = [
            VideoTabKind::Display,
            VideoTabKind::Bloom,
            VideoTabKind::Advanced,
            VideoTabKind::Crt,
        ];
        for tab in tabs {
            for key in video_top_option_keys(tab) {
                if !key.uses_dropdown() || key == VideoTopOptionKey::Resolution {
                    continue;
                }
                for index in 0..key.choice_count() {
                    assert!(
                        key.value_description(index).is_some(),
                        "missing value description for '{}' at index {}",
                        key.label(),
                        index
                    );
                }
            }
        }
    }
}
