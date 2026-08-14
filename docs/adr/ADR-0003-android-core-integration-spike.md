# ADR-0003: Android ↔ Rust Core integration mechanism (spike-first)

**Status:** Proposed — decision gated on spike results
**Date:** 2026-08-13
**Driver:** Phase 9 — Android shell bootstrap (integration not a blocker for bootstrap)

## Context

`AdiyutantCore` is a Rust workspace exposing a service facade (`AdiyutantCoreService`) whose public methods return flat serde DTOs:

- Methods return `Result<Dto, CoreError>`.
- All DTO fields are `String`, primitive types, or `Vec<Dto>` — no `Id<T>`, no `chrono` types in the public surface.
- A few internal models carry `serde_json::Value` (e.g. `ActionProposal.payload_json`), but public DTOs avoid it.
- Errors are surfaced as flat `CoreErrorDto` for external consumers.

The Android shell must call this facade from Kotlin. `ARCHITECTURE.md` lists the integration mechanism as a **Known Unknown**. The Android bootstrap does **not** depend on this decision (it uses `FakeCorePort` per ADR-0002).

## Candidates

| Option | Description | Notes |
|--------|-------------|-------|
| **UniFFI** (mozilla/uniffi-rs) | Generates Kotlin bindings from a UDL/`.udl` file (or procedural macro) over Rust `pub` types | Preferred candidate; typed, supports custom errors and async via callback interfaces |
| **Manual JNI** | Hand-written JNI bindings + `libadiyutant.so` via `jni` crate | Full control, no codegen, higher maintenance, manual marshalling of all DTOs |
| **Local HTTP/JSON bridge** | Rust side runs an HTTP server; Android uses Retrofit/Ktor against `localhost` | Simple, but adds a process boundary and lifecycle complexity; not local-first friendly |
| **WASM / WebView** | Compile Core to wasm and run in a webview | Not appropriate for a native Android app |

## Decision

Choose **UniFFI** as the primary candidate and validate it with a **spike** before committing. The spike must answer these questions:

1. **DTO round-trip:** Can the existing facade DTOs (including `Vec<Dto>` and nested structures) cross the boundary without manual mapping?
2. **Error handling:** How does `CoreError` map to Kotlin exceptions? Can `CoreErrorDto` be produced on error paths?
3. **`serde_json::Value`:** Does the UniFFI version support it, or must the two DTOs that reference it be reworked?
4. **Generic `Id<T>` and internal types:** The public surface uses flat DTOs, but `store: Box<dyn Store<Error = CoreError>>` is a trait object. Can the facade be exposed through UniFFI while keeping `Box<dyn Store>` internal?
5. **Async:** UniFFI callback interfaces for async calls — sufficient for the `viewModelScope` usage?
6. **Build integration:** Gradle plugin (`org.mozilla.uniffi` or `uniffi-bindgen` Gradle tasks) inside `AdiyutantAndroid/`; build time and generated-code size.
7. **Version compatibility:** `uniffi` version vs Rust toolchain used by the workspace.

### Spike scope (deliverable of this ADR)

A minimal proof, not a product integration:

- New temporary Rust crate (or branch in `AdiyutantCore/`) exposing 2–3 facade methods through UniFFI (e.g. `get_today`, `get_current_activity`, `create_checkin`).
- Generated Kotlin bindings.
- A minimal Android test calling the generated bindings against an in-memory `SqliteStore`.
- Round-trip assertions on a DTO with nested `Vec` and on an error path.

### Spike exit criteria

- Pass criteria 1–7 above are answered with concrete evidence.
- If UniFFI fails materially (error mapping, `Value` support, async), revisit the table and either (a) adapt the facade shape so UniFFI works, or (b) pick manual JNI and record the deviation in this ADR.

## Rationale

- **Typed, maintained codegen** — UniFFI is the standard cross-language bridge in the Rust ecosystem for mobile.
- **Facade is already bridge-friendly**: flat DTOs, string dates, no generic types in the public surface.
- **Independent evolution**: the spike does not block Phase 9 bootstrap; UI uses `FakeCorePort` meanwhile.

## Consequences

- While the spike runs, Android UI code targets `CorePort` (ADR-0002) and is unaffected by the outcome.
- If UniFFI is accepted, a `CorePort` implementation (`UniffiCorePort`) replaces `FakeCorePort` behind the same interface.
- The spike may require minor adjustments to the facade (e.g. error-to-DTO shaping) — those are Core changes and will be tracked separately.
- This ADR flips to **Accepted** with the spike evidence attached, or is superseded by the JNI fallback.

## Compliance

- Does not violate the Core boundary: no domain logic duplicated in Android; Core stays the source of truth.
- Does not introduce LLM provider calls or API-key storage in the mobile client (per `AGENTS.md` Security Rules).
