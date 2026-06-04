# INDEX

## Repository Overview

```
Adiyutant/
  README.md              Project overview
  SPECIFICATION.md       Product specification
  ROADMAP.md             Development roadmap
  TASKS.md               Task tracking
  SOLUTION_MAP.md        Project responsibility map
  ARCHITECTURE.md        Architecture overview
  INDEX.md               This file — navigation
  AI_CONTEXT.md          AI agent context
  AI_ENTRY_POINTS.md     AI agent reading guide
  MEMORY.md              Stable project facts
  AGENTS.md              Agent working rules
  SECURITY_NOTES.md      Security/privacy notes
  GLOSSARY.md            Domain terminology

  docs/
    core-boundary.md     Core boundary definition
    agent-gateway.md     Agent Gateway definition
    sync-protocol.md     Future sync notes
    adr/                 Architecture Decision Records (planned)

  AdiyutantCore/         Rust local-first core (planned)
  AdiyutantAndroid/      Android UI shell (planned)
  AdiyutantWeb/          Web/PWA client (planned)
  AdiyutantDesktop/      Desktop UI shell (planned)
```

## Root Documents

| File | Read First |
|------|------------|
| README.md | Yes — start here |
| SPECIFICATION.md | Yes — product definition |
| ARCHITECTURE.md | Yes — system architecture |
| ROADMAP.md | If interested in development plan |
| TASKS.md | If working on a task |
| SOLUTION_MAP.md | If understanding project boundaries |
| AI_CONTEXT.md | If you are an AI agent |
| AGENTS.md | If you are an AI agent |

## Project Directories

| Directory | Status |
|-----------|--------|
| AdiyutantCore/ | Planned — no code yet |
| AdiyutantAndroid/ | Planned — no code yet |
| AdiyutantWeb/ | Planned — no code yet |
| AdiyutantDesktop/ | Planned — no code yet |

## Docs Directory

| File | Content |
|------|---------|
| docs/core-boundary.md | Core vs platform responsibilities |
| docs/agent-gateway.md | AI Gateway design notes |
| docs/sync-protocol.md | Future sync protocol notes |

## Architecture Decisions

ADR directory exists at `docs/adr/`. No ADRs have been created yet. ADR baseline is planned for TASK-0002.

## Current Priority

Bootstrap the repository structure and documentation (TASK-0000). No implementation work yet.

## Open Uncertainties

- Android-Core integration method
- Desktop technology choice
- Web integration method
- Agent Gateway implementation
- Sync protocol design
