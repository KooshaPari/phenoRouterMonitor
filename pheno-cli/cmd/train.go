package cmd

import (
	"encoding/json"
	"fmt"
	"os"
	"strings"

	"github.com/KooshaPari/pheno-cli/internal/state"
	"github.com/spf13/cobra"
)

var (
	trainName     string
	trainRepos    string
	trainDeps     string
	trainFrom     string
	trainTo       string
	trainStateDir string
)

var trainCmd = &cobra.Command{
	Use:   "train",
	Short: "Manage cross-repo release trains",
	Long: `Train manages cross-repository release trains for coordinated releases.

A release train is a group of repositories that are promoted together through
release channels (alpha -> canary -> beta -> rc -> prod).

Examples:
  # List all release trains
  pheno train list

  # Create a new release train
  pheno train create api-train --repos api-gateway,auth-service,user-service

  # Check train status
  pheno train status api-train

  # Promote a train to the next channel
  pheno train promote api-train --to beta`,
}

var trainListCmd = &cobra.Command{
	Use:   "list",
	Short: "List all release trains",
	RunE: func(cmd *cobra.Command, args []string) error {
		mgr := state.NewTrainManager(trainStateDir)
		trains, err := mgr.ListTrains()
		if err != nil {
			return fmt.Errorf("failed to list trains: %w", err)
		}

		if len(trains) == 0 {
			fmt.Println("No release trains found.")
			fmt.Println("Create one with: pheno train create <name> --repos <repo1,repo2>")
			return nil
		}

		fmt.Println("Release Trains:")
		fmt.Println(strings.Repeat("-", 60))
		for _, train := range trains {
			status, _ := mgr.GetTrainStatus(train.Name)
			fmt.Printf("  %-20s %s\n", train.Name, status)
		}
		return nil
	},
}

var trainCreateCmd = &cobra.Command{
	Use:   "create [name]",
	Short: "Create a new release train",
	Args:  cobra.ExactArgs(1),
	RunE: func(cmd *cobra.Command, args []string) error {
		name := args[0]
		mgr := state.NewTrainManager(trainStateDir)

		repos := parseRepoList(trainRepos)
		if len(repos) == 0 {
			return fmt.Errorf("no repositories specified, use --repos flag")
		}

		train, err := mgr.CreateTrain(name, repos, nil)
		if err != nil {
			return fmt.Errorf("failed to create train: %w", err)
		}

		fmt.Printf("✓ Created release train: %s\n", train.Name)
		fmt.Printf("  Repositories: %d\n", len(train.Repos))
		for _, repo := range train.Repos {
			fmt.Printf("    - %s (%s)\n", repo.Name, repo.Channel)
		}
		return nil
	},
}

var trainStatusCmd = &cobra.Command{
	Use:   "status [name]",
	Short: "Show release train status",
	Args:  cobra.ExactArgs(1),
	RunE: func(cmd *cobra.Command, args []string) error {
		name := args[0]
		mgr := state.NewTrainManager(trainStateDir)

		train, err := mgr.GetTrain(name)
		if err != nil {
			return fmt.Errorf("train not found: %w", err)
		}

		status, _ := mgr.GetTrainStatus(name)
		fmt.Printf("Release Train: %s\n", train.Name)
		fmt.Printf("Status: %s\n", status)
		fmt.Printf("Created: %s\n", train.CreatedAt.Format("2006-01-02 15:04"))
		fmt.Printf("Updated: %s\n", train.UpdatedAt.Format("2006-01-02 15:04"))
		fmt.Println("\nRepositories:")
		for _, repo := range train.Repos {
			fmt.Printf("  %-20s %s\n", repo.Name, repo.Channel)
		}
		return nil
	},
}

var trainPromoteCmd = &cobra.Command{
	Use:   "promote [name]",
	Short: "Promote a release train",
	Args:  cobra.ExactArgs(1),
	RunE: func(cmd *cobra.Command, args []string) error {
		name := args[0]
		mgr := state.NewTrainManager(trainStateDir)

		if err := mgr.PromoteTrain(name, trainFrom, trainTo); err != nil {
			return fmt.Errorf("promotion failed: %w", err)
		}

		fmt.Printf("✓ Promoted train '%s' from %s to %s\n", name, trainFrom, trainTo)
		return nil
	},
}

var trainDeleteCmd = &cobra.Command{
	Use:   "delete [name]",
	Short: "Delete a release train",
	Args:  cobra.ExactArgs(1),
	RunE: func(cmd *cobra.Command, args []string) error {
		name := args[0]
		mgr := state.NewTrainManager(trainStateDir)

		if err := mgr.DeleteTrain(name); err != nil {
			return fmt.Errorf("failed to delete train: %w", err)
		}

		fmt.Printf("✓ Deleted release train: %s\n", name)
		return nil
	},
}

func init() {
	trainCmd.PersistentFlags().StringVar(&trainStateDir, "state-dir", "", "Directory for train state storage")

	trainCreateCmd.Flags().StringVar(&trainRepos, "repos", "", "Comma-separated list of repositories")
	_ = trainCreateCmd.MarkFlagRequired("repos")

	trainPromoteCmd.Flags().StringVar(&trainFrom, "from", "", "Source channel")
	trainPromoteCmd.Flags().StringVar(&trainTo, "to", "", "Target channel")
	_ = trainPromoteCmd.MarkFlagRequired("from")
	_ = trainPromoteCmd.MarkFlagRequired("to")

	trainCmd.AddCommand(trainListCmd)
	trainCmd.AddCommand(trainCreateCmd)
	trainCmd.AddCommand(trainStatusCmd)
	trainCmd.AddCommand(trainPromoteCmd)
	trainCmd.AddCommand(trainDeleteCmd)
}

func parseRepoList(s string) []string {
	if s == "" {
		return nil
	}
	parts := strings.Split(s, ",")
	var result []string
	for _, p := range parts {
		p = strings.TrimSpace(p)
		if p != "" {
			result = append(result, p)
		}
	}
	return result
}
