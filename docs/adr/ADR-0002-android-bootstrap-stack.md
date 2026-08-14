# ADR-0002: Android shell bootstrap stack

**Status:** Accepted
**Date:** 2026-08-13
**Driver:** Phase 9 — Android shell bootstrap

## Context

The Android client (`AdiyutantAndroid/`) is currently an empty stub (README + `.gitkeep`). `TASKS.md` declares `TASK-0019 — Phase 9: Android/Kotlin Project Bootstrap` as the next work item, but `AGENTS.md` `Allowed Changes` explicitly forbids code outside `AdiyutantCore/`. The contradiction blocks the first line of Android code.

The approved design for the mobile "Сегодня" screen (`docs/design/today-screen.html`) establishes a specific dark visual language (custom color tokens, typography, cards, 5-tab bottom navigation). The design system itself is still being finalized separately, so the shell must not depend on the final design spec.

Android ↔ Rust Core integration is **not** a blocker for the bootstrap: UI and integration should evolve independently behind a boundary abstraction.

## Decision

Phase 9 bootstraps `AdiyutantAndroid/` with the following stack:

- **Language:** Kotlin
- **UI toolkit:** Jetpack Compose
- **Component infrastructure:** Material 3
- **Visual style:** Adiyutant UI Kit (custom color/typography tokens), **not** default Material appearance
- **Navigation:** Navigation Compose (`androidx.navigation:navigation-compose`)
- **State:** ViewModel (androidx.lifecycle `viewModelScope`, StateFlow)
- **Core boundary:** `CorePort` interface (repository-style boundary between Android and `AdiyutantCore`)
- **First implementation of the boundary:** `FakeCorePort` — returns sample/static data, no Rust dependency

### Interpretation of "Material 3 as infrastructure"

Material 3 components (Scaffold, NavigationBar, Cards, Buttons, Ripple, shape/semantics primitives) are used for behavior and accessibility structure only. All colors, typography, shapes, and spacing come from the Adiyutant UI Kit theme defined in `ui/theme/` — not from Material's default palettes. Visual appearance of every component is overridden to match `docs/design/today-screen.html`.

### Why ViewModel

- Survives configuration changes.
- Natural host for Future-bound data via `StateFlow`.
- Keeps `CorePort` calls off the main thread.

### Why `CorePort` abstraction

- The Rust integration mechanism is undecided (see ADR-0003). UI code must not be coupled to it.
- Enables parallel development: UI against `FakeCorePort`, integration against the real Core.
- Follows the Core boundary rule: UI shells consume Core, they do not duplicate or own domain logic.

## Rationale

1. **Compose is the current Android default** for new projects and fits a token-driven dark UI well.
2. **Material 3 gives free accessibility/behavior** (ripple, semantics, contrast guidance) while the Adiyutant UI Kit owns the look.
3. **Navigation Compose** is the standard tab-navigation solution and keeps routes declarative.
4. **`FakeCorePort` unblocks the shell now** — the Android project builds and navigates without Rust, without JNI, without decisions on UniFFI.
5. **Separating UI and integration** directly addresses the previously identified blocker: integration is now an independent concern.

## Consequences

- `AdiyutantAndroid/` becomes a compilable Gradle project with a 5-tab navigation shell and stub screens.
- All screens are stubs until the design system and data contracts are finalized.
- `CorePort` and its fake must be written so a real implementation can replace the fake without UI changes.
- `AGENTS.md` `Allowed Changes` must be extended to permit Android code scoped to Phase 9.
- The integration decision (ADR-0003) is deferred and validated by a spike, not by the shell.

## Compliance

- Follows `TASKS.md` `TASK-0019`.
- Extends `AGENTS.md` `Allowed Changes` (see the AGENTS.md update accompanying this ADR).
- Consistent with `ARCHITECTURE.md` "Platform Shell" model: Android is a UI shell consuming Core through a boundary.
