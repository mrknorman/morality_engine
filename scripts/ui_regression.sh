#!/usr/bin/env bash
set -euo pipefail

manifest_path="Cargo.toml"

echo "[ui-regression] cargo check"
cargo check --manifest-path "${manifest_path}"

echo "[ui-regression] cargo test --no-run"
cargo test --manifest-path "${manifest_path}" --no-run --offline

tests=(
  "systems::ui::menu::video_visuals::tests::footer_highlight_resolver_prefers_pressed_then_hovered_then_selected"
  "systems::ui::menu::video_visuals::tests::footer_highlight_resolver_breaks_ties_by_higher_selectable_index"
  "systems::ui::menu::video_visuals::tests::hover_description_sync_populates_option_and_open_dropdown_value_content"
  "systems::ui::menu::flow_tests::tab_change_closes_open_video_dropdown_for_owner"
)

for test_name in "${tests[@]}"; do
  echo "[ui-regression] cargo test ${test_name}"
  cargo test --manifest-path "${manifest_path}" --offline "${test_name}"
done

if cargo nextest --version >/dev/null 2>&1; then
  echo "[ui-regression] cargo nextest run"
  cargo nextest run --manifest-path "${manifest_path}" --profile default
else
  echo "[ui-regression] cargo-nextest not installed; skipping nextest run"
fi

if [[ "${UI_RUN_GPU_SMOKE:-0}" == "1" ]]; then
  echo "[ui-regression] UI_RUN_GPU_SMOKE=1 -> running GPU smoke lane"
  ./scripts/ui_gpu_smoke.sh
fi

echo "[ui-regression] complete"
