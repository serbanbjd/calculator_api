---
name: write-adr
description: Record an architecture decision as an ADR in docs/adr/. Use when choosing a language, framework, API style, error format, number representation, layering, testing or deployment approach.
---

# Write an ADR

One decision per file: `docs/adr/NNNN-short-title.md`, numbered in order. ADRs
are immutable once accepted; a change of mind is a new ADR that supersedes the
old one.

## Process

1. Name the decision as a question ("How do we represent numbers?").
2. Tie it to the PRD: which FR/NFR forces this decision?
3. List at least two real options, including "do nothing / simplest thing"
   (ponytail: the stdlib or existing option is the default to beat).
4. Decide, and write the consequences honestly, including the bad ones.
5. Human review (see `review-spec`) before `Accepted`.

## Template

```markdown
# ADR-NNNN: <title>
Status: Proposed | Accepted | Superseded by ADR-XXXX   ·   Date: YYYY-MM-DD
Drivers: FR-…, NFR-…

## Context
## Options considered
### Option A — …
Pros / Cons
### Option B — …
Pros / Cons
## Decision
## Consequences
(positive, negative, and what we must now do or watch)
```

## Decisions this project needs at minimum

- Language, framework and runtime.
- API style and contract (endpoints, request/response shape, OpenAPI).
- Number representation and precision (float vs decimal, limits).
- Error model (status codes, error body format).
- Code structure and layering (where the math lives vs the HTTP layer).
- Testing strategy (which test types, what each one proves).
