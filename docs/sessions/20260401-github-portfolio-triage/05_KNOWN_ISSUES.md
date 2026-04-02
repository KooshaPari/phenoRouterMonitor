# Known Issues

- the shelf `docs/WORKLOG.md` checkout is currently merge-conflicted and was intentionally not
  edited during remote repo mutation work
- many active repos still need required-check-name mapping before CI enforcement can be fully
  encoded in rulesets
- delete-wave execution is blocked until `gh` auth has `delete_repo` scope
- `Socket.dev` does not yet have a clear shelf-local reusable workflow source in the sampled repos
- `Chromatic` and `Sentry` should be applied selectively by repo type rather than forced into every
  active repo
- multiple canonical repos are currently dirty on active branches, so the next PR wave requires
  branch/file-scope cleanup before safe closeout
- `heliosCLI` has an unresolved stash that needs explicit triage before branch cleanup
- several secondary worktrees are present, and at least two are already marked `prunable`; they
  need an ownership pass before cleanup
