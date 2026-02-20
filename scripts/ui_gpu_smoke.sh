#!/usr/bin/env bash
set -euo pipefail

manifest_path="Cargo.toml"

tests=(
  "systems::ui::scroll::tests::ui_gpu_smoke_scroll_keyboard_and_wheel_input_is_stable"
  "systems::ui::scroll::tests::ui_gpu_smoke_mixed_input_stays_clamped_over_many_frames"
)

run_one() {
  local test_name="$1"
  echo "[ui-gpu-smoke] cargo test --ignored ${test_name}"
  local cmd=(
    cargo test
    --manifest-path "${manifest_path}"
    --offline
    "${test_name}"
    --
    --ignored
    --nocapture
  )
  if command -v xvfb-run >/dev/null 2>&1; then
    xvfb-run -a "${cmd[@]}"
  else
    "${cmd[@]}"
  fi
}

if command -v xvfb-run >/dev/null 2>&1; then
  echo "[ui-gpu-smoke] using xvfb-run"
else
  echo "[ui-gpu-smoke] xvfb-run not found; running directly"
fi

for test_name in "${tests[@]}"; do
  run_one "${test_name}"
done

echo "[ui-gpu-smoke] complete"
