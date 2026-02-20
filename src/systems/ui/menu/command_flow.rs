use std::collections::HashSet;

use bevy::prelude::*;

use super::*;
use crate::{
    startup::cursor::CustomCursor,
    systems::ui::discrete_slider::{DiscreteSlider, DiscreteSliderSlot},
};

#[derive(Resource, Default)]
pub(super) struct OptionCommandSuppressions {
    slider_rows_clicked_by_slot: HashSet<(Entity, usize)>,
}

impl OptionCommandSuppressions {
    fn suppress_slider_row_click(&mut self, menu_entity: Entity, row: usize) {
        self.slider_rows_clicked_by_slot.insert((menu_entity, row));
    }

    fn take_slider_row_click_suppression(&mut self, menu_entity: Entity, row: usize) -> bool {
        self.slider_rows_clicked_by_slot.remove(&(menu_entity, row))
    }

    fn clear(&mut self) {
        self.slider_rows_clicked_by_slot.clear();
    }
}

pub(super) fn handle_menu_option_commands(
    mut commands: Commands,
    pause_state: Option<Res<State<PauseState>>>,
    capture_query: Query<Option<&InteractionCaptureOwner>, With<InteractionCapture>>,
    showcase_root_query: Query<Entity, With<debug_showcase::DebugUiShowcaseRoot>>,
    mut dropdown_anchor_state: ResMut<DropdownAnchorState>,
    mut suppressions: ResMut<OptionCommandSuppressions>,
    mut option_query: Query<(
        Entity,
        &Selectable,
        &mut Clickable<SystemMenuActions>,
        &MenuOptionCommand,
        &TransientAudioPallet<SystemMenuSounds>,
        Option<&OptionCycler>,
    )>,
    mut ctx: MenuCommandContext,
) {
    // Query contract:
    // - option command consumption happens in `option_query`.
    // - menu stack + layer/dropdown state mutations are isolated in ParamSet-backed
    //   context queries (`MenuCommandContext`) to prevent aliasing conflicts.
    let pause_state = pause_state.as_ref();
    let active_tabs = active_video_tabs_by_menu(&ctx.video_tab_query);
    let active_layers = {
        let ui_layer_query = ctx.layer_queries.p0();
        layer::active_layers_by_owner_scoped(pause_state, &capture_query, &ui_layer_query)
    };
    let mut dropdown_query = ctx.layer_queries.p1();
    let mut dirty_menus = HashSet::new();
    let mut closed_menus = HashSet::new();
    let mut pending_dropdown_open: Vec<(Entity, usize, usize)> = Vec::new();
    let mut menu_query = ctx.menu_queries.p0();

    if let Some(menu_entity) = ctx.navigation_state.pending_exit_menu.take() {
        let close_menu_system = ctx.navigation_state.pending_exit_closes_menu_system;
        ctx.navigation_state.pending_exit_closes_menu_system = false;

        if let Ok((menu_root, mut menu_stack, mut selectable_menu)) =
            menu_query.get_mut(menu_entity)
        {
            if close_menu_system {
                match menu_root.host {
                    MenuHost::Pause => ctx.next_pause_state.set(PauseState::Unpaused),
                    MenuHost::Debug | MenuHost::Main => {
                        closed_menus.insert(menu_entity);
                    }
                }
            } else if let Some(selected_index) = menu_stack.pop() {
                selectable_menu.selected_index = selected_index;
                dirty_menus.insert(menu_entity);
            } else {
                closed_menus.insert(menu_entity);
            }
        }
    }

    let mut pending_option_entities: Vec<Entity> = Vec::new();
    for (option_entity, _, clickable, _, _, _) in option_query.iter_mut() {
        if clickable.triggered {
            pending_option_entities.push(option_entity);
        }
    }
    pending_option_entities.sort_by_key(|entity| entity.to_bits());

    for option_entity in pending_option_entities {
        let Ok((_, selectable, mut clickable, option_command, click_pallet, cycler)) =
            option_query.get_mut(option_entity)
        else {
            continue;
        };
        if !clickable.triggered {
            continue;
        }
        clickable.triggered = false;

        let active_tab = video_tab_kind(
            active_tabs
                .get(&selectable.menu_entity)
                .copied()
                .unwrap_or(0),
        );
        let active_kind =
            layer::active_layer_kind_for_owner(&active_layers, selectable.menu_entity);

        if active_kind == UiLayerKind::Modal {
            continue;
        }

        if active_kind == UiLayerKind::Dropdown {
            let is_dropdown_toggle = video_top_row_for_command(&option_command.0)
                .is_some_and(|row| video_top_option_uses_dropdown(active_tab, row));
            if !is_dropdown_toggle {
                continue;
            }
        }
        if let Some(row) = video_top_row_for_command(&option_command.0) {
            if !video_top_option_uses_dropdown(active_tab, row) {
                if suppressions.take_slider_row_click_suppression(selectable.menu_entity, row) {
                    continue;
                }
                if cycler.is_some_and(|cycler| cycler.left_triggered || cycler.right_triggered) {
                    continue;
                }
            }
        }

        TransientAudioPallet::play_transient_audio(
            option_entity,
            &mut commands,
            click_pallet,
            SystemMenuSounds::Click,
            ctx.dilation.0,
            &mut ctx.audio_query,
        );

        let Ok((menu_root, mut menu_stack, mut selectable_menu)) =
            menu_query.get_mut(selectable.menu_entity)
        else {
            continue;
        };
        if selectable_menu.selected_index != selectable.index {
            selectable_menu.selected_index = selectable.index;
        }

        menu_stack.remember_selected_index(selectable_menu.selected_index);
        let current_page = menu_stack.current_page();

        let reduction = reduce_menu_command(
            option_command.0.clone(),
            selectable.menu_entity,
            current_page,
            active_tab,
            &mut menu_stack,
            &mut selectable_menu,
            &mut ctx.settings,
            &mut ctx.dropdown_state,
            &mut ctx.navigation_state,
        );

        apply_menu_reducer_result(
            reduction,
            selectable.menu_entity,
            menu_root,
            &mut commands,
            &ctx.asset_server,
            &mut ctx.settings,
            &mut ctx.dropdown_state,
            &mut dropdown_query,
            &mut ctx.crt_settings,
            &mut ctx.main_camera_query,
            &mut ctx.window_exit,
            &mut ctx.next_pause_state,
            &mut ctx.next_main_state,
            &mut ctx.next_game_state,
            &mut ctx.next_sub_state,
            &mut ctx.scene_queue,
            &ctx.main_menu_overlay_query,
            &ctx.offscreen_camera_query,
            &ctx.main_camera_transform_query,
            &showcase_root_query,
            &mut dirty_menus,
            &mut closed_menus,
            &mut pending_dropdown_open,
        );
    }
    suppressions.clear();

    for (menu_entity, row, selected_index) in pending_dropdown_open {
        if closed_menus.contains(&menu_entity) {
            continue;
        }
        let mut dropdown_menu_query = ctx.menu_queries.p2();
        open_dropdown_for_menu(
            menu_entity,
            row,
            selected_index,
            &mut dropdown_anchor_state,
            &mut ctx.dropdown_state,
            &mut dropdown_query,
            &mut dropdown_menu_query,
            &mut ctx.video_top_scroll_query,
        );
    }

    if ctx
        .navigation_state
        .exit_prompt_target_menu
        .is_some_and(|menu_entity| closed_menus.contains(&menu_entity))
    {
        ctx.navigation_state.exit_prompt_target_menu = None;
    }

    for menu_entity in closed_menus {
        ctx.dropdown_state.clear_owner(menu_entity);
        dropdown_anchor_state.remove_owner(menu_entity);
        dirty_menus.remove(&menu_entity);
        commands.entity(menu_entity).despawn_related::<Children>();
        commands.entity(menu_entity).despawn();
    }

    let menu_query = ctx.menu_queries.p1();
    for menu_entity in dirty_menus {
        let Ok((menu_root, menu_stack)) = menu_query.get(menu_entity) else {
            continue;
        };
        let Some(current_page) = menu_stack.current_page() else {
            continue;
        };

        rebuild_menu_page(
            &mut commands,
            &ctx.asset_server,
            menu_entity,
            current_page,
            menu_root.gate,
            &ctx.page_content_query,
        );
    }
}

pub(super) fn handle_option_cycler_commands(
    mut commands: Commands,
    pause_state: Option<Res<State<PauseState>>>,
    capture_query: Query<Option<&InteractionCaptureOwner>, With<InteractionCapture>>,
    mut option_query: Query<(
        Entity,
        &Selectable,
        &mut OptionCycler,
        &MenuOptionCommand,
        &TransientAudioPallet<SystemMenuSounds>,
    )>,
    mut audio_query: Query<(&mut TransientAudio, Option<&DilatableAudio>)>,
    dilation: Res<Dilation>,
    mut settings: ResMut<VideoSettingsState>,
    tabbed_focus: Res<tabbed_menu::TabbedMenuFocusState>,
    ui_layer_query: Query<(
        Entity,
        &UiLayer,
        Option<&Visibility>,
        Option<&InteractionGate>,
    )>,
    tab_query: Query<(&tabs::TabBar, &tabs::TabBarState), With<tabbed_menu::TabbedMenuConfig>>,
) {
    // Query contract:
    // option-cycler writes are local to `option_query`; layer/tab queries are
    // read-only lookups used for arbitration.
    let active_layers =
        layer::active_layers_by_owner_scoped(pause_state.as_ref(), &capture_query, &ui_layer_query);
    let active_tabs = active_video_tabs_by_menu(&tab_query);

    let mut pending_cyclers: Vec<(Entity, bool)> = Vec::new();
    for (entity, _, mut cycler, _, _) in option_query.iter_mut() {
        let direction = if cycler.left_triggered {
            Some(false)
        } else if cycler.right_triggered {
            Some(true)
        } else {
            None
        };
        cycler.left_triggered = false;
        cycler.right_triggered = false;
        if let Some(forward) = direction {
            pending_cyclers.push((entity, forward));
        }
    }
    pending_cyclers.sort_by_key(|(entity, _)| entity.to_bits());

    for (entity, forward) in pending_cyclers {
        let Ok((_, selectable, _cycler, option_command, click_pallet)) =
            option_query.get_mut(entity)
        else {
            continue;
        };

        let active_kind =
            layer::active_layer_kind_for_owner(&active_layers, selectable.menu_entity);
        if active_kind != UiLayerKind::Base || !settings.initialized {
            continue;
        }
        if tabbed_focus.is_tabs_focused(selectable.menu_entity) {
            continue;
        }

        let Some(active_tab) = active_tabs
            .get(&selectable.menu_entity)
            .copied()
            .map(video_tab_kind)
        else {
            continue;
        };
        let Some(row) = video_top_row_for_command(&option_command.0) else {
            continue;
        };
        if video_top_option_uses_dropdown(active_tab, row) {
            continue;
        }
        let changed =
            step_video_top_option_for_input(&mut settings.pending, active_tab, row, forward);
        if !changed {
            continue;
        }

        TransientAudioPallet::play_transient_audio(
            entity,
            &mut commands,
            click_pallet,
            SystemMenuSounds::Click,
            dilation.0,
            &mut audio_query,
        );
    }
}

pub(super) fn handle_video_discrete_slider_slot_commands(
    mut commands: Commands,
    pause_state: Option<Res<State<PauseState>>>,
    capture_query: Query<Option<&InteractionCaptureOwner>, With<InteractionCapture>>,
    cursor: Res<CustomCursor>,
    mut suppressions: ResMut<OptionCommandSuppressions>,
    mut settings: ResMut<VideoSettingsState>,
    tabbed_focus: Res<tabbed_menu::TabbedMenuFocusState>,
    ui_layer_query: Query<(
        Entity,
        &UiLayer,
        Option<&Visibility>,
        Option<&InteractionGate>,
    )>,
    tab_query: Query<(&tabs::TabBar, &tabs::TabBarState), With<tabbed_menu::TabbedMenuConfig>>,
    mut slot_query: Query<(
        Entity,
        &ChildOf,
        &DiscreteSliderSlot,
        &mut Clickable<SystemMenuActions>,
    )>,
    slider_query: Query<(
        &VideoOptionDiscreteSlider,
        &DiscreteSlider,
        &GlobalTransform,
    )>,
    option_audio_query: Query<
        (
            Entity,
            &Selectable,
            &VideoOptionRow,
            &TransientAudioPallet<SystemMenuSounds>,
        ),
        With<MenuOptionCommand>,
    >,
    mut audio_query: Query<(&mut TransientAudio, Option<&DilatableAudio>)>,
    dilation: Res<Dilation>,
) {
    if !settings.initialized {
        return;
    }

    let active_layers =
        layer::active_layers_by_owner_scoped(pause_state.as_ref(), &capture_query, &ui_layer_query);
    let active_tabs = active_video_tabs_by_menu(&tab_query);

    let mut click_targets: Vec<(Entity, Entity, usize)> = Vec::new();
    for (slot_entity, slot_parent, slot, mut clickable) in slot_query.iter_mut() {
        if !clickable.triggered {
            continue;
        }
        clickable.triggered = false;
        click_targets.push((slot_entity, slot_parent.parent(), slot.index));
    }
    click_targets.sort_by_key(|(slot_entity, _, _)| slot_entity.to_bits());

    let mut audio_by_menu_row: std::collections::HashMap<
        (Entity, usize),
        (Entity, &TransientAudioPallet<SystemMenuSounds>),
    > = std::collections::HashMap::new();
    for (option_entity, selectable, row, pallet) in option_audio_query.iter() {
        if row.index >= VIDEO_TOP_OPTION_COUNT {
            continue;
        }
        audio_by_menu_row.insert((selectable.menu_entity, row.index), (option_entity, pallet));
    }

    for (_slot_entity, slider_entity, slot_index) in click_targets {
        let Ok((slider_meta, slider_widget, slider_global)) = slider_query.get(slider_entity)
        else {
            continue;
        };
        if slider_meta.row >= VIDEO_TOP_OPTION_COUNT {
            continue;
        }
        if tabbed_focus.is_tabs_focused(slider_meta.menu_entity) {
            continue;
        }
        if layer::active_layer_kind_for_owner(&active_layers, slider_meta.menu_entity)
            != UiLayerKind::Base
        {
            continue;
        }

        let Some(active_tab) = active_tabs
            .get(&slider_meta.menu_entity)
            .copied()
            .map(video_tab_kind)
        else {
            continue;
        };
        let Some(key) = video_top_option_key(active_tab, slider_meta.row) else {
            continue;
        };
        if !key.uses_slider() {
            continue;
        }

        suppressions.suppress_slider_row_click(slider_meta.menu_entity, slider_meta.row);

        let cursor_local_x = cursor.position.map(|cursor_position| {
            let slider_world_to_local = slider_global.to_matrix().inverse();
            slider_world_to_local
                .transform_point3(cursor_position.extend(0.0))
                .x
        });
        let Some(next_index) = cursor_local_x
            .and_then(|local_x| {
                key.slider_selected_index_from_local_x(
                    local_x,
                    slider_widget.slot_size.x,
                    slider_widget.slot_gap,
                    slider_widget.layout_steps,
                )
            })
            .or_else(|| key.slider_selected_index_from_slot(slot_index))
        else {
            continue;
        };

        let changed = apply_video_top_option_selected_index(
            &mut settings.pending,
            active_tab,
            slider_meta.row,
            next_index,
        );
        if !changed {
            continue;
        }

        if let Some((option_entity, click_pallet)) =
            audio_by_menu_row.get(&(slider_meta.menu_entity, slider_meta.row))
        {
            TransientAudioPallet::play_transient_audio(
                *option_entity,
                &mut commands,
                click_pallet,
                SystemMenuSounds::Click,
                dilation.0,
                &mut audio_query,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::IntoSystem;

    #[test]
    fn command_flow_systems_initialize_without_query_alias_panics() {
        let mut world = World::new();

        let mut menu_option_system = IntoSystem::into_system(handle_menu_option_commands);
        menu_option_system.initialize(&mut world);

        let mut option_cycler_system = IntoSystem::into_system(handle_option_cycler_commands);
        option_cycler_system.initialize(&mut world);

        let mut slider_slot_system =
            IntoSystem::into_system(handle_video_discrete_slider_slot_commands);
        slider_slot_system.initialize(&mut world);
    }
}
