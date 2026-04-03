package compliance

import (
	"encoding/json"
	"fmt"
	"os/exec"
	"path/filepath"
)

type Finding struct {
	RuleID   string `json:"rule_id"`
	FilePath string `json:"file_path"`
	Message  string `json:"message"`
}

type Bridge struct {
	GuardPath string
}

func NewBridge(repoRoot string) *Bridge {
	return &Bridge{
		GuardPath: filepath.Join(repoRoot, "phenotype-infrakit", "target", "debug", "pheno-guard"),
	}
}

func (b *Bridge) Scan(reposDir string) ([]Finding, error) {
	cmd := exec.Command(b.GuardPath, "scan", "--repos-dir", reposDir, "--format", "json")
	output, err := cmd.CombinedOutput()
	if err != nil {
		return nil, fmt.Errorf("failed to run pheno-guard: %w, output: %s", err, string(output))
	}

	var findings []Finding
	if err := json.Unmarshal(output, &findings); err != nil {
		return nil, fmt.Errorf("failed to parse pheno-guard output: %w", err)
	}

	return findings, nil
}
