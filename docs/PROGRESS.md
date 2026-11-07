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
- [ ] Reviewed and accepted by Serban (PR merged, status `Accepted`)

*Done when:* every FR has a testable acceptance criterion and open questions are answered or listed.

## 3. ADRs: architecture

- [ ] ADR-0001 language and HTTP framework
- [ ] ADR-0002 API contract and error model
- [ ] ADR-0003 number representation and precision
- [ ] ADR-0004 code structure (domain vs HTTP layer)
- [ ] ADR-0005 testing strategy
- [ ] Self-reviewed with `review-spec`; accepted by Serban

*Done when:* every library the code will use is named in an accepted ADR.

## 4. Plan, tasks and UML

- [ ] `docs/plan/PLAN.md` with component and sequence diagrams (Mermaid)
- [ ] `docs/plan/TASKS.md` with FR → task coverage in both directions
- [ ] Self-reviewed with `review-spec`; accepted by Serban

*Done when:* every FR maps to a task and every task has a checkable *Done when*.

## 5. Code

- [ ] Walking skeleton: one endpoint end to end with a test
- [ ] All tasks in `TASKS.md` implemented test-first, one commit each
- [ ] `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test` green
- [ ] Manual smoke test with `curl` against `cargo run`

*Done when:* all tasks are ticked in `TASKS.md` and CI-equivalent checks pass.

## 6. README answers

- [ ] How the system boundaries were defined
- [ ] How the PRD was gathered
- [ ] When plans, tasks and PRD can be removed, and what follows
- [ ] How AI-generated docs are reviewed (critical places)
- [ ] When team review should happen
- [ ] Test types added and why; legacy vs green-field preference

*Done when:* README answers all six questions in short, honest prose.

## 7. Hand-in

- [ ] Default branch on GitHub is `main`
- [ ] History reads as a story (small, well-described commits)
- [ ] Repo link sent to the interviewer
