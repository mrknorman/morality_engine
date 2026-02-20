//! Menu composition systems built from reusable `systems::ui` primitives.
//!
//! This module owns policy/state transitions for menu pages while delegating
//! generic behavior (layers, dropdown state, tabs, selectors, scrolling) to
//! primitive modules.
use std::collections::{HashMap, HashSet};

use bevy::{
    app::AppExit,
    anti_alias::{
        contrast_adaptive_sharpening::ContrastAdaptiveSharpening,
        fxaa::{Fxaa, Sensitivity},
    },
    audio::Volume,
    core_pipeline::tonemapping::{DebandDither, Tonemapping},
    post_process::bloom::{Bloom, BloomCompositeMode},
    post_process::effect_stack::ChromaticAberration,
    prelude::*,
    render::view::Msaa,
    window::{
        ClosingWindow, CompositeAlphaMode, MonitorSelection, PresentMode, PrimaryWindow,
        VideoModeSelection, Window, WindowCloseRequested, WindowMode,
    },
};

use crate::{
    data::states::{MainState, PauseState},
    startup::system_menu,
    startup::render::{CrtSettings, MainCamera},
    systems::{
        audio::{continuous_audio, DilatableAudio, TransientAudio, TransientAudioPallet},
        interaction::{
            interaction_gate_allows_for_owner, option_cycler_input_system, Clickable,
            Hoverable, InteractionCapture, InteractionCaptureOwner, InteractionGate,
            InteractionSystem, InteractionVisualState, OptionCycler, Selectable,
            SelectableClickActivation,
            SelectableMenu, SystemMenuActions, SystemMenuSounds,
        },
        time::Dilation,
        ui::{
            discrete_slider,
            dropdown::{self, DropdownAnchorState, DropdownLayerState},
            layer::{self, UiLayer, UiLayerKind},
            scroll::{ScrollFocusFollowLock, ScrollState},
            selector::{self, ShortcutKey},
            tabs,
        },
    },
};

mod defs;
mod command_flow;
mod command_effects;
mod command_reducer;
mod debug_showcase;
mod dropdown_flow;
mod dropdown_view;
mod footer_nav;
mod menu_input;
mod modal_flow;
mod page_content;
mod root_spawn;
mod scroll_adapter;
pub mod schema;
mod stack;
mod tabbed_focus;
mod tabbed_menu;
mod video_visuals;
#[cfg(test)]
mod flow_tests;

pub use defs::{
    MenuCommand, MenuHost, MenuOptionCommand, MenuPage, MenuPageContent, MenuRoot, MenuStack,
    PauseMenuAudio,
};
pub use root_spawn::spawn_menu_root;
use defs::*;
use command_flow::{
    handle_menu_option_commands, handle_option_cycler_commands,
    handle_video_discrete_slider_slot_commands,
    OptionCommandSuppressions,
};
use command_effects::apply_menu_reducer_result;
use command_reducer::reduce_menu_command;
use dropdown_flow::{
    close_dropdowns_for_menu, close_resolution_dropdown_on_outside_click,
    handle_resolution_dropdown_item_commands, handle_resolution_dropdown_keyboard_navigation,
    open_dropdown_for_menu, sync_video_tab_content_state,
};
use dropdown_view::{
    ensure_resolution_dropdown_value_arrows, recenter_resolution_dropdown_item_text,
    sync_resolution_dropdown_items, update_resolution_dropdown_value_arrows,
};
use modal_flow::{
    any_video_modal_open, handle_video_modal_button_commands, handle_video_modal_shortcuts,
    update_apply_confirmation_countdown,
};
use menu_input::{
    apply_menu_intents, enforce_active_layer_focus, handle_menu_shortcuts,
    play_menu_navigation_sound,
    suppress_option_visuals_for_inactive_layers_and_tab_focus,
};
use page_content::{rebuild_menu_page, spawn_page_content};
use stack::MenuNavigationState;
use video_visuals::{
    ensure_video_discrete_slider_slot_clickables, suppress_left_cycle_arrow_for_dropdown_options,
    sync_video_cycle_arrow_positions, sync_video_discrete_slider_widgets,
    sync_video_footer_table_values, sync_video_option_cycler_bounds, sync_video_tabs_visuals,
    sync_video_top_option_hover_descriptions, sync_video_top_selection_bars,
    sync_video_top_table_values,
};

#[derive(Message, Clone, Debug)]
enum MenuIntent {
    TriggerCommand {
        menu_entity: Entity,
        command: MenuCommand,
    },
    TriggerModalButton(VideoModalButton),
}

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
enum MenuSystems {
    Core,
    Commands,
    PostCommands,
    Visual,
}

#[derive(bevy::ecs::system::SystemParam)]
struct WindowExitContext<'w, 's> {
    primary_window_queries: ParamSet<
        'w,
        's,
        (
            Query<
                'w,
                's,
                Entity,
                (With<Window>, With<PrimaryWindow>, Without<ClosingWindow>),
            >,
            Query<'w, 's, &'static mut Window, With<PrimaryWindow>>,
        ),
    >,
    close_requests: MessageWriter<'w, WindowCloseRequested>,
    app_exit: MessageWriter<'w, AppExit>,
}

#[derive(bevy::ecs::system::SystemParam)]
struct MenuCommandContext<'w, 's> {
    menu_queries: ParamSet<
        'w,
        's,
        (
            Query<'w, 's, (&'static MenuRoot, &'static mut MenuStack, &'static mut SelectableMenu)>,
            Query<'w, 's, (&'static MenuRoot, &'static MenuStack)>,
            Query<
                'w,
                's,
                &'static mut SelectableMenu,
                (With<VideoResolutionDropdown>, Without<MenuRoot>),
            >,
        ),
    >,
    page_content_query: Query<'w, 's, (Entity, &'static ChildOf), With<MenuPageContent>>,
    asset_server: Res<'w, AssetServer>,
    audio_query: Query<'w, 's, (&'static mut TransientAudio, Option<&'static DilatableAudio>)>,
    dilation: Res<'w, Dilation>,
    window_exit: WindowExitContext<'w, 's>,
    next_main_state: ResMut<'w, NextState<MainState>>,
    next_pause_state: ResMut<'w, NextState<PauseState>>,
    settings: ResMut<'w, VideoSettingsState>,
    crt_settings: ResMut<'w, CrtSettings>,
    dropdown_state: ResMut<'w, DropdownLayerState>,
    navigation_state: ResMut<'w, MenuNavigationState>,
    main_camera_query: Query<
        'w,
        's,
        (
            &'static mut Bloom,
            &'static mut Tonemapping,
            &'static mut DebandDither,
            &'static mut Fxaa,
            &'static mut ContrastAdaptiveSharpening,
            &'static mut ChromaticAberration,
            &'static mut Msaa,
        ),
        With<MainCamera>,
    >,
    // Read-only layer resolution and mutable dropdown visibility live in one ParamSet to keep
    // Visibility access disjoint (avoids Bevy B0001 conflicts).
    layer_queries: ParamSet<
        'w,
        's,
        (
            Query<
                'w,
                's,
                (
                    Entity,
                    &'static UiLayer,
                    Option<&'static Visibility>,
                    Option<&'static InteractionGate>,
                ),
            >,
            Query<
                'w,
                's,
                (
                    Entity,
                    &'static ChildOf,
                    &'static UiLayer,
                    &'static mut Visibility,
                ),
                With<VideoResolutionDropdown>,
            >,
        ),
    >,
    video_tab_query: Query<
        'w,
        's,
        (&'static tabs::TabBar, &'static tabs::TabBarState),
        With<tabbed_menu::TabbedMenuConfig>,
    >,
    video_top_scroll_query: Query<
        'w,
        's,
        (
            &'static scroll_adapter::ScrollableTableAdapter,
            &'static mut ScrollState,
            &'static mut ScrollFocusFollowLock,
        ),
        With<VideoTopOptionsScrollRoot>,
    >,
}

#[derive(Resource, Default)]
struct InactiveLayerSelectionCache {
    by_layer: HashMap<Entity, usize>,
}

fn resolution_index_from_window(window: &Window) -> usize {
    let current = (window.resolution.width(), window.resolution.height());
    RESOLUTIONS
        .iter()
        .position(|&(w, h)| (current.0 - w).abs() < 1.0 && (current.1 - h).abs() < 1.0)
        .unwrap_or(0)
}

fn apply_resolution(window: &mut Window, index: usize) {
    let clamped = index.min(RESOLUTIONS.len() - 1);
    let (w, h) = RESOLUTIONS[clamped];
    window.resolution.set(w, h);
}

fn snapshot_from_window(window: &Window) -> VideoSettingsSnapshot {
    let mut snapshot = default_video_settings();
    snapshot.display_mode = match window.mode {
        WindowMode::Windowed => VideoDisplayMode::Windowed,
        WindowMode::BorderlessFullscreen(_) => VideoDisplayMode::Borderless,
        WindowMode::Fullscreen(_, _) => VideoDisplayMode::Fullscreen,
    };
    snapshot.resolution_index = resolution_index_from_window(window);
    snapshot.present_mode = video_present_mode_from_bevy(window.present_mode);
    snapshot.window_resizable = window.resizable;
    snapshot.window_decorations = window.decorations;
    snapshot.window_transparent = window.transparent;
    snapshot.composite_alpha = video_composite_alpha_from_bevy(window.composite_alpha_mode);
    snapshot
}

fn apply_snapshot_to_window(window: &mut Window, snapshot: VideoSettingsSnapshot) {
    window.mode = match snapshot.display_mode {
        VideoDisplayMode::Windowed => WindowMode::Windowed,
        VideoDisplayMode::Borderless => WindowMode::BorderlessFullscreen(MonitorSelection::Current),
        VideoDisplayMode::Fullscreen => {
            WindowMode::Fullscreen(MonitorSelection::Current, VideoModeSelection::Current)
        }
    };
    apply_resolution(window, snapshot.resolution_index);
    window.present_mode = bevy_present_mode(snapshot.present_mode);
    window.resizable = snapshot.window_resizable;
    window.decorations = snapshot.window_decorations;
    window.transparent = snapshot.window_transparent;
    window.composite_alpha_mode = bevy_composite_alpha(snapshot.composite_alpha);
}

fn active_video_tabs_by_menu(
    tab_query: &Query<(&tabs::TabBar, &tabs::TabBarState), With<tabbed_menu::TabbedMenuConfig>>,
) -> HashMap<Entity, usize> {
    let mut active_tab_by_menu = HashMap::new();
    for (tab_bar, tab_state) in tab_query.iter() {
        active_tab_by_menu.insert(tab_bar.owner, tab_state.active_index);
    }
    active_tab_by_menu
}

fn snapshot_bloom_settings(snapshot: VideoSettingsSnapshot) -> Bloom {
    let mut bloom = match snapshot.bloom_style {
        BloomStyle::Natural => Bloom::NATURAL,
        BloomStyle::OldSchool => Bloom::OLD_SCHOOL,
        BloomStyle::ScreenBlur => Bloom::SCREEN_BLUR,
    };

    bloom.intensity *= match snapshot.bloom_intensity {
        BloomIntensity::Low => 0.65,
        BloomIntensity::Medium => 1.0,
        BloomIntensity::High => 1.45,
    };

    bloom.high_pass_frequency *= match snapshot.bloom_scatter {
        BloomScatter::Tight => 0.7,
        BloomScatter::Normal => 1.0,
        BloomScatter::Wide => 1.3,
    };

    bloom.composite_mode = match snapshot.bloom_composite {
        BloomComposite::EnergyConserving => BloomCompositeMode::EnergyConserving,
        BloomComposite::Additive => BloomCompositeMode::Additive,
    };

    match snapshot.bloom_threshold {
        BloomThreshold::Off => {
            bloom.prefilter.threshold = 0.0;
            bloom.prefilter.threshold_softness = 0.0;
        }
        BloomThreshold::Soft => {
            bloom.prefilter.threshold = 0.4;
            bloom.prefilter.threshold_softness = 0.35;
        }
        BloomThreshold::Hard => {
            bloom.prefilter.threshold = 0.7;
            bloom.prefilter.threshold_softness = 0.1;
        }
    }

    bloom.scale = match snapshot.anamorphic_scale {
        AnamorphicScale::Off => Vec2::ONE,
        AnamorphicScale::Wide => Vec2::new(2.0, 1.0),
        AnamorphicScale::UltraWide => Vec2::new(4.0, 1.0),
    };
    if snapshot.anamorphic_scale != AnamorphicScale::Off {
        bloom.max_mip_dimension = Bloom::NATURAL.max_mip_dimension * 2;
    }

    bloom.low_frequency_boost = match snapshot.bloom_boost {
        BloomBoost::Off => 0.0,
        BloomBoost::Low => 0.35,
        BloomBoost::Medium => 0.7,
        BloomBoost::High => 1.0,
    };
    bloom.low_frequency_boost_curvature = match snapshot.bloom_boost {
        BloomBoost::Off => 0.0,
        BloomBoost::Low => 0.5,
        BloomBoost::Medium => 0.8,
        BloomBoost::High => 1.0,
    };

    if !snapshot.bloom_enabled {
        bloom.intensity = 0.0;
        bloom.low_frequency_boost = 0.0;
    }

    bloom
}

fn video_present_mode_from_bevy(mode: PresentMode) -> VideoPresentMode {
    match mode {
        PresentMode::AutoVsync => VideoPresentMode::AutoVsync,
        PresentMode::AutoNoVsync => VideoPresentMode::AutoNoVsync,
        PresentMode::Fifo => VideoPresentMode::Fifo,
        PresentMode::FifoRelaxed => VideoPresentMode::FifoRelaxed,
        PresentMode::Immediate => VideoPresentMode::Immediate,
        PresentMode::Mailbox => VideoPresentMode::Mailbox,
    }
}

fn bevy_present_mode(mode: VideoPresentMode) -> PresentMode {
    match mode {
        VideoPresentMode::AutoVsync => PresentMode::AutoVsync,
        VideoPresentMode::AutoNoVsync => PresentMode::AutoNoVsync,
        VideoPresentMode::Fifo => PresentMode::Fifo,
        VideoPresentMode::FifoRelaxed => PresentMode::FifoRelaxed,
        VideoPresentMode::Immediate => PresentMode::Immediate,
        VideoPresentMode::Mailbox => PresentMode::Mailbox,
    }
}

fn video_composite_alpha_from_bevy(mode: CompositeAlphaMode) -> VideoCompositeAlpha {
    match mode {
        CompositeAlphaMode::Auto => VideoCompositeAlpha::Auto,
        CompositeAlphaMode::Opaque => VideoCompositeAlpha::Opaque,
        CompositeAlphaMode::PreMultiplied => VideoCompositeAlpha::PreMultiplied,
        CompositeAlphaMode::PostMultiplied => VideoCompositeAlpha::PostMultiplied,
        CompositeAlphaMode::Inherit => VideoCompositeAlpha::Inherit,
    }
}

fn bevy_composite_alpha(mode: VideoCompositeAlpha) -> CompositeAlphaMode {
    match mode {
        VideoCompositeAlpha::Auto => CompositeAlphaMode::Auto,
        VideoCompositeAlpha::Opaque => CompositeAlphaMode::Opaque,
        VideoCompositeAlpha::PreMultiplied => CompositeAlphaMode::PreMultiplied,
        VideoCompositeAlpha::PostMultiplied => CompositeAlphaMode::PostMultiplied,
        VideoCompositeAlpha::Inherit => CompositeAlphaMode::Inherit,
    }
}

fn fxaa_sensitivity_from_quality(quality: FxaaQuality) -> Sensitivity {
    match quality {
        FxaaQuality::Low => Sensitivity::Low,
        FxaaQuality::Medium => Sensitivity::Medium,
        FxaaQuality::High => Sensitivity::High,
        FxaaQuality::Ultra => Sensitivity::Ultra,
        FxaaQuality::Extreme => Sensitivity::Extreme,
    }
}

fn fxaa_quality_from_sensitivity(sensitivity: Sensitivity) -> FxaaQuality {
    match sensitivity {
        Sensitivity::Low => FxaaQuality::Low,
        Sensitivity::Medium => FxaaQuality::Medium,
        Sensitivity::High => FxaaQuality::High,
        Sensitivity::Ultra => FxaaQuality::Ultra,
        Sensitivity::Extreme => FxaaQuality::Extreme,
    }
}

fn cas_strength_value(strength: CasStrength) -> f32 {
    match strength {
        CasStrength::Off => 0.0,
        CasStrength::Low => 0.3,
        CasStrength::Medium => 0.6,
        CasStrength::High => 0.9,
    }
}

fn cas_strength_from_value(value: f32) -> CasStrength {
    if value <= 0.001 {
        CasStrength::Off
    } else if value < 0.45 {
        CasStrength::Low
    } else if value < 0.75 {
        CasStrength::Medium
    } else {
        CasStrength::High
    }
}

fn crt_effect_scalar(level: CrtEffectLevel) -> f32 {
    match level {
        CrtEffectLevel::Off => 0.0,
        CrtEffectLevel::Low => 0.33,
        CrtEffectLevel::Medium => 0.66,
        CrtEffectLevel::High => 1.0,
    }
}

fn crt_effect_level_from_scalar(value: f32) -> CrtEffectLevel {
    if value <= 0.001 {
        CrtEffectLevel::Off
    } else if value < 0.45 {
        CrtEffectLevel::Low
    } else if value < 0.8 {
        CrtEffectLevel::Medium
    } else {
        CrtEffectLevel::High
    }
}

fn apply_snapshot_to_post_processing(
    snapshot: VideoSettingsSnapshot,
    crt_settings: &mut CrtSettings,
    main_camera_query: &mut Query<
        (
            &mut Bloom,
            &mut Tonemapping,
            &mut DebandDither,
            &mut Fxaa,
            &mut ContrastAdaptiveSharpening,
            &mut ChromaticAberration,
            &mut Msaa,
        ),
        With<MainCamera>,
    >,
) {
    crt_settings.spacing = match snapshot.scan_spacing {
        ScanSpacing::Off => 0,
        ScanSpacing::Fine => 1,
        ScanSpacing::Balanced => 2,
        ScanSpacing::Sparse => 3,
    };
    crt_settings.thickness = match snapshot.scan_thickness {
        ScanThickness::Off | ScanThickness::Low => 1,
        ScanThickness::Medium => 2,
        ScanThickness::High => 3,
    };
    crt_settings.darkness = match snapshot.scan_darkness {
        ScanDarkness::Off => 0.0,
        ScanDarkness::Low => 0.45,
        ScanDarkness::Medium => 0.6,
        ScanDarkness::High => 0.75,
    };
    crt_settings.curvature_strength = match snapshot.scan_thickness {
        ScanThickness::Off => 0.0,
        ScanThickness::Low => 0.04,
        ScanThickness::Medium => 0.08,
        ScanThickness::High => 0.13,
    };
    crt_settings.static_strength = match snapshot.scan_darkness {
        ScanDarkness::Off => 0.0,
        ScanDarkness::Low => 0.008,
        ScanDarkness::Medium => 0.015,
        ScanDarkness::High => 0.03,
    };
    crt_settings.jitter_strength = 0.36 * crt_effect_scalar(snapshot.scan_jitter);
    crt_settings.aberration_strength = 0.7 * crt_effect_scalar(snapshot.scan_aberration);
    crt_settings.phosphor_strength = crt_effect_scalar(snapshot.scan_phosphor);
    crt_settings.vignette_strength = 0.85 * crt_effect_scalar(snapshot.scan_vignette);
    crt_settings.glow_strength = 0.9 * crt_effect_scalar(snapshot.scan_phosphor);

    let all_effects_off = snapshot.scan_spacing == ScanSpacing::Off
        && snapshot.scan_thickness == ScanThickness::Off
        && snapshot.scan_darkness == ScanDarkness::Off
        && snapshot.scan_jitter == CrtEffectLevel::Off
        && snapshot.scan_aberration == CrtEffectLevel::Off
        && snapshot.scan_phosphor == CrtEffectLevel::Off
        && snapshot.scan_vignette == CrtEffectLevel::Off;
    crt_settings.pipeline_enabled = snapshot.crt_enabled && !all_effects_off;

    if let Ok((
        mut bloom,
        mut tonemapping,
        mut deband_dither,
        mut fxaa,
        mut cas,
        mut chromatic,
        mut msaa,
    )) = main_camera_query.single_mut()
    {
        *bloom = snapshot_bloom_settings(snapshot);
        *tonemapping = match snapshot.tonemapping {
            VideoTonemapping::None => Tonemapping::None,
            VideoTonemapping::Reinhard => Tonemapping::Reinhard,
            VideoTonemapping::ReinhardLuminance => Tonemapping::ReinhardLuminance,
            VideoTonemapping::AcesFitted => Tonemapping::AcesFitted,
            VideoTonemapping::AgX => Tonemapping::AgX,
            VideoTonemapping::SomewhatBoringDisplayTransform => {
                Tonemapping::SomewhatBoringDisplayTransform
            }
            VideoTonemapping::TonyMcMapface => Tonemapping::TonyMcMapface,
            VideoTonemapping::BlenderFilmic => Tonemapping::BlenderFilmic,
        };
        *deband_dither = if snapshot.deband_dither_enabled {
            DebandDither::Enabled
        } else {
            DebandDither::Disabled
        };
        let fxaa_sensitivity = fxaa_sensitivity_from_quality(snapshot.fxaa_quality);
        fxaa.enabled = snapshot.fxaa_enabled;
        fxaa.edge_threshold = fxaa_sensitivity;
        fxaa.edge_threshold_min = fxaa_sensitivity;
        cas.enabled = snapshot.cas_enabled && snapshot.cas_strength != CasStrength::Off;
        cas.sharpening_strength = cas_strength_value(snapshot.cas_strength);
        cas.denoise = false;
        chromatic.intensity = if snapshot.chromatic_enabled {
            0.03 * crt_effect_scalar(snapshot.chromatic_intensity)
        } else {
            0.0
        };
        chromatic.max_samples = 8;
        *msaa = match snapshot.msaa {
            VideoMsaa::Off => Msaa::Off,
            VideoMsaa::Sample2 => Msaa::Sample2,
            VideoMsaa::Sample4 => Msaa::Sample4,
            // HDR render target formats commonly cap guaranteed MSAA at 4x.
            // Clamp 8x to 4x to avoid device validation panics.
            VideoMsaa::Sample8 => Msaa::Sample4,
        };
    }
}

fn initialize_video_settings_from_window(
    mut settings: ResMut<VideoSettingsState>,
    crt_settings: Res<CrtSettings>,
    main_camera_query: Query<
        (
            &Bloom,
            &Tonemapping,
            &DebandDither,
            &Fxaa,
            &ContrastAdaptiveSharpening,
            &ChromaticAberration,
            &Msaa,
        ),
        With<MainCamera>,
    >,
    window_query: Query<&Window, With<PrimaryWindow>>,
) {
    if settings.initialized {
        return;
    }
    let Ok(window) = window_query.single() else {
        return;
    };
    let mut snapshot = snapshot_from_window(window);
    if !crt_settings.pipeline_enabled {
        snapshot.crt_enabled = false;
        snapshot.scan_spacing = ScanSpacing::Off;
        snapshot.scan_thickness = ScanThickness::Off;
        snapshot.scan_darkness = ScanDarkness::Off;
        snapshot.scan_jitter = CrtEffectLevel::Off;
        snapshot.scan_aberration = CrtEffectLevel::Off;
        snapshot.scan_phosphor = CrtEffectLevel::Off;
        snapshot.scan_vignette = CrtEffectLevel::Off;
    } else {
        snapshot.crt_enabled = true;
        snapshot.scan_spacing = if crt_settings.spacing <= 0 {
            ScanSpacing::Off
        } else if crt_settings.spacing <= 1 {
            ScanSpacing::Fine
        } else if crt_settings.spacing == 2 {
            ScanSpacing::Balanced
        } else {
            ScanSpacing::Sparse
        };
        snapshot.scan_thickness = if crt_settings.curvature_strength <= 0.001 {
            ScanThickness::Off
        } else if crt_settings.curvature_strength < 0.06 {
            ScanThickness::Low
        } else if crt_settings.curvature_strength < 0.1 {
            ScanThickness::Medium
        } else {
            ScanThickness::High
        };
        snapshot.scan_darkness = if crt_settings.static_strength <= 0.001 {
            ScanDarkness::Off
        } else if crt_settings.static_strength < 0.012 {
            ScanDarkness::Low
        } else if crt_settings.static_strength < 0.022 {
            ScanDarkness::Medium
        } else {
            ScanDarkness::High
        };
        snapshot.scan_jitter = crt_effect_level_from_scalar((crt_settings.jitter_strength / 0.36).clamp(0.0, 1.0));
        snapshot.scan_aberration = crt_effect_level_from_scalar(
            (crt_settings.aberration_strength / 0.7).clamp(0.0, 1.0),
        );
        snapshot.scan_phosphor =
            crt_effect_level_from_scalar(crt_settings.phosphor_strength.clamp(0.0, 1.0));
        snapshot.scan_vignette = crt_effect_level_from_scalar(
            (crt_settings.vignette_strength / 0.85).clamp(0.0, 1.0),
        );
    }

    if let Ok((bloom, tonemapping, deband_dither, fxaa, cas, chromatic, msaa)) =
        main_camera_query.single()
    {
        snapshot.bloom_enabled = bloom.intensity > 0.001;
        snapshot.bloom_style = if bloom.high_pass_frequency < 0.45 && bloom.intensity >= 0.5 {
            BloomStyle::ScreenBlur
        } else if bloom.prefilter.threshold >= 0.5
            || matches!(bloom.composite_mode, BloomCompositeMode::Additive)
        {
            BloomStyle::OldSchool
        } else {
            BloomStyle::Natural
        };
        snapshot.bloom_intensity = if bloom.intensity < 0.1 {
            BloomIntensity::Low
        } else if bloom.intensity < 0.25 {
            BloomIntensity::Medium
        } else {
            BloomIntensity::High
        };
        snapshot.bloom_scatter = if bloom.high_pass_frequency < 0.85 {
            BloomScatter::Tight
        } else if bloom.high_pass_frequency <= 1.15 {
            BloomScatter::Normal
        } else {
            BloomScatter::Wide
        };
        snapshot.bloom_composite = match bloom.composite_mode {
            BloomCompositeMode::EnergyConserving => BloomComposite::EnergyConserving,
            BloomCompositeMode::Additive => BloomComposite::Additive,
        };
        snapshot.bloom_threshold = if bloom.prefilter.threshold < 0.2 {
            BloomThreshold::Off
        } else if bloom.prefilter.threshold < 0.55 {
            BloomThreshold::Soft
        } else {
            BloomThreshold::Hard
        };
        snapshot.anamorphic_scale = if bloom.scale.x < 1.5 {
            AnamorphicScale::Off
        } else if bloom.scale.x < 3.0 {
            AnamorphicScale::Wide
        } else {
            AnamorphicScale::UltraWide
        };
        snapshot.bloom_boost = if bloom.low_frequency_boost <= 0.05 {
            BloomBoost::Off
        } else if bloom.low_frequency_boost < 0.5 {
            BloomBoost::Low
        } else if bloom.low_frequency_boost < 0.85 {
            BloomBoost::Medium
        } else {
            BloomBoost::High
        };

        snapshot.tonemapping = match tonemapping {
            Tonemapping::None => VideoTonemapping::None,
            Tonemapping::Reinhard => VideoTonemapping::Reinhard,
            Tonemapping::ReinhardLuminance => VideoTonemapping::ReinhardLuminance,
            Tonemapping::AcesFitted => VideoTonemapping::AcesFitted,
            Tonemapping::AgX => VideoTonemapping::AgX,
            Tonemapping::SomewhatBoringDisplayTransform => {
                VideoTonemapping::SomewhatBoringDisplayTransform
            }
            Tonemapping::TonyMcMapface => VideoTonemapping::TonyMcMapface,
            Tonemapping::BlenderFilmic => VideoTonemapping::BlenderFilmic,
        };
        snapshot.deband_dither_enabled = matches!(deband_dither, DebandDither::Enabled);
        snapshot.fxaa_enabled = fxaa.enabled;
        snapshot.fxaa_quality = fxaa_quality_from_sensitivity(fxaa.edge_threshold);
        snapshot.cas_strength = cas_strength_from_value(cas.sharpening_strength);
        snapshot.cas_enabled = cas.enabled && snapshot.cas_strength != CasStrength::Off;
        snapshot.chromatic_intensity =
            crt_effect_level_from_scalar((chromatic.intensity / 0.03).clamp(0.0, 1.0));
        snapshot.chromatic_enabled =
            chromatic.intensity > 0.0001 && snapshot.chromatic_intensity != CrtEffectLevel::Off;
        snapshot.msaa = match msaa {
            Msaa::Off => VideoMsaa::Off,
            Msaa::Sample2 => VideoMsaa::Sample2,
            Msaa::Sample4 => VideoMsaa::Sample4,
            // Normalize unsupported/high samples to the UI's max supported setting.
            Msaa::Sample8 => VideoMsaa::Sample4,
        };
    }

    settings.saved = snapshot;
    settings.pending = snapshot;
    settings.initialized = true;
}

fn sanitize_menu_selection_indices(
    mut menu_query: Query<
        (Entity, &mut SelectableMenu),
        Or<(
            With<MenuRoot>,
            With<VideoResolutionDropdown>,
            With<VideoModalRoot>,
        )>,
    >,
    selectable_query: Query<&Selectable>,
) {
    let mut indices_by_menu: HashMap<Entity, Vec<usize>> = HashMap::new();

    for selectable in selectable_query.iter() {
        indices_by_menu
            .entry(selectable.menu_entity)
            .or_default()
            .push(selectable.index);
    }

    for (menu_entity, mut menu) in menu_query.iter_mut() {
        let Some(indices) = indices_by_menu.get_mut(&menu_entity) else {
            if menu.selected_index != 0 {
                menu.selected_index = 0;
            }
            continue;
        };

        indices.sort_unstable();
        indices.dedup();
        if indices.is_empty() {
            menu.selected_index = 0;
            continue;
        }

        if !indices.contains(&menu.selected_index) {
            menu.selected_index = indices[0];
        }
    }
}

fn enforce_video_menu_hover_click_activation(
    mut menu_query: Query<(&MenuStack, &mut SelectableMenu), With<MenuRoot>>,
) {
    for (menu_stack, mut selectable_menu) in menu_query.iter_mut() {
        if menu_stack.current_page() == Some(MenuPage::Video) {
            selectable_menu.click_activation = SelectableClickActivation::HoveredOnly;
        }
    }
}

fn enforce_menu_layer_invariants(
    mut navigation_state: ResMut<MenuNavigationState>,
    mut dropdown_state: ResMut<DropdownLayerState>,
    mut dropdown_anchor_state: ResMut<DropdownAnchorState>,
    menu_root_query: Query<Entity, With<MenuRoot>>,
    modal_query: Query<(), With<VideoModalRoot>>,
    mut dropdown_query: Query<(Entity, &ChildOf, &UiLayer, &mut Visibility), With<VideoResolutionDropdown>>,
) {
    stack::clear_stale_menu_targets(&mut navigation_state, &menu_root_query);
    dropdown::enforce_single_visible_layer::<VideoResolutionDropdown, MenuRoot>(
        &mut dropdown_state,
        &menu_root_query,
        any_video_modal_open(&modal_query),
        &mut dropdown_query,
    );
    dropdown_anchor_state.retain_open_parents::<MenuRoot>(&dropdown_state, &menu_root_query);
}

pub struct MenusPlugin;

impl Plugin for MenusPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<VideoSettingsState>()
            .init_resource::<DropdownLayerState>()
            .init_resource::<DropdownAnchorState>()
            .init_resource::<MenuNavigationState>()
            .init_resource::<OptionCommandSuppressions>()
            .init_resource::<tabbed_menu::TabbedMenuFocusState>()
            .init_resource::<InactiveLayerSelectionCache>()
            .add_message::<MenuIntent>()
            .add_message::<tabs::TabChanged>();
        app.configure_sets(
            Update,
            (
                MenuSystems::Core,
                MenuSystems::Commands.after(MenuSystems::Core),
                MenuSystems::PostCommands.after(MenuSystems::Commands),
                MenuSystems::Visual.after(MenuSystems::PostCommands),
            ),
        );
        app.add_systems(
            Update,
            (
                enforce_video_menu_hover_click_activation,
                scroll_adapter::sync_video_top_option_hit_regions_to_viewport,
            )
                .chain()
                .before(InteractionSystem::Selectable),
        );
        app.add_systems(
            Update,
            (
                // Scroll integration point (Stage 1 contract):
                // owner-scoped scroll input collection should run in Core after
                // `InteractionSystem::Selectable` has produced deterministic hover/press
                // state, and before menu command execution mutates page state.
                initialize_video_settings_from_window,
                selector::sync_option_cycler_bounds,
                enforce_active_layer_focus,
                tabbed_menu::sync_tabbed_menu_focus,
                option_cycler_input_system,
                system_menu::consume_cycle_arrow_clicks,
                tabbed_menu::commit_tab_activation,
                sync_video_tab_content_state,
                tabbed_menu::suppress_tabbed_options_while_tabs_focused,
                play_menu_navigation_sound,
                handle_menu_shortcuts,
                handle_resolution_dropdown_keyboard_navigation,
                apply_menu_intents,
            )
                .chain()
                .in_set(MenuSystems::Core)
                .after(InteractionSystem::Selectable),
        );
        app.add_systems(
            Update,
            (
                // Scroll integration point (Stage 1 contract):
                // scroll-derived click/drag commands (e.g., scrollbar thumb/track)
                // should be handled in the Commands stage with other direct UI command
                // handlers to preserve deterministic ordering.
                handle_video_discrete_slider_slot_commands,
                handle_video_modal_button_commands,
                handle_resolution_dropdown_item_commands,
                close_resolution_dropdown_on_outside_click,
            )
                .chain()
                .in_set(MenuSystems::Commands)
                .after(InteractionSystem::Selectable),
        );
        app.add_systems(
            Update,
            (
                debug_showcase::handle_menu_demo_commands,
                debug_showcase::handle_dropdown_trigger_commands,
                debug_showcase::handle_dropdown_item_commands,
                debug_showcase::close_dropdown_on_outside_click,
            )
                .chain()
                .in_set(MenuSystems::Commands)
                .after(InteractionSystem::Selectable),
        );
        app.add_systems(
            Update,
            update_apply_confirmation_countdown
                .in_set(MenuSystems::Commands)
                .after(close_resolution_dropdown_on_outside_click)
                .after(InteractionSystem::Selectable),
        );
        app.add_systems(
            Update,
            handle_menu_option_commands
                .in_set(MenuSystems::Commands)
                .after(update_apply_confirmation_countdown)
                .after(InteractionSystem::Selectable),
        );
        app.add_systems(
            Update,
            (
                // Scroll integration point (Stage 1 contract):
                // scroll state reduction and owner-local focus-follow should occur in
                // PostCommands so base command mutations are settled before visual sync.
                handle_option_cycler_commands,
                tabs::sanitize_tab_selection_indices,
                tabs::sync_tab_bar_state,
                tabbed_menu::cleanup_tabbed_menu_state,
                sanitize_menu_selection_indices,
                scroll_adapter::sync_video_top_scroll_focus_follow,
                enforce_menu_layer_invariants,
            )
                .chain()
                .in_set(MenuSystems::PostCommands)
                .after(InteractionSystem::Selectable),
        );
        app.add_systems(
            Update,
            handle_video_modal_shortcuts
                .after(option_cycler_input_system)
                .before(apply_menu_intents)
                .in_set(MenuSystems::Core)
                .after(InteractionSystem::Selectable),
        );
        app.add_systems(
            Update,
            (
                ensure_resolution_dropdown_value_arrows,
                update_resolution_dropdown_value_arrows,
                recenter_resolution_dropdown_item_text,
            )
                .chain()
                .in_set(MenuSystems::Visual)
                .after(sync_resolution_dropdown_items)
                .after(InteractionSystem::Selectable),
        );
        app.add_systems(
            Update,
            (
                debug_showcase::sync_menu_demo_visuals,
                debug_showcase::sync_tabs_demo_visuals,
                debug_showcase::sync_dropdown_demo_visuals,
            )
                .chain()
                .in_set(MenuSystems::Visual)
                .after(InteractionSystem::Selectable),
        );
        app.add_systems(
            Update,
            (
                // Scroll integration point (Stage 1 contract):
                // scroll viewport surface + scrollbar visuals should render in Visual after
                // menu/layout state is finalized for the frame.
                suppress_option_visuals_for_inactive_layers_and_tab_focus,
                sync_video_top_table_values,
                sync_video_top_option_hover_descriptions,
                sync_video_option_cycler_bounds,
                sync_video_discrete_slider_widgets,
                discrete_slider::ensure_discrete_slider_slots,
                ensure_video_discrete_slider_slot_clickables,
                discrete_slider::sync_discrete_slider_slots,
                sync_video_footer_table_values,
                sync_video_tabs_visuals,
                sync_resolution_dropdown_items,
                system_menu::ensure_selection_indicators,
                system_menu::update_selection_indicators,
                system_menu::ensure_selection_bars,
                system_menu::update_selection_bars,
                sync_video_top_selection_bars,
                system_menu::ensure_cycle_arrows,
                sync_video_cycle_arrow_positions,
                system_menu::update_cycle_arrows,
                suppress_left_cycle_arrow_for_dropdown_options,
            )
                .chain()
                .in_set(MenuSystems::Visual)
                .after(InteractionSystem::Selectable),
        );
    }
}
