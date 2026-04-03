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
	fmt.Printf("Total Repositories: %d\n", report.TotalRepos)
	fmt.Printf("Compliant: %d\n", report.CompliantRepos)
	fmt.Printf("Drift Detected: %d\n", report.DriftDetected)
	fmt.Printf("Failed: %d\n", report.FailedRepos)
	fmt.Printf("\nDuration: %dms\n\n", report.DurationMs)

	if len(report.Findings) == 0 {
		fmt.Println("✅ All repositories are compliant!")
		return nil
	}

	fmt.Println("⚠️  Drift Findings:")
	fmt.Println()

	for _, finding := range report.Findings {
		icon := "🔴"
		if finding.Severity == policy.SeverityMedium {
			icon = "🟡"
		} else if finding.Severity == policy.SeverityLow {
			icon = "🟢"
		}

		fmt.Printf("%s %s\n", icon, finding.Repo)
		fmt.Printf("   File: %s\n", finding.File)
		fmt.Printf("   Issue: %s\n", finding.Issue)
		fmt.Printf("   Severity: %s\n\n", finding.Severity)
	}

	if complianceFailOnDrift && report.DriftDetected > 0 {
		return fmt.Errorf("drift detected in %d repositories", report.DriftDetected)
	}

	return nil
}
