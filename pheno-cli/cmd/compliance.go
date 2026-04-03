package cmd

import (
	"fmt"
	"os"

	"github.com/spf13/cobra"
	"github.com/KooshaPari/pheno-cli/internal/compliance"
)

var (
	complianceReposDir string
)

func init() {
	complianceCmd := &cobra.Command{
		Use:   "compliance",
		Short: "Run compliance scan using pheno-guard (Rust core)",
		RunE:  runCompliance,
	}
	complianceCmd.Flags().StringVar(&complianceReposDir, "repos-dir", ".", "Directory to scan")
	rootCmd.AddCommand(complianceCmd)
}

func runCompliance(cmd *cobra.Command, args []string) error {
	wd, err := os.Getwd()
	if err != nil {
		return err
	}
	
	bridge := compliance.NewBridge(wd)
	
	fmt.Printf("Running compliance scan in %s...\n", complianceReposDir)
	findings, err := bridge.Scan(complianceReposDir)
	if err != nil {
		return fmt.Errorf("compliance scan failed: %w", err)
	}
	
	if len(findings) == 0 {
		fmt.Println("✅ Compliance scan passed!")
	} else {
		fmt.Printf("❌ Found %d compliance findings:\n", len(findings))
		for _, f := range findings {
			fmt.Printf("  [%s] %s: %s\n", f.RuleID, f.FilePath, f.Message)
		}
		os.Exit(1)
	}
	
	return nil
}
