package cmd

import (
	"fmt"
	"os"
	"text/tabwriter"

	"github.com/spf13/cobra"
	"github.com/KooshaPari/pheno-cli/internal/state"
)

var (
	trainName     string
	trainRepos    []string
	trainStateDir string
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
	mgr := state.NewTrainManager("")
	trains, err := mgr.ListTrains()
	if err != nil {
		return fmt.Errorf("failed to list trains: %w", err)
	}

	if len(trains) == 0 {
		fmt.Println("No release trains found.")
		return nil
	}

	w := tabwriter.NewWriter(os.Stdout, 0, 0, 2, ' ', 0)
	fmt.Fprintf(w, "NAME\tCHANNEL\tREPOS\n")
	for _, t := range trains {
		fmt.Fprintf(w, "%s\t%s\t%d\n", t.Name, t.Channel, len(t.Repos))
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
	trains, err := mgr.ListTrains()
	if err != nil {
		return err
	}
	for _, t := range trains {
		if t.Name == args[0] {
			fmt.Printf("Train: %s\nChannel: %s\nRepos: %d\n", t.Name, t.Channel, len(t.Repos))
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

var trainToChannel string

func runTrainPromote(cmd *cobra.Command, args []string) error {
	mgr := state.NewTrainManager(trainStateDir)
	if err := mgr.PromoteTrain(args[0], trainToChannel); err != nil {
		return err
	}
	fmt.Printf("Promoted train '%s' to %s\n", args[0], trainToChannel)
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
	trainPromoteCmd.Flags().StringVar(&trainToChannel, "to", "", "Target channel")
}
