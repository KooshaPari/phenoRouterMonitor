package policy

import (
	"context"
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"strings"
	"time"
)

// Severity represents the severity of a policy violation.
type Severity int

const (
	SeverityInfo Severity = iota
	SeverityWarning
	SeverityError
	SeverityCritical
)

func (s Severity) String() string {
	switch s {
	case SeverityInfo:
		return "info"
	case SeverityWarning:
		return "warning"
	case SeverityError:
		return "error"
	case SeverityCritical:
		return "critical"
	default:
		return "unknown"
	}
}

// DriftType represents the type of policy drift.
type DriftType int

const (
	DriftMissingFile DriftType = iota
	DriftMissingHook
	DriftMissingCIWorkflow
	DriftMissingCICheck
	DriftConfigMismatch
	DriftGateFailure
)

func (d DriftType) String() string {
	switch d {
	case DriftMissingFile:
		return "missing_file"
	case DriftMissingHook:
		return "missing_hook"
	case DriftMissingCIWorkflow:
		return "missing_ci_workflow"
	case DriftMissingCICheck:
		return "missing_ci_check"
	case DriftConfigMismatch:
		return "config_mismatch"
	case DriftGateFailure:
		return "gate_failure"
	default:
		return "unknown"
	}
}

// DriftItem represents a single policy drift item.
type DriftItem struct {
	Type        DriftType `json:"type"`
	Severity    Severity  `json:"severity"`
	Path        string    `json:"path"`
	Expected    string    `json:"expected,omitempty"`
	Actual      string    `json:"actual,omitempty"`
	Description string    `json:"description"`
}

// DriftReport contains all detected policy drifts.
type DriftReport struct {
	Timestamp      time.Time            `json:"timestamp"`
	RepoPath       string               `json:"repo_path"`
	RepoName       string               `json:"repo_name"`
	Items          []DriftItem          `json:"items"`
	ItemsBySeverity map[Severity][]DriftItem `json:"items_by_severity"`
	Summary        Summary              `json:"summary"`
}

// Summary provides a count of drifts by severity.
type Summary struct {
	Total      int `json:"total"`
	Info       int `json:"info"`
	Warning    int `json:"warning"`
	Error      int `json:"error"`
	Critical   int `json:"critical"`
}

// NewDriftReport creates a new drift report.
func NewDriftReport(repoPath string) *DriftReport {
	return &DriftReport{
		Timestamp:       time.Now(),
		RepoPath:        repoPath,
		RepoName:        filepath.Base(repoPath),
		Items:           []DriftItem{},
		ItemsBySeverity: make(map[Severity][]DriftItem),
		Summary:         Summary{},
	}
}

// AddItem adds a drift item to the report.
func (r *DriftReport) AddItem(item DriftItem) {
	r.Items = append(r.Items, item)
	r.updateSummary(item.Severity)
}

func (r *DriftReport) updateSummary(severity Severity) {
	r.Summary.Total++
	switch severity {
	case SeverityInfo:
		r.Summary.Info++
	case SeverityWarning:
		r.Summary.Warning++
	case SeverityError:
		r.Summary.Error++
	case SeverityCritical:
		r.Summary.Critical++
	}
}

// Report generates a JSON report of the drift.
func (r *DriftReport) Report() (string, error) {
	data, err := json.MarshalIndent(r, "", "  ")
	if err != nil {
		return "", fmt.Errorf("failed to marshal report: %w", err)
	}
	return string(data), nil
}

// DetectDrift detects policy drift in a repository.
func DetectDrift(ctx context.Context, repoPath string, config *OrgConfig) (*DriftReport, error) {
	report := NewDriftReport(repoPath)

	// Check for required files
	for _, file := range config.RequiredStandards.Files {
		if err := checkFile(report, repoPath, file); err != nil {
			// Continue checking other files
		}
	}

	// Check for required hooks
	for _, hook := range config.RequiredStandards.Hooks {
		if err := checkHook(report, repoPath, hook); err != nil {
			// Continue checking other hooks
		}
	}

	// Check for required CI workflows
	for _, workflow := range config.RequiredStandards.CI.RequiredWorkflows {
		if err := checkCIWorkflow(report, repoPath, workflow); err != nil {
			// Continue checking other workflows
		}
	}

	return report, nil
}

func checkFile(report *DriftReport, repoPath, file string) error {
	path := filepath.Join(repoPath, file)
	if _, err := os.Stat(path); err != nil {
		if os.IsNotExist(err) {
			report.AddItem(DriftItem{
				Type:        DriftMissingFile,
				Severity:    SeverityError,
				Path:        path,
				Description: fmt.Sprintf("Required file missing: %s", file),
			})
			return nil
		}
		return err
	}
	return nil
}

func checkHook(report *DriftReport, repoPath, hook string) error {
	hookPath := filepath.Join(repoPath, ".git", "hooks", hook)
	if _, err := os.Stat(hookPath); err != nil {
		if os.IsNotExist(err) {
			report.AddItem(DriftItem{
				Type:        DriftMissingHook,
				Severity:    SeverityWarning,
				Path:        hookPath,
				Description: fmt.Sprintf("Git hook missing: %s", hook),
			})
			return nil
		}
		return err
	}
	return nil
}

func checkCIWorkflow(report *DriftReport, repoPath, workflow string) error {
	workflowPath := filepath.Join(repoPath, ".github", "workflows", workflow)
	if _, err := os.Stat(workflowPath); err != nil {
		if os.IsNotExist(err) {
			report.AddItem(DriftItem{
				Type:        DriftMissingCIWorkflow,
				Severity:    SeverityError,
				Path:        workflowPath,
				Description: fmt.Sprintf("CI workflow missing: %s", workflow),
			})
			return nil
		}
		return err
	}
	return nil
}

// ReportSummary returns a human-readable summary of the drift report.
func ReportSummary(report *DriftReport) string {
	var sb strings.Builder

	sb.WriteString(fmt.Sprintf("Policy Drift Report for %s\n", report.RepoName))
	sb.WriteString(fmt.Sprintf("Generated: %s\n", report.Timestamp.Format(time.RFC3339)))
	sb.WriteString(strings.Repeat("-", 50))
	sb.WriteString("\n")

	sb.WriteString(fmt.Sprintf("Total Issues: %d\n", report.Summary.Total))
	sb.WriteString(fmt.Sprintf("  Critical: %d\n", report.Summary.Critical))
	sb.WriteString(fmt.Sprintf("  Error: %d\n", report.Summary.Error))
	sb.WriteString(fmt.Sprintf("  Warning: %d\n", report.Summary.Warning))
	sb.WriteString(fmt.Sprintf("  Info: %d\n", report.Summary.Info))
	sb.WriteString("\n")

	if len(report.Items) > 0 {
		sb.WriteString("Details:\n")
		for _, item := range report.Items {
			sb.WriteString(fmt.Sprintf("  [%s] %s: %s\n", item.Severity, item.Type, item.Description))
		}
	} else {
		sb.WriteString("✓ No policy drift detected\n")
	}

	return sb.String()
}

// SeverityOrder returns the severity order (higher = more severe).
func SeverityOrder(s Severity) int {
	return int(s)
}

// DetermineSeverity determines severity based on drift type.
func DetermineSeverity(dt DriftType) Severity {
	switch dt {
	case DriftMissingFile:
		return SeverityError
	case DriftMissingHook:
		return SeverityWarning
	case DriftMissingCIWorkflow:
		return SeverityError
	case DriftMissingCICheck:
		return SeverityWarning
	case DriftConfigMismatch:
		return SeverityWarning
	case DriftGateFailure:
		return SeverityCritical
	default:
		return SeverityInfo
	}
}
