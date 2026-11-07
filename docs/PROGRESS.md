# Homework progress

Status of the Calculator API homework. Tick a box only when its *Done when* is
true and the work is merged to `main`. Each step is reviewed by Serban through
a pull request before the next step builds on it.

Timebox: ~4 h · Deadline: Tue 2026-11-10

## 1. Project setup and agent skills

- [x] Repo skeleton, README stub, `docs/` folders
- [x] `superpowers` and `ponytail` declared as agent libraries (`.claude/settings.json`)
- [x] Agent guide (`CLAUDE.md`, `AGENTS.md`)
- [x] Project skills: `write-prd`, `write-adr`, `plan-tasks`, `implement-task`, `review-spec`
- [x] Rust crate scaffold; `cargo fmt`, `clippy`, `test` pass

*Done when:* an agent opening the repo knows the workflow and the commands.

## 2. PRD: what the calculator covers

- [x] Scope decided with Serban (operations, in/out of scope)
- [x] `docs/prd/PRD.md` drafted with `write-prd`
- [x] Self-reviewed with `review-spec`; findings fixed
- [x] Reviewed and accepted by Serban (PR merged, status `Accepted`)

*Done when:* every FR has a testable acceptance criterion and open questions are answered or listed.

## 3. ADRs: architecture

- [x] ADR-0001 language and HTTP framework
- [x] ADR-0002 API contract and error model
- [x] ADR-0003 number representation and precision
- [x] ADR-0004 code structure (domain vs HTTP layer)
- [x] ADR-0005 testing strategy
- [x] Self-reviewed with `review-spec`; accepted by Serban (PR #3)

*Done when:* every library the code will use is named in an accepted ADR.

## 4. Plan, tasks and UML

- [x] `docs/plan/PLAN.md` with component and sequence diagrams (Mermaid)
- [x] `docs/plan/TASKS.md` with FR → task coverage in both directions
- [x] Self-reviewed with `review-spec`; accepted by Serban (PR #4)

*Done when:* every FR maps to a task and every task has a checkable *Done when*.

## 5. Code

- [x] Walking skeleton: one endpoint end to end with a test
- [x] All tasks in `TASKS.md` implemented test-first, one commit each
- [x] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` green
- [x] Manual smoke test with `curl` against `cargo run`

*Done when:* all tasks are ticked in `TASKS.md` and CI-equivalent checks pass.

## 6. README answers

- [x] How the system boundaries were defined
- [x] How the PRD was gathered
- [x] When plans, tasks and PRD can be removed, and what follows
- [x] How AI-generated docs are reviewed (critical places)
- [x] When team review should happen
- [x] Test types added and why; legacy vs green-field preference

*Done when:* README answers all six questions in short, honest prose.

## 7. Hand-in

- [ ] Default branch on GitHub is `main`
- [ ] History reads as a story (small, well-described commits)
- [ ] Repo link sent to the interviewer
