---
name: implement-task
description: Implement one task from docs/plan/TASKS.md test-first, with a minimal diff and one commit. Use when generating code from the plan.
---

# Implement a task

Builds on superpowers `executing-plans` and `test-driven-development`, with
ponytail active for the code itself.

## Steps

1. Pick the next task whose dependencies are done. Read the FRs and ADRs it
   references; do not rely on memory of them.
2. **Red:** write the test(s) that encode the task's *Done when*. Run them and
   watch them fail for the right reason.
3. **Green:** write the smallest code that passes. Stdlib and existing code
   first; no options or abstractions the task does not need.
4. **Refactor** only what this task touched. Run the full test suite and the
   linters.
5. Mark the task done in `TASKS.md` in the same commit.
6. Commit: `feat(T-07): <what>`, body names the FR/ADR IDs and anything skipped.

## Stop and ask when

- The task needs behaviour the PRD does not define.
- The code would contradict an ADR.
- A test can only pass by weakening or skipping another test.

In those cases update the docs (or propose an ADR) first; never quietly diverge.

## Never

- Skip, delete or loosen a test to get green.
- Add a dependency the plan or an ADR does not name.
- Cut input validation or error handling to make the diff smaller.
