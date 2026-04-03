package cmd

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"

	"context"

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
	Long:  `Compliance reports organization-wide standards compliance across repositories.`,
	RunE:  runCompliance,
}

func init() {
	complianceCmd.Flags().StringVar(&complianceReposDir, "repos-dir", ".", "Directory containing repositories to check")
	complianceCmd.Flags().StringVar(&compliancePolicyFile, "policy-file", "", "Path to organization policy file (TOML)")
	complianceCmd.Flags().BoolVar(&complianceOutputJSON, "json", false, "Output results as JSON")
	complianceCmd.Flags().BoolVar(&complianceFailOnDrift, "fail-on-drift", false, "Exit with error code if drift is detected")
}

func runCompliance(cmd *cobra.Command, args []string) error {
	var cfg *policy.OrgConfig
	var err error

	if compliancePolicyFile != "" {
		cfg, err = policy.LoadConfig(compliancePolicyFile)
		if err != nil {
			return fmt.Errorf("failed to load policy file: %w", err)
		}
	} else {
		cfg = policy.DefaultConfig()
	}

	ctx := context.Background()
	report, err := policy.DetectDrift(ctx, complianceReposDir, cfg)
	if err != nil {
		return fmt.Errorf("drift detection failed: %w", err)
	}

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
	fmt.Printf("Total: %d\n", report.Summary.Total)
	fmt.Printf("Info: %d\n", report.Summary.Info)
	fmt.Printf("Warning: %d\n", report.Summary.Warning)
	fmt.Printf("Error: %d\n", report.Summary.Error)
	fmt.Printf("Critical: %d\n\n", report.Summary.Critical)

	if len(report.Items) == 0 {
		fmt.Println("✅ All repositories are compliant!")
		if complianceFailOnDrift {
			return fmt.Errorf("drift detected")
		}
		return nil
	}

	fmt.Println("⚠️  Drift Findings:")
	fmt.Println()

	for _, item := range report.Items {
		icon := "🔴"
		if item.Severity == policy.SeverityWarning {
			icon = "🟡"
		} else if item.Severity == policy.SeverityInfo {
			icon = "🟢"
		}

		repoName := filepath.Dir(item.Path)
		if repoName == "." {
			repoName = item.Path
		}

		fmt.Printf("%s %s\n", icon, repoName)
		if item.Path != "" {
			fmt.Printf("   File: %s\n", filepath.Base(item.Path))
		}
		fmt.Printf("   Issue: %s\n", item.Description)
		fmt.Printf("   Severity: %s\n\n", severityToString(item.Severity))
	}

	if complianceFailOnDrift {
		os.Exit(1)
	}

	return nil
}

func severityToString(s policy.Severity) string {
	switch s {
	case policy.SeverityInfo:
		return "info"
	case policy.SeverityWarning:
		return "warning"
	case policy.SeverityError:
		return "error"
	case policy.SeverityCritical:
		return "critical"
	default:
		return "unknown"
	}
}
