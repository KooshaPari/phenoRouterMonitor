package cmd

import (
	"fmt"
	"os"
	"text/tabwriter"

	"github.com/KooshaPari/pheno-cli/internal/state"
	"github.com/spf13/cobra"
)

var (
	trainRepos    []string
	trainStateDir string
	trainToChan   string
)

var trainCmd = &cobra.Command{
	Use:   "train",
	Short: "Manage release trains",
	Long:  `Release trains coordinate multi-repo releases.`,
}

var trainListCmd = &cobra.Command{
	Use:   "list",
	Short: "List all release trains",
	RunE:  runTrainList,
}

func runTrainList(cmd *cobra.Command, args []string) error {
	mgr := state.NewTrainManager(trainStateDir)
	trains := mgr.ListTrains()
	if len(trains) == 0 {
		fmt.Println("No release trains found.")
		return nil
	}
	w := tabwriter.NewWriter(os.Stdout, 0, 0, 2, ' ', 0)
	fmt.Fprintf(w, "NAME\tID\tREPOS\tTARGET\n")
	for _, t := range trains {
		target := t.TargetChannel
		if target == "" {
			target = "-"
		}
		fmt.Fprintf(w, "%s\t%s\t%d\t%s\n", t.Name, t.ID, len(t.Repos), target)
	}
	w.Flush()
	return nil
}

var trainCreateCmd = &cobra.Command{
	Use:   "create [name]",
	Short: "Create a release train",
	Args:  cobra.ExactArgs(1),
	RunE:  runTrainCreate,
}

func runTrainCreate(cmd *cobra.Command, args []string) error {
	mgr := state.NewTrainManager(trainStateDir)
	if len(trainRepos) == 0 {
		return fmt.Errorf("use --repos flag")
	}
	_, err := mgr.CreateTrain(args[0], trainRepos, nil)
	if err != nil {
		return err
	}
	fmt.Printf("Created train '%s'\n", args[0])
	return nil
}

var trainStatusCmd = &cobra.Command{
	Use:   "status [name]",
	Short: "Show train status",
	Args:  cobra.ExactArgs(1),
	RunE:  runTrainStatus,
}

func runTrainStatus(cmd *cobra.Command, args []string) error {
	mgr := state.NewTrainManager(trainStateDir)
	trains := mgr.ListTrains()
	for _, t := range trains {
		if t.Name == args[0] {
			target := t.TargetChannel
			if target == "" {
				target = "-"
			}
			fmt.Printf("Train: %s\nID: %s\nRepos: %d\nTarget: %s\n", t.Name, t.ID, len(t.Repos), target)
			return nil
		}
	}
	return fmt.Errorf("train not found")
}

var trainPromoteCmd = &cobra.Command{
	Use:   "promote [name]",
	Short: "Promote train",
	Args:  cobra.ExactArgs(1),
	RunE:  runTrainPromote,
}

func runTrainPromote(cmd *cobra.Command, args []string) error {
	mgr := state.NewTrainManager(trainStateDir)
	if err := mgr.PromoteTrain(args[0], trainToChan); err != nil {
		return err
	}
	fmt.Printf("Promoted train '%s' to %s\n", args[0], trainToChan)
	return nil
}

var trainDeleteCmd = &cobra.Command{
	Use:   "delete [name]",
	Short: "Delete train",
	Args:  cobra.ExactArgs(1),
	RunE:  runTrainDelete,
}

func runTrainDelete(cmd *cobra.Command, args []string) error {
	mgr := state.NewTrainManager(trainStateDir)
	if err := mgr.DeleteTrain(args[0]); err != nil {
		return err
	}
	fmt.Printf("Deleted train '%s'\n", args[0])
	return nil
}

func init() {
	trainCmd.AddCommand(trainListCmd, trainCreateCmd, trainStatusCmd, trainPromoteCmd, trainDeleteCmd)
	trainCmd.PersistentFlags().StringVar(&trainStateDir, "state-dir", "", "State directory")
	trainCreateCmd.Flags().StringSliceVar(&trainRepos, "repos", nil, "Repositories")
	trainPromoteCmd.Flags().StringVar(&trainToChan, "to", "", "Target channel")
}
