package cmd

import (
	"fmt"

	"github.com/KooshaPari/pheno-cli/internal/standards"
	"github.com/spf13/cobra"
)

var (
	installPillar  string
	installForce   bool
	installDryRun  bool
)

var standardsCmd = &cobra.Command{
	Use:   "standards",
	Short: "Manage repository governance standards",
	Long: `Manage repository governance standards from phenotype-governance.

Install, update, check, or remove standards from repositories.
Standards include linter configs, CI/CD templates, CODEOWNERS, and more.`,
}

var installCmd = &cobra.Command{
	Use:   "install [repo]",
	Short: "Install standards to a repository",
	Long:  `Install governance standards to a repository. If no repo is specified, uses the current directory.`,
	Args:  cobra.RangeArgs(0, 1),
	RunE: func(cmd *cobra.Command, args []string) error {
		repo := "."
		if len(args) > 0 {
			repo = args[0]
		}
		return standards.Install(repo, installPillar, installForce, installDryRun)
	},
}

var uninstallCmd = &cobra.Command{
	Use:   "uninstall [repo]",
	Short: "Uninstall standards from a repository",
	Long:  `Uninstall governance standards from a repository.`,
	Args:  cobra.RangeArgs(0, 1),
	RunE: func(cmd *cobra.Command, args []string) error {
		repo := "."
		if len(args) > 0 {
			repo = args[0]
		}
		return standards.Uninstall(repo)
	},
}

var updateCmd = &cobra.Command{
	Use:   "update [repo]",
	Short: "Update standards to latest governance",
	Long:  `Update installed standards to the latest from phenotype-governance.`,
	Args:  cobra.RangeArgs(0, 1),
	RunE: func(cmd *cobra.Command, args []string) error {
		repo := "."
		if len(args) > 0 {
			repo = args[0]
		}
		return standards.Update(repo)
	},
}

var checkCmd = &cobra.Command{
	Use:   "check [repo]",
	Short: "Check standards compliance",
	Long:  `Check if a repository complies with installed standards.`,
	Args:  cobra.RangeArgs(0, 1),
	RunE: func(cmd *cobra.Command, args []string) error {
		repo := "."
		if len(args) > 0 {
			repo = args[0]
		}
		return standards.Check(repo)
	},
}

var statusCmd = &cobra.Command{
	Use:   "status [repo]",
	Short: "Show standards status",
	Long:  `Show the current status of installed standards.`,
	Args:  cobra.RangeArgs(0, 1),
	RunE: func(cmd *cobra.Command, args []string) error {
		repo := "."
		if len(args) > 0 {
			repo = args[0]
		}
		return standards.Status(repo)
	},
}

var diffCmd = &cobra.Command{
	Use:   "diff [repo]",
	Short: "Show pending changes from governance",
	Long:  `Show what would change if standards were updated.`,
	Args:  cobra.RangeArgs(0, 1),
	RunE: func(cmd *cobra.Command, args []string) error {
		repo := "."
		if len(args) > 0 {
			repo = args[0]
		}
		return standards.Diff(repo)
	},
}

var downloadCmd = &cobra.Command{
	Use:   "download",
	Short: "Download governance repository",
	Long:  `Download the phenotype-governance repository to cache.`,
	RunE: func(cmd *cobra.Command, args []string) error {
		return standards.Download()
	},
}

func init() {
	rootCmd.AddCommand(standardsCmd)
	standardsCmd.AddCommand(installCmd)
	standardsCmd.AddCommand(uninstallCmd)
	standardsCmd.AddCommand(updateCmd)
	standardsCmd.AddCommand(checkCmd)
	standardsCmd.AddCommand(statusCmd)
	standardsCmd.AddCommand(diffCmd)
	standardsCmd.AddCommand(downloadCmd)

	installCmd.Flags().StringVar(&installPillar, "pillar", "all", "Pillar to install (linters, ci, codeowners, agents, devcontainer, templates, all)")
	installCmd.Flags().BoolVar(&installForce, "force", false, "Overwrite existing files")
	installCmd.Flags().BoolVar(&installDryRun, "dry-run", false, "Show what would be installed without writing")
}
