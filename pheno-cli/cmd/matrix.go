package cmd

import (
	"fmt"
	"os"

	"github.com/spf13/cobra"

	"github.com/KooshaPari/pheno-cli/internal/adapters"
	"github.com/KooshaPari/pheno-cli/internal/detect"
	"github.com/KooshaPari/pheno-cli/internal/discover"
	"github.com/KooshaPari/pheno-cli/internal/matrix"
)

var matrixCmd = &cobra.Command{
	Use:   "matrix",
	Short: "Generate release matrix",
	RunE:  runMatrix,
}

func init() {
	matrixCmd.Flags().String("repos-dir", ".", "root directory to discover repositories")
	matrixCmd.Flags().String("output", "", "output file path (default: stdout)")
}

func runMatrix(cmd *cobra.Command, args []string) error {
	reposDir, _ := cmd.Flags().GetString("repos-dir")
	outputPath, _ := cmd.Flags().GetString("output")

	repos, err := discover.FindRepositories(reposDir)
	if err != nil {
		return fmt.Errorf("discovering repositories: %w", err)
	}

	var packages []adapters.Package
	for _, repo := range repos {
		detected := detect.DetectLanguages(repo.Path)
		for _, d := range detected {
			packages = append(packages, adapters.Package{
				Name:         repo.Name,
				Language:     d.Language,
				Registry:     d.Registry,
				ManifestPath: d.ManifestPath,
			})
		}
	}

	rows := matrix.GenerateMatrix(packages)
	md := matrix.FormatMarkdown(rows)

	if outputPath != "" {
		if err := os.WriteFile(outputPath, []byte(md), 0644); err != nil {
			return fmt.Errorf("writing output file: %w", err)
		}
		fmt.Printf("matrix written to %s\n", outputPath)
		return nil
	}

	fmt.Print(md)
	return nil
}
