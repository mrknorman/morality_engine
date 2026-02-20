#!/usr/bin/env bash
set -euo pipefail

manifest_path="Cargo.toml"

tests=(
  "systems::ui::menu::command_flow::tests::command_flow_systems_initialize_without_query_alias_panics"
  "systems::ui::menu::dropdown_flow::tests::dropdown_flow_systems_initialize_without_query_alias_panics"
  "systems::ui::menu::dropdown_view::tests::dropdown_view_systems_initialize_without_query_alias_panics"
  "systems::ui::menu::menu_input::tests::menu_input_systems_initialize_without_query_alias_panics"
  "systems::ui::menu::main_menu::tests::main_menu_systems_initialize_without_query_alias_panics"
  "systems::ui::menu::modal_flow::tests::modal_flow_systems_initialize_without_query_alias_panics"
  "systems::ui::menu::tabbed_menu::tests::tabbed_menu_systems_initialize_without_query_alias_panics"
  "systems::ui::menu::debug_showcase::tests::debug_showcase_systems_initialize_without_query_alias_panics"
  "systems::ui::menu::video_visuals::tests::video_visual_systems_initialize_without_query_alias_panics"
)

for test_name in "${tests[@]}"; do
  echo "[ui-query-safety] cargo test $test_name"
  cargo test --manifest-path "$manifest_path" "$test_name"
done

echo "[ui-query-safety] complete"
