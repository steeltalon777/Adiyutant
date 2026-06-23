# Adiyutant Bundle v1 — Portable Import/Export Contract

| Field | Value |
|---|---|
| Status | Current |
| Date | 2026-06-22 |
| Bundle format | Adiyutant Bundle v1 |
| Transport | `.adiyutant.zip` or unpacked directory |
| Source TZ | `docs/tz/phase-8.4A-portable-bundle-contract.md` |
| Scope | Phase 8.4A only |

## 1. Purpose

An Adiyutant Bundle v1 is the portable container that moves a user's Life Core, routines, human-readable rules, checklist templates, planning preferences, and Markdown context documents between an assistant (for example a ChatGPT-assisted session) and a local Adiyutant Core install. A bundle can also carry preview-only Project and Roadmap YAML, which Core parses and reports on but does not persist in 8.4A. Core reads a bundle through one of three `BundleSource` forms (directory, zip file, zip bytes), validates it against the manifest, plans an import, and either returns a structured report (validate, preview), applies the plan in a single transaction (apply, append/merge), or serialises the supported Core state back out (export).

## 2. Directory layout

Both the unpacked directory and the entries inside a `.adiyutant.zip` use the same relative layout. The manifest is the authoritative map of which sections exist; the layout below shows every section Core recognises in 8.4A.

```text
adiyutant-pack/
  manifest.yaml              # required, authoritative
  life-core.yaml             # optional, applied as ContextDocument
  routines.yaml              # optional, applied as routine preferences
  rules.yaml                 # optional, applied as human context only
  checklists.yaml            # optional, applied as ChecklistTemplate
  planning.yaml              # optional, preferences/templates/hints only
  context/                   # optional directory
    personal.md              # applied as ContextDocument
    work.md                  # applied as ContextDocument
    health.md                # applied as ContextDocument
  projects/                  # optional directory, preview-only in 8.4A
    adiyutant.yaml
    warehouse.yaml
  roadmaps/                  # optional directory, preview-only in 8.4A
    adiyutant-roadmap.yaml
    warehouse-roadmap.yaml
```

| Entry | Required | Apply behaviour (8.4A) |
|---|---|---|
| `manifest.yaml` | Yes | Authoritative identity/capabilities/section map. |
| `life-core.yaml` | No | Applied as `Vec<ContextDocument>`. |
| `routines.yaml` | No | Applied as routine preferences. |
| `rules.yaml` | No | Applied as human context; never wired to `LocalRuleGateway`. |
| `checklists.yaml` | No | Applied as `Vec<ChecklistTemplate>` with embedded `ChecklistItem`s. |
| `planning.yaml` | No | Applied as preferences/templates/candidate hints only. Never creates a dated `DayPlan`. |
| `context/*.md` | No | Applied as `ContextDocument` records. |
| `projects/*.yaml` | No | Parsed and previewed only. Ignored on apply. |
| `roadmaps/*.yaml` | No | Parsed and previewed only. Ignored on apply. |
| Any other top-level file | n/a | Warning: unknown top-level file, never applied. |

Sections absent from the directory and absent from `manifest.yaml` are not present. Sections declared in `manifest.yaml` but missing on disk are reported as errors during validation.

## 3. `manifest.yaml` schema

`manifest.yaml` is required and authoritative. Core must not guess which files are important when the manifest is present. Unknown top-level fields and unknown optional fields generate warnings and are ignored.

```yaml
schema_version: "adiyutant.bundle.v1"   # required, exact string
bundle_id: "max-core-2026-06"           # required, stable per bundle
title: "Max Life Core"                  # required, human title
created_at: "2026-06-22T10:00:00Z"      # required, ISO-8601 UTC

capabilities:
  projects: "preview-only"              # required, only "preview-only" in 8.4A
  roadmaps: "preview-only"              # required, only "preview-only" in 8.4A

sections:
  life_core: "life-core.yaml"           # optional, relative file path
  routines: "routines.yaml"             # optional
  rules: "rules.yaml"                   # optional
  checklists: "checklists.yaml"         # optional
  planning: "planning.yaml"             # optional
  context_dir: "context/"               # optional, relative directory
  projects_dir: "projects/"             # optional
  roadmaps_dir: "roadmaps/"             # optional
```

| Field | Required | Type | Notes |
|---|---|---|---|
| `schema_version` | Required | String | Must equal `"adiyutant.bundle.v1"` exactly. Any other value is an Error. |
| `bundle_id` | Required | String | Stable per bundle. Used in import history and reports. |
| `title` | Required | String | Human-readable title. Used in CLI output and `import_runs.bundle_title`. |
| `created_at` | Required | String (ISO-8601) | UTC timestamp. Not used for merge logic. |
| `capabilities.projects` | Required | String enum | Only `"preview-only"` in 8.4A. Stored verbatim in the report. |
| `capabilities.roadmaps` | Required | String enum | Only `"preview-only"` in 8.4A. Stored verbatim in the report. |
| `sections.life_core` | Optional | Relative path | Points at a single YAML file. |
| `sections.routines` | Optional | Relative path | Points at a single YAML file. |
| `sections.rules` | Optional | Relative path | Points at a single YAML file. |
| `sections.checklists` | Optional | Relative path | Points at a single YAML file. |
| `sections.planning` | Optional | Relative path | Points at a single YAML file. |
| `sections.context_dir` | Optional | Relative directory | Must end in `/`. |
| `sections.projects_dir` | Optional | Relative directory | Must end in `/`. |
| `sections.roadmaps_dir` | Optional | Relative directory | Must end in `/`. |
| Any other top-level field | n/a | Any | Warning, ignored. |

## 4. Validation matrix

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

Implementation note: warnings for unknown optional fields require parsing YAML into an inspectable value tree (or equivalent) before strict typed deserialization, so that the parser can detect extra keys without dropping them silently.

## 5. Section behaviour (8.4A)

| Section | Apply | Preview only |
|---|---|---|
| `life-core.yaml` | Apply as `ContextDocument` records. | No |
| `routines.yaml` | Apply as routine preferences. | No |
| `rules.yaml` | Apply as human-readable rules/context. Never mapped to `LocalRuleGateway`. | No |
| `checklists.yaml` | Apply as `ChecklistTemplate` with embedded `ChecklistItem`s. | No |
| `planning.yaml` | Apply preferences/templates/candidate hints only. Never creates a dated `DayPlan`. | No |
| `context/*.md` | Apply as `ContextDocument` records. | No |
| `projects/*` | Validate and preview only. Ignored on apply with explicit report action. | Yes |
| `roadmaps/*` | Validate and preview only. Ignored on apply with explicit report action. | Yes |

## 6. Section schemas

### 6.1 `life-core.yaml`

Top-level shape: optional `profile` section and `context_documents` array. Each `context_documents` entry is parsed into a `ContextDocument`. Optional fields default on the Core side.

```yaml
profile:
  name: "Alex"                    # optional, free-text
  bio: "Software engineer"        # optional, free-text
  values:                         # optional, list of free-text
    - "Health first"
  goals:                          # optional, list of free-text
    - "Run a marathon"

context_documents:
  - slug: "life-purpose"          # stable, used for merge
    title: "Life Purpose"         # required
    doc_type: "life_core"         # see ContextDocumentType enum
    content: |                    # Markdown body
      # My Life Purpose
      Build tools that help people live intentionally.
```

`ContextDocumentType` values accepted in 8.4A: `core`, `goals`, `rules`, `routine`, `health`, `work`, `custom`, `life_core`, `recovery_protocol`, `tone`, `planning_preferences`. Any other value is an Error.

Top-level `context_documents` entries use `slug` as the stable merge key and `content` as the Markdown body field.

### 6.2 `routines.yaml`

```yaml
routines:
  - slug: "morning-routine"        # stable, used for merge
    title: "Morning Routine"
    description: "Start the day right"  # optional
    steps:
      - "Wake up at 6:30"
      - "Drink water"
```

### 6.3 `rules.yaml`

```yaml
rules:
  - slug: "deep-work-first"        # stable, used for merge
    title: "Deep work first"
    description: "Always complete one deep work block before checking messages"
    category: "productivity"       # optional
```

`rules.yaml` is stored as human context. It is never mapped to `LocalRuleGateway`. A future Rule Engine v2 may interpret it; that work is out of scope for 8.4A.

### 6.4 `checklists.yaml`

Top-level shape: `templates` array of `ChecklistTemplate` entries, each with embedded `ChecklistItem`s.

```yaml
templates:
  - slug: "morning-ready"
    title: "Morning Ready"
    category: "morning"
    items:
      - slug: "dressed"
        question: "Am I dressed and ready?"
        kind: "Checkbox"
        order: 1
      - slug: "energy-level"
        question: "Energy level?"
        kind: "Scale"
        options: ["1", "2", "3", "4", "5"]
        order: 2
```

`ChecklistItemKind` accepted values: `Checkbox`, `Choice`, `Scale`, `Text`, `OptionalComment`, `HabitEvent`, `TaskLink`, `TimerStart`. `slug` is optional on `ChecklistItem` for backward compatibility, but recommended for stable identity within a template.

### 6.5 `planning.yaml`

```yaml
default_mode: "manual_review"
daily_capacity:
  deep_tasks_max: 3
  light_tasks_max: 5

templates:
  - id: "normal-workday"
    title: "Normal Workday"
    blocks:
      - kind: "morning"
        start: "06:30"
      - kind: "deep_work"
        start: "09:00"
        duration_minutes: 120

candidate_hints:
  - "Prefer project tasks before random backlog"
```

### 6.6 `context/*.md`

Each file is a Markdown body. Filename without extension is the stable slug. Title is taken from the first H1 when present, otherwise from the slug.

```markdown
# Personal

Sleep target: 7.5h.
Move body every day.
One uninterrupted deep block in the morning.
```

### 6.7 `projects/*.yaml` (preview-only)

```yaml
# preview-only — ignored on apply in 8.4A
slug: "adiyutant"
title: "Adiyutant"
phases:
  - slug: "mvp"
    title: "MVP"
```

The parser keeps the entire YAML value tree for each `projects/*.yaml` file and exposes it through the report. Persistence is out of scope for 8.4A and is delivered in 8.4B.

### 6.8 `roadmaps/*.yaml` (preview-only)

```yaml
# preview-only — ignored on apply in 8.4A
slug: "adiyutant-roadmap"
project_slug: "adiyutant"
phases:
  - slug: "mvp"
    title: "MVP"
    items:
      - slug: "bundle-v1"
        title: "Bundle v1 contract"
```

The parser keeps the entire YAML value tree for each `roadmaps/*.yaml` file and exposes it through the report. Persistence is out of scope for 8.4A and is delivered in 8.4B.

## 7. Planning semantics

`planning.yaml` carries preferences, templates, and candidate hints only. It does not create a dated `DayPlan`. The validator must reject any of the following forbidden fields with a clear Error message at the planning section:

```yaml
# Forbidden in 8.4A
date: "2026-06-22"
tasks:
  - title: "Concrete imported task"
```

as persisted `DayPlan` data. Pre-8.4A bundles that include `date` or `tasks` keys inside `planning.yaml` are rejected at the planning section during validation.

## 8. Bundle rules

Bundle rules are stored as human context only. They are not mapped to `LocalRuleGateway` and they are not evaluated. The current separation is:

```text
Bundle rules != LocalRuleGateway rules
```

Future Rule Engine v2 may add:

```text
human rules -> structured rules -> local suggestions
```

That future work is out of scope for Phase 8.4.

## 9. Import modes

| Mode | Behaviour |
|---|---|
| `validate` | Parse + validate. Returns a `BundleValidationReportDto`. No writes. |
| `preview` | Parse + validate + compute import plan. Returns a `BundlePreviewDto`. No writes. |
| `apply --mode append` | Parse + validate + compute import plan + writes. Slug/id conflicts do not update existing entities and are reported. |
| `apply --mode merge` | Parse + validate + compute import plan + writes. Slug/id matches update existing entities. Absent entities are never deleted. |
| `export` | Write a new bundle from supported Core state. Returns a `BundleExportReportDto`. |

`preview` and `apply` must use the same internal planner so the apply report's action counts match the preview report. `replace` is rejected. `validate` and `preview` may return reports without persisting any state. `apply` must persist one `import_runs` row.

## 10. Zip security requirements

`.adiyutant.zip` input is untrusted. Acceptance tests must cover malicious archives. The mandatory checks, in order:

1. Reject absolute paths.
2. Reject paths containing `..` path traversal.
3. Reject symlinks.
4. Reject hardlinks, if the chosen zip library exposes them.
5. Enforce max file count: 200.
6. Enforce max uncompressed size: 10 MB.
7. Enforce max single YAML file size: 1 MB.
8. Enforce max single Markdown file size: 2 MB.
9. Allow only `.yaml`, `.yml`, and `.md` extensions.
10. Allow only declared/root-approved files and directories.
11. Reject hidden executable-ish junk unless explicitly allowed by the bundle contract.

If the selected zip library cannot expose enough metadata to reject symlinks and hardlinks safely, the implementation must stop and report the dependency blocker instead of silently weakening this requirement.

## 11. Stable identity and merge keys

| Model | Stable key | Notes |
|---|---|---|
| `ChecklistTemplate` | `slug` | Nullable for backward compatibility. Uniqueness must tolerate null existing values. |
| `ChecklistItem` | `slug` inside the template's embedded JSON | Optional for backward compatibility. Stable within a template. |
| `ContextDocument` | `source_slug` | Optional for backward compatibility. `title + doc_type` may be used as a documented temporary fallback if the executor proves schema change is too risky for 8.4A. |
| `ContextDocument` (fallback) | `title + doc_type` | Used only when the executor explicitly opts in and documents it. |
| `RoutineEntry` | `slug` | Required in YAML. |
| `RuleEntry` | `slug` | Required in YAML. Stored as context. |
| `PlanningTemplate` | `id` | Required in YAML. Applied as preference/template. |

Existing rows without slugs must remain readable and writable. Recommended SQLite migration behaviour: add nullable slug/source columns, add unique indexes that tolerate existing null values, do not backfill slugs destructively, add tests for old rows without slugs.

## 12. Reports and history

Every validate, preview, apply, and export operation returns a structured report DTO. The minimum DTO set is:

```text
BundleValidationReportDto
BundlePreviewDto
BundleApplyReportDto
BundleExportReportDto
BundleIssueDto
BundleSectionReportDto
BundleActionDto
```

The minimum fields on every report are:

```text
bundle_id
schema_version
status          # valid | valid_with_warnings | invalid | applied | failed
errors[]
warnings[]
sections[]
actions[]
```

The minimum action kinds are:

```text
create
update
skip_existing
ignored_preview_only
conflict
error
```

Apply history is persisted to SQLite in the `import_runs` table. Only `apply` must persist `import_runs`. `validate` and `preview` may return reports without persistence.

```text
import_runs
  id              # uuid
  bundle_id
  bundle_title
  schema_version
  mode            # append | merge
  status          # applied | failed
  started_at
  finished_at
  summary_json    # serialised BundleApplyReportDto (or validation report on failure)
```

## 13. Transactionality

Apply is all-or-nothing for supported sections. The required pipeline is:

```text
parse -> validate entire bundle -> preview entire bundle -> apply supported sections in transaction
```

Rules:

- Apply proceeds only when preview has no errors.
- Warnings may exist, including preview-only project and roadmap sections.
- Unsupported preview-only sections are skipped with explicit report actions and do not roll back the supported sections.
- If any supported-section write fails, context, checklist, routine, rule, and planning writes plus the `import_runs` row must be consistent with rollback or failure reporting.
- Apply is not allowed to be a sequence of independent `Store` calls from the service layer without a transaction boundary. The Store must expose a bounded transaction seam, either as a composite import apply method or as a generic transaction API.

## 14. Export scope

Phase 8.4A export writes only supported sections:

```text
manifest.yaml
life-core.yaml
routines.yaml
rules.yaml
checklists.yaml
planning.yaml
context/*.md
```

Export must not write project or roadmap content before 8.4B. The exported manifest advertises:

```yaml
capabilities:
  projects: "preview-only"
  roadmaps: "preview-only"
```

Export targets behave as follows:

- Directory path: write the unpacked bundle directory.
- `.adiyutant.zip` path: write a zip transport.
- Existing-target overwrite behaviour must be safe and explicit in CLI output. The CLI must not silently merge export output into an arbitrary existing directory without tests.

## 15. CLI commands

The CLI exposes four command shapes. Exact Rust signatures may differ to fit path and bytes handling, but the external behaviour matches.

```bash
adiyutant import validate <path>
adiyutant import preview <path> --mode append|merge
adiyutant import apply <path> --mode append|merge
adiyutant export bundle <path>
```

CLI output requirements:

- Include bundle id, schema version, status, error count, warning count, and section summary.
- Non-zero exit for invalid validation or failed apply.
- Zero exit for valid-with-warnings preview or apply unless apply fails.
- Deterministic output suitable for CLI smoke tests.

`ADIYUTANT_DB_PATH` is the environment variable used by smoke tests to point at a temporary SQLite file.

## 16. Out of scope (8.4A)

- Android implementation.
- Cloud or backend sync.
- Executable Rule Engine v2 or `LocalRuleGateway` mapping.
- Destructive `replace` import mode.
- Concrete dated `DayPlan` import from `planning.yaml`.
- Project or roadmap persistence (delivered in 8.4B).
- LLM provider calls from Core.
- API key storage in client projects.

## 17. Non-goals reminder

Core does not call LLM providers directly. Mobile clients must not store LLM provider API keys. Core stores meaning; platforms execute OS-specific behaviour. These rules are part of `docs/core-boundary.md` and remain in force.

## 18. Example bundle

A complete example that exercises every section in this contract lives at `docs/contracts/examples/adiyutant-bundle-basic/`. The example is `Current` and may be used as a parser/integration test input.
