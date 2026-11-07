---
name: write-prd
description: Draft or update the Product Requirements Document in docs/prd/PRD.md. Use when defining what the calculator API must do, adding a feature, or when a requirement is unclear.
---

# Write the PRD

The PRD says **what** the system does and **what it does not**. No technology
choices here; those go in ADRs.

## Process

1. Use superpowers `brainstorming`: ask the human one question at a time until
   the problem, the users and the scope are clear. Prefer multiple-choice
   questions. Do not invent answers.
2. Set the **boundary** first: list what is in scope, out of scope, and the
   external actors (who calls the API, what it depends on).
3. Write numbered requirements. Each one must be testable: if you cannot write
   an acceptance check for it, it is not a requirement yet.
4. Record every answer you assumed rather than heard under *Open questions*.
5. Show the draft to the human for review (see `review-spec`) before marking it
   `Accepted`.

## Template

```markdown
# PRD: Calculator API
Status: Draft | Accepted   ·   Last reviewed: YYYY-MM-DD

## 1. Problem and goal
## 2. Users and use cases
## 3. Scope
### In scope
### Out of scope (and why)
## 4. Functional requirements
| ID | Requirement | Acceptance criteria | Priority (MoSCoW) |
## 5. Non-functional requirements
| ID | Requirement | Measure |
## 6. Error behaviour
(invalid input, division by zero, overflow, precision: what the caller gets back)
## 7. Open questions and assumptions
## 8. Glossary
```

## Checklist before handing over

- [ ] Every FR has an acceptance criterion a test can check.
- [ ] Edge cases are explicit: division by zero, overflow, NaN/Infinity,
      precision and rounding, very large inputs, malformed requests.
- [ ] Out-of-scope list is as concrete as the in-scope list.
- [ ] No implementation detail (frameworks, libraries) leaked in.
