# UI Manual Validation Checklist

Last updated: 2026-02-20

Use this checklist after menu/UI refactors and before release branches.

## Preconditions

- Build runs cleanly: `cargo check`.
- Automated regression passes: `./scripts/ui_regression.sh`.
- Start with default settings and no stale save-state overrides.

## Main Menu

- Open options from main menu with keyboard and mouse.
- Verify only one item is visually selected at a time.
- Verify navigation sound plays on up/down and selection sound on activate.
- Press `Escape` from main menu and confirm it returns to the previous menu layer (does not hard-exit unexpectedly).

## Pause Menu (In-Game)

- Pause in gameplay and open options.
- Confirm gameplay interactions are blocked while pause/menu layers are active.
- Confirm dimming appears on underlying layers when nested menus/modals open.
- Confirm menu audio matches expected pause/options behavior.

## Video Menu: Tabs / Top Options / Footer

- Keyboard path:
  - `Tab`/arrow navigation between tabs, options, and footer.
  - `Enter` activates selected tab and commands.
  - `Right` acts as activate where allowed, `Left` acts as back only where defined.
- Mouse path:
  - Hover/selection transitions between tabs/options/footer remain single-owner and deterministic.
  - No dual/highlight overlap between footer buttons.
- Focus arbitration:
  - When tabs are focused, top option highlight is suppressed.
  - When focus returns to options, tab-only highlight is suppressed.

## Dropdowns

- Open dropdown via keyboard (`Right`, shortcut keys) and mouse click.
- Verify opening on off-screen rows first scrolls row into view.
- Verify dropdown aligns with scrolled row position.
- Verify hover highlight tracks cursor correctly per item.
- Verify click and `Enter` commit selected item and close dropdown.
- Verify clicking outside closes dropdown.
- Verify dropdown does not immediately reopen after selection.

## Sliders and Selectors

- Slider rows:
  - Left/right arrows move value in expected direction.
  - Clicking pips sets the expected discrete level.
  - Non-looping behavior works for ordered scales (min/max clamp visuals correct).
- Selector rows:
  - On/off style options still loop where intended.
  - Clicking value text and option label increments as expected.

## Modals (Apply / Exit Unsaved)

- Apply flow:
  - Open apply modal, verify countdown decrements in paused mode.
  - Confirm keep/revert shortcuts (`Y`, `N`, `Backspace`, `Escape`) match expected buttons.
- Exit unsaved flow:
  - Trigger unsaved changes and attempt exit.
  - Confirm modal appears and underlying menu interactions are blocked.
  - Confirm confirm/cancel actions route correctly.

## Scrollable Areas

- Mouse wheel/trackpad scroll updates viewport in correct direction.
- Scrollbar drag/track interactions remain responsive.
- Auto-scroll edge zones trigger gradual scrolling (no jump-to-end behavior).
- Keyboard navigation reclaims focus-follow and keeps selected row visible.
- Dropdowns and hover boxes remain correctly layered over scroll content.

## HoverBox

- Hover over video option names for >0.5s and verify description appears.
- Hover over dropdown values (where supported) and verify value description appears.
- Verify hover boxes hide when moving away, switching tabs, or opening higher-priority layers.
- Verify hover boxes do not appear for excluded options (for example resolution value rows).

## Debug UI Showcase

- Open debug showcase from debug menu.
- Confirm all showcase windows remain visible and interactable.
- Verify drag works without triggering close.
- Verify each demo primitive is interactive:
  - Menu/selector
  - Tabs
  - Dropdown
  - Scrollbar/scrollable
  - HoverBox (tabs demo labels)

## Stability / Panic Sweep

- Rapidly alternate keyboard + mouse in options/video/dropdown/modal flows.
- Confirm no B0001 panic or winit handler errors.
- If panic occurs:
  - Capture stack trace (`RUST_BACKTRACE=1`).
  - Note exact input sequence and active owner/layer when it occurs.

