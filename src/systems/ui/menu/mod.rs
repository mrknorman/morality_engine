use std::collections::{HashMap, HashSet};

use bevy::{
    app::AppExit,
    audio::Volume,
    post_process::bloom::{Bloom, BloomCompositeMode},
    prelude::*,
    window::{
        ClosingWindow, MonitorSelection, PresentMode, PrimaryWindow, Window, WindowCloseRequested,
        WindowMode,
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
            InteractionCapture, InteractionCaptureOwner, InteractionGate, InteractionSystem,
            InteractionVisualState, OptionCycler, Selectable, SelectableClickActivation,
            SelectableMenu, SystemMenuActions, SystemMenuSounds,
        },
        time::Dilation,
        ui::{
            discrete_slider,
            dropdown::{self, DropdownAnchorState, DropdownLayerState},
            layer::{self, UiLayer, UiLayerKind},
            selector::{self, ShortcutKey},
            tabs,
        },
    },
};

mod defs;
mod command_flow;
mod command_effects;
mod command_reducer;
mod dropdown_flow;
mod dropdown_view;
mod footer_nav;
mod menu_input;
mod modal_flow;
mod page_content;
mod root_spawn;
pub mod schema;
mod stack;
mod tabbed_focus;
mod tabbed_menu;
mod video_visuals;

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
};
use page_content::{rebuild_menu_page, spawn_page_content};
use stack::MenuNavigationState;
use video_visuals::{
    ensure_video_discrete_slider_slot_clickables, suppress_left_cycle_arrow_for_dropdown_options,
    sync_video_cycle_arrow_positions, sync_video_discrete_slider_widgets,
    sync_video_footer_table_values, sync_video_option_cycler_bounds, sync_video_tabs_visuals,
    sync_video_top_selection_bars, sync_video_top_table_values,
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
    main_camera_query: Query<'w, 's, &'static mut Bloom, With<MainCamera>>,
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
        _ => VideoDisplayMode::Borderless,
    };
    snapshot.vsync_enabled = matches!(window.present_mode, PresentMode::AutoVsync);
    snapshot.resolution_index = resolution_index_from_window(window);
    snapshot
}

fn apply_snapshot_to_window(window: &mut Window, snapshot: VideoSettingsSnapshot) {
    window.mode = match snapshot.display_mode {
        VideoDisplayMode::Windowed => WindowMode::Windowed,
        VideoDisplayMode::Borderless => WindowMode::BorderlessFullscreen(MonitorSelection::Current),
    };
    apply_resolution(window, snapshot.resolution_index);
    window.present_mode = if snapshot.vsync_enabled {
        PresentMode::AutoVsync
    } else {
        PresentMode::AutoNoVsync
    };
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
        BloomStyle::Off => {
            let mut bloom = Bloom::NATURAL;
            bloom.intensity = 0.0;
            bloom
        }
        BloomStyle::Natural => Bloom::NATURAL,
        BloomStyle::OldSchool => Bloom::OLD_SCHOOL,
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

    if snapshot.bloom_style == BloomStyle::Off {
        bloom.intensity = 0.0;
    }

    bloom
}

fn apply_snapshot_to_post_processing(
    snapshot: VideoSettingsSnapshot,
    crt_settings: &mut CrtSettings,
    main_camera_query: &mut Query<&mut Bloom, With<MainCamera>>,
) {
    let (spacing, thickness, darkness) = match snapshot.scan_spacing {
        ScanSpacing::Off => (0, 1, 0.0),
        ScanSpacing::Fine => (1, 1, 0.45),
        ScanSpacing::Balanced => (2, 1, 0.6),
        ScanSpacing::Sparse => (3, 2, 0.75),
    };
    crt_settings.spacing = spacing;
    crt_settings.thickness = thickness;
    crt_settings.darkness = darkness;
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
    let all_effects_off = snapshot.scan_spacing == ScanSpacing::Off
        && snapshot.scan_thickness == ScanThickness::Off
        && snapshot.scan_darkness == ScanDarkness::Off;
    crt_settings.pipeline_enabled = !all_effects_off;

    if let Ok(mut bloom) = main_camera_query.single_mut() {
        *bloom = snapshot_bloom_settings(snapshot);
    }
}

fn initialize_video_settings_from_window(
    mut settings: ResMut<VideoSettingsState>,
    crt_settings: Res<CrtSettings>,
    main_camera_query: Query<&Bloom, With<MainCamera>>,
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
        snapshot.scan_spacing = ScanSpacing::Off;
        snapshot.scan_thickness = ScanThickness::Off;
        snapshot.scan_darkness = ScanDarkness::Off;
    } else {
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
    }

    if let Ok(bloom) = main_camera_query.single() {
        snapshot.bloom_style = if bloom.intensity <= 0.001 {
            BloomStyle::Off
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
                // Scroll integration point (Stage 1 contract):
                // scroll viewport surface + scrollbar visuals should render in Visual after
                // menu/layout state is finalized for the frame.
                sync_video_top_table_values,
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
