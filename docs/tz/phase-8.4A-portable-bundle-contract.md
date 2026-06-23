# TZ: Phase 8.4A — Portable Bundle Contract

## Execution Strategy

- [ ] 🟢 Parallel execution recommended
- **Reason:** The work has separable ownership boundaries: bundle contract/docs/examples, parser/validator/security, persistence/import history, service planner/apply/export, and CLI/tests. Some integration points are sequential, so use staged parallelism: define shared DTO/model names first, then split independent implementation units, then run a parent integration checkpoint.

## Execution Checklist

- [ ] 0. Context verified
- [ ] 1. Architecture boundaries confirmed
- [ ] 2. Implementation stage 1 complete — contract, DTO/model skeletons, and dependency choice
- [ ] 3. Implementation stage 2 complete — parallel parser/validator/docs/persistence units integrated
- [ ] 4. Implementation stage 3 complete — shared preview/apply planner, transactional apply, export, and CLI
- [ ] 5. Unit/component tests complete
- [ ] 6. Integration tests with real dependencies complete
- [ ] 7. Stand smoke tests complete
- [ ] 8. UI automation tests complete or explicitly marked not applicable
- [ ] 9. User scenario tests complete
- [ ] 10. Regression checks complete
- [ ] 11. Documentation updated
- [ ] 12. Final acceptance review complete

## Check Rules

- Architect creates checklist items and acceptance criteria.
- Executor agents may check implementation and test items only after implementing the assigned work and attaching evidence.
- QA verifier may check final acceptance only after reviewing command output, smoke evidence, and unchecked/skipped items.
- Failed or unavailable checks stay unchecked with a blocker note.
- This TZ must remain separate from Phase 8.4B. Do not implement Project/Roadmap persistence here.

## Source Scope

Primary scope reference:

- `/home/makc/AI_sandbox/ADIYUTANT/docs/scope/phase-8.4-portable-bundle-and-projects-roadmaps.md`

Current repository facts to verify before editing:

- Rust workspace: `/home/makc/AI_sandbox/ADIYUTANT/AdiyutantCore`
- Current crates: `adiyutant_core`, `adiyutant_store`, `adiyutant_cli`
- Current persistence: SQLite via `rusqlite`, migrations currently through v2
- Existing supported target domains for this TZ: `ContextDocument`, `ChecklistTemplate`, `ChecklistItem`, routines/planning context/preferences, import history
- Unsupported but parseable in this TZ: `projects/*`, `roadmaps/*`

## Goal

Implement **Adiyutant Bundle v1** as a safe portable import/export contract that Core can validate, preview, apply in append/merge mode, and export. The bundle must support both an unpacked directory and `.adiyutant.zip` transport, with strict manifest validation, zip security, report DTOs, persisted import history, and all-or-nothing transactional apply for supported sections.

This phase creates the data-transfer foundation before Android. It must not create Project/Roadmap domain persistence; those sections are preview-only until Phase 8.4B.

## Non-Goals

- No Android implementation.
- No Project/Roadmap tables, models, services, or CLI beyond preview-only bundle parsing/reporting.
- No direct LLM/provider calls.
- No executable rule engine or `LocalRuleGateway` mapping.
- No destructive `replace` or delete import mode.
- No concrete dated `DayPlan` import from `planning.yaml`.
- No cloud/sync/backend import.

## Architecture Boundaries

### Core responsibilities

- Own bundle schema semantics, validation, preview planning, report DTOs, and import/export mapping to Core domain models.
- Treat YAML/Markdown/zip input as untrusted.
- Store user meaning only; do not perform Android file picker behavior or platform-specific UI flows.
- Keep bundle rules as human context, not executable local rules.

### Store responsibilities

- Persist import history.
- Provide stable slug/source lookup for merge behavior.
- Execute supported-section apply as an atomic transaction.

### CLI responsibilities

- Resolve file/directory paths from command-line arguments.
- Call Core facade methods.
- Print reports in a deterministic, script-friendly format.
- Use `ADIYUTANT_DB_PATH` for smoke tests.

## Required Bundle Contract

### Accepted source forms

Core must parse both forms into one internal representation, `AdiyutantBundle`:

```text
1. Directory bundle:
   adiyutant-pack/

2. Zip bundle:
   max-life-core.adiyutant.zip
```

The executor may implement source adapters such as:

```text
BundleSource::Directory(PathBuf)
BundleSource::ZipFile(PathBuf)
BundleSource::ZipBytes(Vec<u8>)
```

Exact names are flexible; the architectural requirement is that Android later can pass zip bytes without owning validation logic.

### Required structure

```text
adiyutant-pack/
  manifest.yaml
  life-core.yaml
  routines.yaml
  rules.yaml
  checklists.yaml
  planning.yaml
  context/
    personal.md
    work.md
    health.md
  projects/
    adiyutant.yaml
    warehouse.yaml
  roadmaps/
    adiyutant-roadmap.yaml
    warehouse-roadmap.yaml
```

### Manifest

`manifest.yaml` is required and authoritative.

Minimum shape:

```yaml
schema_version: "adiyutant.bundle.v1"
bundle_id: "max-core-2026-06"
title: "Max Life Core"
created_at: "2026-06-22T10:00:00Z"

capabilities:
  projects: "preview-only"
  roadmaps: "preview-only"

sections:
  life_core: "life-core.yaml"
  routines: "routines.yaml"
  rules: "rules.yaml"
  checklists: "checklists.yaml"
  planning: "planning.yaml"
  context_dir: "context/"
  projects_dir: "projects/"
  roadmaps_dir: "roadmaps/"
```

Validation rules:

| Case | Result |
|---|---|
| Missing `manifest.yaml` | Error |
| Unsupported `schema_version` | Error |
| Duplicate slug/id | Error |
| Missing required field | Error |
| Invalid enum/status value | Error |
| Unknown top-level file | Warning |
| Unknown optional field | Warning, ignored |
| Unknown required-like section not declared in manifest | Warning unless explicitly required by schema |

Implementation note: to emit warnings for unknown optional fields, do not rely only on strict serde struct deserialization. Parse through a YAML value representation first or use an equivalent mechanism that can inspect extra keys.

### Supported sections in Phase 8.4A

| Section | 8.4A behavior |
|---|---|
| `life-core.yaml` | Apply as Life Core/profile context documents. |
| `routines.yaml` | Apply as routine context/preferences if no stronger typed model exists. |
| `rules.yaml` | Apply as human-readable rules/context, not executable rules. |
| `checklists.yaml` | Apply as `ChecklistTemplate` with embedded `ChecklistItem`s. |
| `planning.yaml` | Apply preferences/templates/candidate hints only; never create a dated `DayPlan`. |
| `context/*.md` | Apply as `ContextDocument` records. |
| `projects/*` | Validate/preview only; ignored on apply with explicit report action. |
| `roadmaps/*` | Validate/preview only; ignored on apply with explicit report action. |

### Planning semantics

Allowed `planning.yaml` direction:

```yaml
default_mode: "manual_review"
daily_capacity:
  deep_tasks_max: 3
  light_tasks_max: 5

templates:
  - id: "normal-workday"
    title: "Normal workday"
    blocks:
      - kind: "morning"
        start: "06:30"
      - kind: "deep_work"
        start: "09:00"
        duration_minutes: 120

candidate_hints:
  - "Prefer project tasks before random backlog"
  - "If sleep score < 6, use recovery mode"
```

Forbidden in this TZ:

```yaml
date: "2026-06-22"
tasks:
  - title: "Concrete imported task"
```

as persisted `DayPlan` data.

## Import Modes

Supported MVP operations:

```text
validate
preview
apply --mode append
apply --mode merge
export
```

Definitions:

- `preview` = parse + validate + compute import plan + no writes.
- `apply` = parse + validate + compute the same import plan + writes.
- `preview` and `apply` must use the same internal planner.
- `append` adds new entities; slug/id conflicts do not update existing entities and must be reported.
- `merge` updates existing entities by stable slug/id and adds absent entities.
- Neither mode may delete absent entities.
- `replace` is out of scope.

## Zip Security Requirements

`.adiyutant.zip` input is untrusted. Acceptance tests must cover malicious archives.

Mandatory checks:

- Reject absolute paths.
- Reject paths containing `..` path traversal.
- Reject symlinks.
- Reject hardlinks if the chosen zip library exposes them.
- Enforce max file count: 200.
- Enforce max uncompressed size: 10 MB.
- Enforce max single YAML file size: 1 MB.
- Enforce max single Markdown file size: 2 MB.
- Allow only `.yaml`, `.yml`, and `.md` extensions.
- Allow only declared/root-approved files and directories.
- Reject hidden executable-ish junk unless explicitly allowed by the bundle contract.

If the selected zip library cannot expose enough metadata to reject symlinks/hardlinks safely, stop implementation and report the dependency blocker instead of silently weakening this requirement.

## Stable Identity and Merge Keys

Bundle YAML uses stable human/AI-readable slugs. Core creates internal UUIDs.

Required model/storage support in this TZ:

- `ChecklistTemplate` gets a stable `slug` suitable for merge, with a uniqueness constraint that does not break existing records.
- `ChecklistItem` gets stable identity within the template, preferably `slug: Option<String>` inside embedded JSON with serde backward compatibility.
- `ContextDocument` gets a stable `source_slug` and enough source metadata to merge imported docs. `title + doc_type` may be used only as a documented temporary fallback if the executor proves schema change is too risky for this slice.
- Existing rows without slugs must remain readable and writable.

Recommended SQLite migration behavior:

- Add nullable slug/source columns first.
- Add unique indexes that tolerate existing null values.
- Do not backfill slugs destructively.
- Add tests for old rows without slugs.

## Report and History Requirements

Every validate/preview/apply operation returns a structured report DTO.

Minimum DTOs:

```text
BundleValidationReportDto
BundlePreviewDto
BundleApplyReportDto
BundleExportReportDto
BundleIssueDto
BundleSectionReportDto
BundleActionDto
```

Minimum report fields:

- `bundle_id`
- `schema_version`
- `status`: `valid`, `valid_with_warnings`, `invalid`, `applied`, `failed`
- `errors[]`
- `warnings[]`
- `sections[]`
- `actions[]`

Minimum action kinds:

- `create`
- `update`
- `skip_existing`
- `ignored_preview_only`
- `conflict`
- `error`

Persist apply history:

```text
import_runs
  id
  bundle_id
  bundle_title
  schema_version
  mode
  status
  started_at
  finished_at
  summary_json
```

Only `apply` must persist `import_runs`. `validate` and `preview` may return reports without persistence.

## Transactionality Requirement

Supported-section apply must be all-or-nothing.

Required pipeline:

```text
parse → validate entire bundle → preview entire bundle → apply supported sections in transaction
```

Rules:

- Apply only proceeds when preview has no errors.
- Warnings may exist, including preview-only project/roadmap sections.
- Unsupported preview-only sections are skipped with explicit report actions.
- If any supported section write fails, context/checklist/routine/rule/planning writes and import run status must be consistent with rollback/failure reporting.
- Do not implement apply as a sequence of independent `Store` calls from the service layer without a transaction boundary.

The executor must add an explicit store-level transaction seam. Exact method names are flexible; acceptable approaches include a composite import apply method or a bounded transaction API exposed through the Store abstraction.

## Export Requirements

Phase 8.4A export includes only supported sections:

```text
manifest.yaml
life-core.yaml
routines.yaml
rules.yaml
checklists.yaml
planning.yaml
context/*.md
```

Export must not write project/roadmap content before 8.4B. The exported manifest should advertise:

```yaml
capabilities:
  projects: "preview-only"
  roadmaps: "preview-only"
```

Export target behavior:

- Directory path: write unpacked bundle directory.
- `.adiyutant.zip` path: write zip transport.
- Existing target overwrite behavior must be safe and explicit in CLI output; do not silently merge export output into arbitrary existing directories without tests.

## Public API / Service Requirements

Add facade methods on `AdiyutantCoreService` or an equivalent public Core service boundary.

Required capabilities:

```text
validate_bundle(source) -> BundleValidationReportDto
preview_bundle_import(source, mode) -> BundlePreviewDto
apply_bundle_import(source, mode) -> BundleApplyReportDto
export_bundle(target) -> BundleExportReportDto
```

Exact Rust signatures may differ to fit path/bytes handling, but external behavior must match.

Errors must use existing `CoreError`/`CoreErrorDto` semantics; malformed user input should be `InvalidInput`, not `Internal`.

## CLI Requirements

Add commands:

```bash
adiyutant import validate <path>
adiyutant import preview <path> --mode append|merge
adiyutant import apply <path> --mode append|merge
adiyutant export bundle <path>
```

CLI output requirements:

- Include bundle id, schema version, status, error count, warning count, and section summary.
- Non-zero exit for invalid validation or failed apply.
- Zero exit for valid-with-warnings preview/apply unless apply fails.
- Deterministic output suitable for CLI smoke tests.

## Exact Files / Areas in Scope

Expected source areas for implementation agents:

- `/home/makc/AI_sandbox/ADIYUTANT/AdiyutantCore/Cargo.toml`
- `/home/makc/AI_sandbox/ADIYUTANT/AdiyutantCore/adiyutant_core/Cargo.toml`
- `/home/makc/AI_sandbox/ADIYUTANT/AdiyutantCore/adiyutant_cli/Cargo.toml`
- `/home/makc/AI_sandbox/ADIYUTANT/AdiyutantCore/adiyutant_core/src/bundle/` new module tree
- `/home/makc/AI_sandbox/ADIYUTANT/AdiyutantCore/adiyutant_core/src/model/import_run.rs` new model
- `/home/makc/AI_sandbox/ADIYUTANT/AdiyutantCore/adiyutant_core/src/model/context_document.rs`
- `/home/makc/AI_sandbox/ADIYUTANT/AdiyutantCore/adiyutant_core/src/model/checklist_template.rs`
- `/home/makc/AI_sandbox/ADIYUTANT/AdiyutantCore/adiyutant_core/src/model/mod.rs`
- `/home/makc/AI_sandbox/ADIYUTANT/AdiyutantCore/adiyutant_core/src/dto.rs`
- `/home/makc/AI_sandbox/ADIYUTANT/AdiyutantCore/adiyutant_core/src/store.rs`
- `/home/makc/AI_sandbox/ADIYUTANT/AdiyutantCore/adiyutant_core/src/service/` new/updated bundle service module
- `/home/makc/AI_sandbox/ADIYUTANT/AdiyutantCore/adiyutant_store/src/lib.rs`
- `/home/makc/AI_sandbox/ADIYUTANT/AdiyutantCore/adiyutant_cli/src/main.rs`
- `/home/makc/AI_sandbox/ADIYUTANT/AdiyutantCore/adiyutant_cli/src/commands/` new import/export command modules
- `/home/makc/AI_sandbox/ADIYUTANT/AdiyutantCore/adiyutant_cli/tests/cli_smoke.rs`
- `/home/makc/AI_sandbox/ADIYUTANT/docs/contracts/adiyutant-bundle-v1.md`
- `/home/makc/AI_sandbox/ADIYUTANT/docs/contracts/examples/adiyutant-bundle-basic/`

Dependency guidance:

- Default candidate YAML parser: `serde_yaml` or a maintained serde-compatible YAML crate.
- Default candidate zip parser/writer: `zip` crate.
- If dependency choice affects security checks, stop and document the blocker before weakening zip validation.

## Parallel Work Units

### Stage 0 — Parent integration setup

Owner: primary executor/orchestrator.

Responsibilities:

- Confirm this TZ and scope document are current.
- Decide exact YAML/zip dependencies.
- Define shared DTO/model names and module boundaries.
- Reserve shared files (`dto.rs`, `store.rs`, module declarations) for parent integration to prevent parallel edit conflicts.

### Stage 1 — Parallel units

#### Unit A — Contract docs and examples

Owned files/areas:

- `docs/contracts/adiyutant-bundle-v1.md`
- `docs/contracts/examples/adiyutant-bundle-basic/`

Deliverables:

- Bundle v1 contract with manifest, section schemas, validation matrix, zip security rules, and examples.
- Basic example bundle containing supported sections plus preview-only project/roadmap files.

Verification:

- Markdown review.
- Example bundle is used by parser/integration tests.

#### Unit B — Parser, validation, zip security

Owned files/areas:

- New `adiyutant_core/src/bundle/` files.
- Unit tests under `adiyutant_core`.

Deliverables:

- Directory and zip source parsing into `AdiyutantBundle`.
- Manifest validation.
- Zip security checks.
- Section validation and issue reporting.

Verification:

- Unit tests for valid bundle, missing manifest, bad schema, duplicate slugs, invalid enum, unknown files, path traversal, oversized files, unsupported extensions.

#### Unit C — Persistence, slugs, import history

Owned files/areas:

- `adiyutant_core/src/model/import_run.rs`
- `context_document.rs`, `checklist_template.rs`, `model/mod.rs`
- `store.rs`
- `adiyutant_store/src/lib.rs`

Deliverables:

- SQLite migration v3 or next available version.
- `import_runs` table.
- Stable slug/source fields and lookup/upsert support.
- Transaction seam for bundle apply.

Verification:

- Store integration tests using in-memory SQLite and migrated schema.
- Backward compatibility tests for existing records without slugs.

#### Unit D — Report DTOs and facade planning

Owned files/areas:

- `dto.rs` additions coordinated through parent.
- `service/bundle.rs` or equivalent.

Deliverables:

- Report DTOs.
- Shared preview/apply planner.
- Apply service using the same plan as preview.
- Export service.

Verification:

- Component tests with mock/noop store where possible.
- Service tests proving preview and apply action counts match before writes.

### Stage 2 — Parent integration

Owner: primary executor/orchestrator.

Responsibilities:

- Merge shared file edits.
- Wire CLI commands.
- Add CLI smoke tests.
- Run full workspace checks.

## Test Strategy

### Static checks

Required:

```bash
cargo fmt
cargo check --workspace
cargo clippy --all-targets -- -D warnings
```

Run from:

```text
/home/makc/AI_sandbox/ADIYUTANT/AdiyutantCore
```

### Unit tests

Required coverage:

- Manifest parsing.
- Section validation.
- Unknown field warning behavior.
- Duplicate slug/id detection.
- Planning concrete-day rejection.
- Zip path/security limit validation.
- Import planner append vs merge semantics.
- Export serialization for directory and zip where possible.

Command:

```bash
cargo test -p adiyutant_core
```

### Component tests

Required coverage:

- `AdiyutantCoreService::validate_bundle` returns structured reports.
- `preview_bundle_import` and `apply_bundle_import` use the same planner.
- Preview-only project/roadmap sections are warnings/actions, not persisted writes.
- CLI command handlers produce deterministic summaries.

Command:

```bash
cargo test --workspace bundle
```

### Integration tests with real dependencies

Required:

- SQLite in-memory tests for migration v3/import_runs/slug columns.
- SQLite transaction rollback test: intentionally fail one supported section and prove no partial context/checklist writes remain.
- Real zip archive tests, not only mocked zip entries.

Command:

```bash
cargo test -p adiyutant_store
```

### Real stand smoke tests

Required because this touches runtime behavior and CLI.

Stand:

- Database: SQLite file in `/tmp` or a test workspace temp directory.
- Lifecycle: create fresh file, run CLI commands, delete file after evidence is captured.
- Env var: `ADIYUTANT_DB_PATH`.
- Seed data: none for basic import; use `docs/contracts/examples/adiyutant-bundle-basic/` as input.
- Services: no long-running services.
- Health check: `adiyutant today` or a new import validation command after migration.
- Cleanup: remove temp DB and exported temp bundle after smoke.

Smoke sequence:

```bash
cargo run -p adiyutant_cli -- import validate ../docs/contracts/examples/adiyutant-bundle-basic
cargo run -p adiyutant_cli -- import preview ../docs/contracts/examples/adiyutant-bundle-basic --mode append
cargo run -p adiyutant_cli -- import apply ../docs/contracts/examples/adiyutant-bundle-basic --mode append
cargo run -p adiyutant_cli -- export bundle /tmp/adiyutant-export-test
```

The executor may adjust relative paths, but must report exact commands.

### UI automation

Not applicable in this TZ. No Android/web/desktop UI is implemented. Leave checklist item unchecked or mark with explicit N/A evidence in the completion report according to project workflow.

### User scenario tests

Required scenario:

```text
ChatGPT-assisted bundle directory → validate → preview → apply append → validate exported bundle
```

Evidence must show:

- Warnings for preview-only projects/roadmaps.
- Applied context/checklist sections.
- `import_runs` has one apply record.
- Exported bundle validates.

### Regression checks

Required:

```bash
cargo test --workspace
```

Also verify existing CLI smoke tests still pass.

### Acceptance review

Executor completion report must include evidence table:

| Check | Command / Tool | Result | Evidence |
|---|---|---|---|
| Static | `cargo fmt`, `cargo check --workspace`, `cargo clippy --all-targets -- -D warnings` | pass/fail | log/summary |
| Unit | `cargo test -p adiyutant_core` | pass/fail | log/summary |
| DB integration | `cargo test -p adiyutant_store` | pass/fail | migration/rollback note |
| Workspace regression | `cargo test --workspace` | pass/fail | log/summary |
| Stand smoke | CLI import/export sequence | pass/fail | DB path + command output |
| UI automation | N/A | skipped | no UI in TZ |

## Acceptance Criteria

1. Directory and `.adiyutant.zip` bundle inputs parse into one internal bundle representation.
2. Missing manifest, unsupported schema, duplicate slugs, missing required fields, invalid enums, unsafe zip paths, unsupported extensions, and oversize files are rejected with structured errors.
3. Unknown top-level files and unknown optional fields produce warnings without unsafe writes.
4. `preview` and `apply` use the same planner; apply report action counts match preview except for final status/timestamps.
5. `append` never updates existing slug-matched entities.
6. `merge` updates existing slug-matched entities and creates missing ones, but never deletes absent entities.
7. `projects/*` and `roadmaps/*` are validated/previewed and explicitly ignored on apply.
8. Bundle rules are stored as human context and are not wired into `LocalRuleGateway`.
9. `planning.yaml` cannot create dated `DayPlan` records.
10. Supported-section apply is transactional; partial writes are covered by an integration test.
11. `import_runs` records apply attempts/results with summary JSON.
12. Export creates a valid 8.4A bundle with preview-only project/roadmap capabilities.
13. CLI validate/preview/apply/export commands work against a real SQLite temp DB.
14. Existing `cargo fmt`, `cargo check`, `cargo clippy`, and `cargo test --workspace` pass.

## Risks and Mitigations

| Risk | Mitigation |
|---|---|
| Zip crate cannot safely detect symlinks/hardlinks | Treat as dependency blocker; do not weaken requirement silently. |
| Adding slug fields breaks old rows/JSON | Use nullable columns, serde defaults, and backward compatibility tests. |
| Preview/apply logic drifts | Use one planner function for both operations; test action parity. |
| Transaction hidden behind service-level individual writes | Add explicit Store transaction/composite seam and rollback tests. |
| YAML becomes executable rule language | Store rules as context only; keep Rule Engine v2 out of scope. |
| Project/roadmap data gets stuffed into context docs | Enforce preview-only status for those sections in this TZ. |

## Architecture Review

- Review file: `/home/makc/AI_sandbox/ADIYUTANT/docs/reviews/architecture-review-phase-8.4A-portable-bundle-contract.md`
- Verdict: Approved with conditions.
- Blockers: none after this TZ includes zip security, transactionality, stable slugs, report/history, and preview-only project/roadmap handling.

Executor must read the review before implementation and keep its warnings tracked in the completion report.
