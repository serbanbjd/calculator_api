# Agent guide: Calculator API

This repo is built with **spec-driven development**. The docs are the source of
truth; code is an implementation of them. Read this file before any change.

## Golden rules

1. **No code without a spec.** Every change traces to a PRD requirement
   (`docs/prd/`), respects the ADRs (`docs/adr/`), and implements a task in
   `docs/plan/`. If none fits, stop and update the docs first.
2. **Docs change before code.** When reality disagrees with a spec, propose the
   doc change, get it reviewed, then change code.
3. **Test first.** Write the failing test that encodes the requirement, then the
   code (superpowers `test-driven-development`).
4. **Smallest complete change.** Follow ponytail: stdlib and existing code first,
   no speculative options or abstractions. Never cut input validation, error
   handling or security.
5. **Small commits that tell the story.** One logical step per commit,
   Conventional Commits (`feat:`, `fix:`, `docs:`, `test:`, `chore:`), and the
   commit body says *why*. Reference IDs (`FR-3`, `ADR-0002`, `T-05`).
6. **Verify before claiming done.** Run the tests and linters and show the
   output (superpowers `verification-before-completion`).

## Workflow and which skill to use

| Step | Output | Project skill | Library skills it builds on |
|------|--------|---------------|-----------------------------|
| 1. Requirements | `docs/prd/PRD.md` | `write-prd` | superpowers `brainstorming` |
| 2. Architecture | `docs/adr/NNNN-*.md` | `write-adr` | superpowers `brainstorming` |
| 3. Plan | `docs/plan/PLAN.md`, `TASKS.md`, UML | `plan-tasks` | superpowers `writing-plans` |
| 4. Code | `src/`, `tests/` (Rust) | `implement-task` | superpowers `executing-plans`, `test-driven-development`; ponytail |
| Any doc | review notes | `review-spec` | superpowers `requesting-code-review` |

## Conventions

- IDs are stable and never reused: `FR-n` (functional), `NFR-n` (non-functional),
  `ADR-NNNN`, `T-nn` (tasks). Removed items are marked *withdrawn*, not deleted.
- Each doc starts with a status line: `Draft`, `Accepted`, `Superseded by …`.
- Diagrams are Mermaid in Markdown so they render on GitHub and diff as text.
- Known shortcuts get a code comment: `shortcut: <limit>, <when to upgrade>`
  (ponytail convention).

## Stack

Rust (stable, edition 2024), built with Cargo. Libraries (HTTP framework,
serialization, test helpers) are chosen in ADRs, not added ad hoc.

## Commands

Run all of these before every commit; all must pass.

```sh
cargo fmt --check              # formatting
cargo clippy --all-targets -- -D warnings  # lints incl. tests
cargo test                     # unit + integration tests
cargo run                      # start the API locally
```

Unit tests live next to the code in `#[cfg(test)] mod tests`; API-level tests
live in `tests/`.

## Boundaries for the agent

- Do not add dependencies without an ADR or a task that names them.
- Do not change public API shapes without updating the PRD and the API contract.
- Ask the human when a requirement is ambiguous instead of inventing behaviour;
  record the answer in the PRD's open-questions section.
