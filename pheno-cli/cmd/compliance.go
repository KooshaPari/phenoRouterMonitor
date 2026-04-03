package cmd

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"strings"

	"github.com/KooshaPari/pheno-cli/internal/policy"
	"github.com/spf13/cobra"
)

var (
	complianceReposDir    string
	compliancePolicyFile  string
	complianceOutputJSON  bool
	complianceFailOnDrift bool
)

var complianceCmd = &cobra.Command{
	Use:   "compliance",
	Short: "Check organization-wide standards compliance",
	Long: `Compliance reports organization-wide standards compliance across repositories.

It scans repositories and checks for:
- Required governance files (mise.toml, cliff.toml, etc.)
- Required hooks (pre-commit, pre-push)
- Required CI/CD workflows
- Policy drift from organization standards

Examples:
  # Check compliance in current directory
  pheno compliance

  # Check compliance across all repos
  pheno compliance --repos-dir ~/projects

  # Output as JSON for CI integration
  pheno compliance --json

  # Fail if any drift is detected (for CI gates)
  pheno compliance --fail-on-drift`,
	RunE: runCompliance,
}

func init() {
	complianceCmd.Flags().StringVar(&complianceReposDir, "repos-dir", ".", "Directory containing repositories to check")
	complianceCmd.Flags().StringVar(&compliancePolicyFile, "policy-file", "", "Path to organization policy file (TOML)")
	complianceCmd.Flags().BoolVar(&complianceOutputJSON, "json", false, "Output results as JSON")
	complianceCmd.Flags().BoolVar(&complianceFailOnDrift, "fail-on-drift", false, "Exit with error code if drift is detected")
}

func runCompliance(cmd *cobra.Command, args []string) error {
	// Load policy configuration
	var cfg *policy.OrgConfig
	var err error

	if compliancePolicyFile != "" {
		cfg, err = policy.LoadConfig(compliancePolicyFile)
		if err != nil {
			return fmt.Errorf("failed to load policy file: %w", err)
		}
	} else {
		// Use default configuration
		cfg = policy.DefaultConfig()
	}

	// Detect drift
	detector := &policy.DriftDetector{
		Policy: cfg,
	}

	report, err := detector.Detect(complianceReposDir)
	if err != nil {
		return fmt.Errorf("drift detection failed: %w", err)
	}

	// Output results
	if complianceOutputJSON {
		return outputComplianceJSON(report)
	}

	return outputComplianceText(report)
}

func outputComplianceJSON(report *policy.DriftReport) error {
	data, err := json.MarshalIndent(report, "", "  ")
	if err != nil {
		return fmt.Errorf("failed to marshal report: %w", err)
	}
	fmt.Println(string(data))
	return nil
}

func outputComplianceText(report *policy.DriftReport) error {
	fmt.Printf("\n📊 Compliance Report\n")
	fmt.Printf("====================\n\n")
	fmt.Printf("Total Repositories Scanned: %d\n", report.Summary.Total)
	fmt.Printf("Info Level: %d\n", report.Summary.Info)
	fmt.Printf("Warning Level: %d\n", report.Summary.Warning)
	fmt.Printf("Error Level: %d\n", report.Summary.Error)
	fmt.Printf("Critical Level: %d\n", report.Summary.Critical)
	fmt.Printf("\nDuration: %s\n\n", report.Duration)

	if len(report.Items) == 0 {
		fmt.Println("✅ All repositories are compliant!")
		return nil
	}

	fmt.Println("⚠️  Drift Findings:")
	fmt.Println()

	for _, item := range report.Items {
		icon := "🔴"
		if item.Severity == policy.SeverityMedium {
			icon = "🟡"
		} else if item.Severity == policy.SeverityLow {
			icon = "🟢"
		}

		fmt.Printf("%s %s\n", icon, item.RepoPath)
		if item.FilePath != "" {
			fmt.Printf("   File: %s\n", item.FilePath)
		}
		fmt.Printf("   Issue: %s\n", item.Message)
		fmt.Printf("   Severity: %s\n\n", severityToString(item.Severity))
	}

	if complianceFailOnDrift && len(report.Items) > 0 {
		return fmt.Errorf("drift detected in %d repositories", len(report.Items))
	}

	return nil
}

func severityToString(s policy.Severity) string {
	switch s {
	case policy.SeverityLow:
		return "low"
	case policy.SeverityMedium:
		return "medium"
	case policy.SeverityHigh:
		return "high"
	default:
		return "unknown"
	}
}
