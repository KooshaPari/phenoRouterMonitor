# agentapi-plusplus Pilot Guide

agentapi-plusplus is a Go project. Go uses the go proxy for module distribution via git tags.

## Bootstrap

```bash
cd /path/to/agentapi-plusplus
pheno-cli bootstrap --lang go --registry go_proxy
```

## Expected Outputs

### mise.toml

```toml
[tools]
go = "1.22"
```

### CI Workflows

`.github/workflows/ci.yml` — runs on every push/PR:
- `go build ./...`
- `go test ./...`
- `go vet ./...`
- `staticcheck ./...`

`.github/workflows/release.yml` — runs on version tags matching `v*-alpha.*`:
- `go test ./...`
- `git tag` pushed to trigger go proxy indexing

### Git Hooks

`pre-commit`:
- Runs `go vet ./...`
- Runs `gofmt -l .`

`commit-msg`:
- Enforces conventional commit format

## Alpha Publish via Git Tag

Go modules are distributed via git tags and the go proxy — there is no explicit publish command.

1. Bump version to alpha:
   ```bash
   pheno-cli version bump --channel alpha --increment 1
   # Outputs: v1.0.0-alpha.1
   ```

2. Tag and push:
   ```bash
   git tag v1.0.0-alpha.1
   git push origin main --tags
   ```

3. The go proxy automatically indexes the tag. Consumers can then:
   ```bash
   go get github.com/org/agentapi-plusplus@v1.0.0-alpha.1
   ```

## Expected Pre-release Format

```
v1.0.0-alpha.1
```

Go uses SemVer with a `v` prefix. pheno-cli generates this via the `go_proxy` registry target.

## Validation

```bash
pheno-cli validate --repo .
```

Expected output:
```
[PASS] mise.toml exists
[PASS] CI workflows installed
[PASS] Git hooks installed
[PASS] go.mod module path verified
[PASS] All checks passed
```
