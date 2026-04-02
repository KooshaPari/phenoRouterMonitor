package standards

import (
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"strings"
	"time"
)

// Manifest represents the standards manifest
type Manifest struct {
	Version       string    `json:"version"`
	InstalledAt   time.Time `json:"installed_at"`
	UpdatedAt     time.Time `json:"updated_at"`
	Pillars       []string  `json:"pillars"`
	Files         []File    `json:"files"`
	GovernanceURL string    `json:"governance_url"`
}

// File represents a managed file
type File struct {
	Path     string `json:"path"`
	Hash     string `json:"hash"`
	Modified bool   `json:"modified"`
	Pillar   string `json:"pillar"`
}

func getGovernancePath() string {
	home, _ := os.UserHomeDir()
	return filepath.Join(home, ".pheno", "governance")
}

func getLocalGovPath() string {
	return "/Users/kooshapari/CodeProjects/Phenotype/repos/phenotype-governance"
}

// Download downloads or symlinks the governance repository
func Download() error {
	govPath := getGovernancePath()
	localPath := getLocalGovPath()

	os.RemoveAll(govPath)

	if err := os.MkdirAll(filepath.Dir(govPath), 0755); err != nil {
		return fmt.Errorf("failed to create cache dir: %w", err)
	}

	if err := os.Symlink(localPath, govPath); err != nil {
		return copyDir(localPath, govPath)
	}

	fmt.Printf("Governance cached at: %s\n", govPath)
	return nil
}

func copyDir(src, dst string) error {
	return os.MkdirAll(dst, 0755)
}

// Install installs standards to a repository
func Install(repoPath, pillar string, force, dryRun bool) error {
	absPath, err := filepath.Abs(repoPath)
	if err != nil {
		return fmt.Errorf("failed to resolve path: %w", err)
	}

	govPath := getGovernancePath()
	if _, err := os.Stat(govPath); os.IsNotExist(err) {
		fmt.Println("Governance not cached. Downloading...")
		if err := Download(); err != nil {
			return fmt.Errorf("failed to download governance: %w", err)
		}
	}

	mappings := getFileMappings(pillar)

	if dryRun {
		fmt.Println("Dry-run: Would install:")
		for _, m := range mappings {
			fmt.Printf("  + %s\n", m.Dest)
		}
		return nil
	}

	installed := 0
	skipped := 0

	for _, m := range mappings {
		src := filepath.Join(govPath, m.Src)
		dst := filepath.Join(absPath, m.Dest)

		if _, err := os.Stat(src); os.IsNotExist(err) {
			continue
		}

		if !force {
			if _, err := os.Stat(dst); err == nil {
				skipped++
				continue
			}
		}

		if err := os.MkdirAll(filepath.Dir(dst), 0755); err != nil {
			return fmt.Errorf("mkdir %s: %w", filepath.Dir(dst), err)
		}

		if err := copyFile(src, dst); err != nil {
			return fmt.Errorf("copy %s: %w", m.Dest, err)
		}

		installed++
		fmt.Printf("+ %s\n", m.Dest)
	}

	// Save manifest
	manifest := loadOrCreateManifest(absPath)
	manifest.Pillars = appendIfMissing(manifest.Pillars, pillar)
	manifest.UpdatedAt = time.Now()
	saveManifest(absPath, manifest)

	fmt.Printf("\nInstalled %d files (%d skipped)\n", installed, skipped)
	return nil
}

// Uninstall removes standards from a repository
func Uninstall(repoPath string) error {
	absPath, err := filepath.Abs(repoPath)
	if err != nil {
		return fmt.Errorf("failed to resolve path: %w", err)
	}

	manifestPath := filepath.Join(absPath, ".standards", "manifest.json")
	if _, err := os.Stat(manifestPath); os.IsNotExist(err) {
		fmt.Println("No standards manifest found.")
		return nil
	}

	manifest := loadManifest(absPath)
	removed := 0

	for _, f := range manifest.Files {
		path := filepath.Join(absPath, f.Path)
		if _, err := os.Stat(path); err == nil {
			os.Remove(path)
			removed++
			fmt.Printf("- %s\n", f.Path)
		}
	}

	os.RemoveAll(filepath.Join(absPath, ".standards"))
	fmt.Printf("\nRemoved %d files\n", removed)
	return nil
}

// Update updates standards to latest
func Update(repoPath string) error {
	fmt.Printf("Updating standards for: %s\n", repoPath)
	if err := Download(); err != nil {
		return err
	}
	return Install(repoPath, "all", false, false)
}

// Check validates compliance
func Check(repoPath string) error {
	absPath, err := filepath.Abs(repoPath)
	if err != nil {
		return fmt.Errorf("failed to resolve path: %w", err)
	}

	fmt.Printf("Checking standards for: %s\n\n", absPath)

	manifestPath := filepath.Join(absPath, ".standards", "manifest.json")
	if _, err := os.Stat(manifestPath); os.IsNotExist(err) {
		fmt.Println("Status: not installed")
		return nil
	}

	manifest := loadManifest(absPath)
	missing := 0

	for _, f := range manifest.Files {
		path := filepath.Join(absPath, f.Path)
		if _, err := os.Stat(path); os.IsNotExist(err) {
			fmt.Printf("Missing: %s\n", f.Path)
			missing++
		}
	}

	if missing == 0 {
		fmt.Println("Status: compliant")
	} else {
		fmt.Printf("\nStatus: non-compliant (%d missing)\n", missing)
	}

	return nil
}

// Status shows current status
func Status(repoPath string) error {
	absPath, err := filepath.Abs(repoPath)
	if err != nil {
		return fmt.Errorf("failed to resolve path: %w", err)
	}

	manifestPath := filepath.Join(absPath, ".standards", "manifest.json")
	if _, err := os.Stat(manifestPath); os.IsNotExist(err) {
		fmt.Println("Standards not installed.")
		fmt.Println("Run 'pheno standards install' to install.")
		return nil
	}

	manifest := loadManifest(absPath)
	fmt.Printf("Version: %s\n", manifest.Version)
	fmt.Printf("Installed: %s\n", manifest.InstalledAt.Format(time.RFC3339))
	fmt.Printf("Updated: %s\n", manifest.UpdatedAt.Format(time.RFC3339))
	fmt.Printf("Pillars: %s\n", strings.Join(manifest.Pillars, ", "))
	fmt.Printf("Files: %d managed\n", len(manifest.Files))

	modified := 0
	for _, f := range manifest.Files {
		if f.Modified {
			modified++
		}
	}
	if modified > 0 {
		fmt.Printf("Modified: %d user-modified\n", modified)
	}

	return nil
}

// Diff shows pending changes
func Diff(repoPath string) error {
	absPath, err := filepath.Abs(repoPath)
	if err != nil {
		return fmt.Errorf("failed to resolve path: %w", err)
	}

	fmt.Printf("Pending changes for: %s\n\n", absPath)

	govPath := getGovernancePath()
	mappings := getFileMappings("all")

	for _, m := range mappings {
		src := filepath.Join(govPath, m.Src)
		dst := filepath.Join(absPath, m.Dest)

		if _, err := os.Stat(src); os.IsNotExist(err) {
			continue
		}

		srcHash, _ := fileHash(src)
		dstHash, _ := fileHash(dst)

		if srcHash != dstHash {
			if _, err := os.Stat(dst); os.IsNotExist(err) {
				fmt.Printf("+ New: %s\n", m.Dest)
			} else {
				fmt.Printf("~ Modified: %s\n", m.Dest)
			}
		}
	}

	return nil
}

type fileMapping struct {
	Src  string
	Dest string
}

func getFileMappings(pillar string) []fileMapping {
	all := []fileMapping{
		{Src: "templates/github/CODEOWNERS/CODEOWNERS", Dest: ".github/CODEOWNERS"},
		{Src: "templates/github/AGENTS.md.template", Dest: "AGENTS.md"},
		{Src: "templates/github/ISSUE_TEMPLATE/bug_report.md", Dest: ".github/ISSUE_TEMPLATE/bug_report.md"},
		{Src: "templates/github/ISSUE_TEMPLATE/feature_request.md", Dest: ".github/ISSUE_TEMPLATE/feature_request.md"},
		{Src: "templates/github/ISSUE_TEMPLATE/task.md", Dest: ".github/ISSUE_TEMPLATE/task.md"},
		{Src: "templates/github/PULL_REQUEST_TEMPLATE.md", Dest: ".github/PULL_REQUEST_TEMPLATE.md"},
		{Src: "templates/ci/ci.yml.template", Dest: ".github/workflows/ci.yml"},
		{Src: "templates/ci/ci-rust.yml.template", Dest: ".github/workflows/ci-rust.yml"},
		{Src: "templates/ci/ci-python.yml.template", Dest: ".github/workflows/ci-python.yml"},
		{Src: "templates/ci/ci-go.yml.template", Dest: ".github/workflows/ci-go.yml"},
		{Src: "templates/ci/ci-typescript.yml.template", Dest: ".github/workflows/ci-typescript.yml"},
		{Src: "configs/rust/clippy.toml", Dest: "clippy.toml"},
		{Src: "configs/rust/rustfmt.toml", Dest: "rustfmt.toml"},
		{Src: "configs/python/ruff.toml", Dest: "ruff.toml"},
		{Src: "configs/python/mypy.ini", Dest: "mypy.ini"},
		{Src: "configs/go/golangci.yml", Dest: ".golangci.yml"},
		{Src: "configs/typescript/eslintrc.json", Dest: ".eslintrc.json"},
		{Src: "configs/typescript/prettierrc.json", Dest: ".prettierrc.json"},
		{Src: "configs/universal/renovate.json", Dest: "renovate.json"},
		{Src: "configs/universal/pre-commit.yaml", Dest: ".pre-commit-config.yaml"},
		{Src: ".editorconfig", Dest: ".editorconfig"},
	}

	if pillar == "all" || pillar == "" {
		return all
	}

	// Filter by pillar (simplified)
	var filtered []fileMapping
	for _, m := range all {
		if pillar == "codeowners" && m.Dest == ".github/CODEOWNERS" {
			filtered = append(filtered, m)
		} else if pillar == "ci" && strings.Contains(m.Dest, ".github/workflows/") {
			filtered = append(filtered, m)
		} else if pillar == "linters" && (strings.HasSuffix(m.Dest, ".toml") || strings.HasSuffix(m.Dest, ".yml") || strings.HasSuffix(m.Dest, ".json")) {
			filtered = append(filtered, m)
		} else if pillar == "templates" && strings.Contains(m.Dest, "ISSUE_TEMPLATE") {
			filtered = append(filtered, m)
		} else if pillar == "agents" && m.Dest == "AGENTS.md" {
			filtered = append(filtered, m)
		}
	}
	return filtered
}

func loadManifest(repoPath string) Manifest {
	path := filepath.Join(repoPath, ".standards", "manifest.json")
	data, err := os.ReadFile(path)
	if err != nil {
		return Manifest{}
	}
	var m Manifest
	json.Unmarshal(data, &m)
	return m
}

func loadOrCreateManifest(repoPath string) Manifest {
	m := loadManifest(repoPath)
	if m.Version == "" {
		m = Manifest{
			Version:       "main",
			InstalledAt:   time.Now(),
			UpdatedAt:     time.Now(),
			Pillars:       []string{},
			Files:         []File{},
			GovernanceURL: "https://github.com/kooshapari/phenotype-governance",
		}
	}
	return m
}

func saveManifest(repoPath string, m Manifest) error {
	dir := filepath.Join(repoPath, ".standards")
	os.MkdirAll(dir, 0755)

	path := filepath.Join(dir, "manifest.json")
	data, err := json.MarshalIndent(m, "", "  ")
	if err != nil {
		return err
	}
	return os.WriteFile(path, data, 0644)
}

func copyFile(src, dst string) error {
	data, err := os.ReadFile(src)
	if err != nil {
		return err
	}
	return os.WriteFile(dst, data, 0644)
}

func fileHash(path string) (string, error) {
	data, err := os.ReadFile(path)
	if err != nil {
		return "", err
	}
	hash := sha256.Sum256(data)
	return hex.EncodeToString(hash[:]), nil
}

func appendIfMissing(slice []string, item string) []string {
	for _, s := range slice {
		if s == item {
			return slice
		}
	}
	return append(slice, item)
}
