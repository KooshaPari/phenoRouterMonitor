---
audience: [developers, agents]
---

# Implementation Strategy

## Strategy

Do not mutate `/Users/kooshapari/CodeProjects/archive/TripleM` until its current dirty state is
preserved.

## Safe Sequence

1. snapshot or stash current dirty state
2. restore tracked files from `HEAD`
3. reinstall dependencies
4. only then investigate whether the purgecss experiment should be replayed intentionally
