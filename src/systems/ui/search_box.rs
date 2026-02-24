//! SearchBox composition primitive built on top of `TextInputBox`.
//!
//! Responsibilities:
//! - compose and configure `TextInputBox`
//! - maintain normalized incremental query state
//! - expose query/submission/clear messages
use bevy::{
    ecs::{lifecycle::HookContext, world::DeferredWorld},
    prelude::*,
};

use crate::systems::ui::{
    layer::UiLayerKind,
    text_input_box::{
        TextInputBox, TextInputBoxCancelled, TextInputBoxChanged, TextInputBoxPlaceholder,
        TextInputBoxSubmitted, TextInputBoxValue,
    },
};

pub struct SearchBoxPlugin;

impl Plugin for SearchBoxPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<SearchBoxQueryChanged>()
            .add_message::<SearchBoxSubmitted>()
            .add_message::<SearchBoxClearRequested>()
            .add_systems(
                PostUpdate,
                (
                    sync_search_query_from_text_input_changes,
                    sync_search_submitted_from_text_input_submitted,
                    clear_search_box_on_text_input_cancel,
                    handle_search_box_clear_requests,
                ),
            );
    }
}

#[derive(Component, Clone, Copy, Debug)]
#[require(SearchBoxConfig, SearchBoxQuery)]
#[component(on_insert = SearchBox::on_insert)]
pub struct SearchBox {
    pub owner: Entity,
    pub input_layer: UiLayerKind,
}

impl SearchBox {
    pub const fn new(owner: Entity, input_layer: UiLayerKind) -> Self {
        Self { owner, input_layer }
    }

    fn on_insert(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
        let Some(search_box) = world.entity(entity).get::<SearchBox>().copied() else {
            return;
        };
        let config = world
            .entity(entity)
            .get::<SearchBoxConfig>()
            .cloned()
            .unwrap_or_default();

        if world.entity(entity).get::<TextInputBox>().is_none() {
            world.commands().entity(entity).insert((
                TextInputBox::new(search_box.owner, search_box.input_layer),
                TextInputBoxPlaceholder::new(config.placeholder.clone()),
            ));
        } else if world.entity(entity).get::<TextInputBoxPlaceholder>().is_none() {
            world
                .commands()
                .entity(entity)
                .insert(TextInputBoxPlaceholder::new(config.placeholder));
        }
    }
}

#[derive(Component, Clone, Debug)]
pub struct SearchBoxConfig {
    pub placeholder: String,
    pub trim_whitespace: bool,
    pub normalize_ascii_case: bool,
}

impl Default for SearchBoxConfig {
    fn default() -> Self {
        Self {
            placeholder: String::from("Search..."),
            trim_whitespace: true,
            normalize_ascii_case: true,
        }
    }
}

#[derive(Component, Clone, Debug, Default, PartialEq, Eq)]
pub struct SearchBoxQuery {
    pub raw: String,
    pub normalized: String,
}

#[derive(Message, Clone, Debug, PartialEq, Eq)]
pub struct SearchBoxQueryChanged {
    pub entity: Entity,
    pub raw: String,
    pub normalized: String,
}

#[derive(Message, Clone, Debug, PartialEq, Eq)]
pub struct SearchBoxSubmitted {
    pub entity: Entity,
    pub raw: String,
    pub normalized: String,
}

#[derive(Message, Clone, Debug, PartialEq, Eq)]
pub struct SearchBoxClearRequested {
    pub entity: Entity,
}

fn normalize_query(raw: &str, config: &SearchBoxConfig) -> String {
    let trimmed = if config.trim_whitespace {
        raw.trim()
    } else {
        raw
    };
    if config.normalize_ascii_case {
        trimmed.to_ascii_lowercase()
    } else {
        trimmed.to_string()
    }
}

fn sync_search_query_from_text_input_changes(
    mut changed_input: MessageReader<TextInputBoxChanged>,
    mut query: Query<(&SearchBoxConfig, &mut SearchBoxQuery), With<SearchBox>>,
    mut changed_output: MessageWriter<SearchBoxQueryChanged>,
) {
    for changed in changed_input.read() {
        let Ok((config, mut search_query)) = query.get_mut(changed.entity) else {
            continue;
        };
        let normalized = normalize_query(&changed.value, config);
        if search_query.raw == changed.value && search_query.normalized == normalized {
            continue;
        }
        search_query.raw = changed.value.clone();
        search_query.normalized = normalized.clone();
        changed_output.write(SearchBoxQueryChanged {
            entity: changed.entity,
            raw: search_query.raw.clone(),
            normalized,
        });
    }
}

fn sync_search_submitted_from_text_input_submitted(
    mut submitted_input: MessageReader<TextInputBoxSubmitted>,
    query: Query<(&SearchBoxConfig, &SearchBoxQuery), With<SearchBox>>,
    mut submitted_output: MessageWriter<SearchBoxSubmitted>,
) {
    for submitted in submitted_input.read() {
        let Ok((config, search_query)) = query.get(submitted.entity) else {
            continue;
        };
        submitted_output.write(SearchBoxSubmitted {
            entity: submitted.entity,
            raw: submitted.value.clone(),
            normalized: if search_query.raw == submitted.value {
                search_query.normalized.clone()
            } else {
                normalize_query(&submitted.value, config)
            },
        });
    }
}

fn clear_search_box_on_text_input_cancel(
    mut cancelled_input: MessageReader<TextInputBoxCancelled>,
    mut clear_writer: MessageWriter<SearchBoxClearRequested>,
    query: Query<(), With<SearchBox>>,
) {
    for cancelled in cancelled_input.read() {
        if query.get(cancelled.entity).is_ok() {
            clear_writer.write(SearchBoxClearRequested {
                entity: cancelled.entity,
            });
        }
    }
}

fn handle_search_box_clear_requests(
    mut clear_requests: MessageReader<SearchBoxClearRequested>,
    mut query: Query<(&SearchBoxConfig, &mut TextInputBoxValue, &mut SearchBoxQuery), With<SearchBox>>,
    mut text_changed_writer: MessageWriter<TextInputBoxChanged>,
    mut search_changed_writer: MessageWriter<SearchBoxQueryChanged>,
) {
    for request in clear_requests.read() {
        let Ok((config, mut value, mut search_query)) = query.get_mut(request.entity) else {
            continue;
        };
        if value.0.is_empty() && search_query.raw.is_empty() && search_query.normalized.is_empty() {
            continue;
        }
        value.0.clear();
        search_query.raw.clear();
        search_query.normalized = normalize_query("", config);

        text_changed_writer.write(TextInputBoxChanged {
            entity: request.entity,
            value: String::new(),
        });
        search_changed_writer.write(SearchBoxQueryChanged {
            entity: request.entity,
            raw: String::new(),
            normalized: search_query.normalized.clone(),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::IntoSystem;

    #[test]
    fn search_box_insert_hook_adds_text_input_box() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let owner = app.world_mut().spawn_empty().id();
        let entity = app
            .world_mut()
            .spawn((SearchBox::new(owner, UiLayerKind::Base),))
            .id();
        app.update();

        assert!(app.world().entity(entity).contains::<TextInputBox>());
        assert!(app.world().entity(entity).contains::<SearchBoxQuery>());
        let placeholder = app
            .world()
            .entity(entity)
            .get::<TextInputBoxPlaceholder>()
            .expect("search placeholder");
        assert_eq!(placeholder.0, "Search...");
    }

    #[test]
    fn text_input_change_updates_normalized_search_query() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<TextInputBoxChanged>();
        app.add_message::<SearchBoxQueryChanged>();

        let owner = app.world_mut().spawn_empty().id();
        let entity = app.world_mut().spawn((
            SearchBox::new(owner, UiLayerKind::Base),
            SearchBoxConfig::default(),
            SearchBoxQuery::default(),
        )).id();

        app.world_mut()
            .write_message(TextInputBoxChanged {
                entity,
                value: String::from("  TeSt  "),
            });

        let mut system = IntoSystem::into_system(sync_search_query_from_text_input_changes);
        system.initialize(app.world_mut());
        system
            .run((), app.world_mut())
            .expect("search query sync system should run");
        system.apply_deferred(app.world_mut());

        let query = app
            .world()
            .entity(entity)
            .get::<SearchBoxQuery>()
            .expect("search query state");
        assert_eq!(query.raw, "  TeSt  ");
        assert_eq!(query.normalized, "test");
    }

    #[test]
    fn search_box_systems_initialize_without_query_alias_panics() {
        let mut world = World::new();

        let mut changed_system =
            IntoSystem::into_system(sync_search_query_from_text_input_changes);
        changed_system.initialize(&mut world);

        let mut submitted_system =
            IntoSystem::into_system(sync_search_submitted_from_text_input_submitted);
        submitted_system.initialize(&mut world);

        let mut cancel_clear_system =
            IntoSystem::into_system(clear_search_box_on_text_input_cancel);
        cancel_clear_system.initialize(&mut world);

        let mut clear_system = IntoSystem::into_system(handle_search_box_clear_requests);
        clear_system.initialize(&mut world);
    }
}
