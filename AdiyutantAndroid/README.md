# AdiyutantAndroid

Status: **Phase 9 — Android shell bootstrap (in progress).**

Android UI shell for Adiyutant. Kotlin + Jetpack Compose. Material 3 is used
as component infrastructure only — the visual style comes from the Adiyutant
UI Kit dark tokens (`app/src/main/kotlin/com/adiyutant/app/ui/theme/`), sourced
from `docs/design/today-screen.html`.

## Stack (ADR-0002)

- Kotlin
- Jetpack Compose
- Material 3 (infrastructure)
- Navigation Compose (5-tab shell)
- ViewModel + StateFlow
- `CorePort` boundary + `FakeCorePort` (sample data, no Rust dependency yet)

## Build

```bash
./gradlew assembleDebug        # build APK
./gradlew test                 # JVM unit tests
./gradlew build                # full build
```

The Android ↔ Rust Core integration is **not** wired yet. UI works against
`FakeCorePort`. The integration mechanism is decided by a spike — see
`docs/adr/ADR-0003-android-core-integration-spike.md`.

## Scope guard

Out of scope for Phase 9 (see `../ROADMAP.md` and `../AGENTS.md`):
full screen implementations from design prototypes, extending Core DTOs,
and Android ↔ Rust Core integration.
