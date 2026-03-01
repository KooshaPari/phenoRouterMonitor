package cmd

import (
	"fmt"
	"os"

	"github.com/spf13/cobra"
	"github.com/spf13/viper"
)

var rootCmd = &cobra.Command{
	Use:   "pheno",
	Short: "Phenotype release governance and DX CLI",
	Long:  "Org-wide release governance, automated publishing, and developer experience tooling for Phenotype repositories.",
}

func Execute() {
	if err := rootCmd.Execute(); err != nil {
		fmt.Fprintln(os.Stderr, err)
		os.Exit(1)
	}
}

func init() {
	cobra.OnInitialize(initConfig)
	rootCmd.PersistentFlags().String("config", "", "config file (default: ~/.config/pheno/config.toml)")
	rootCmd.PersistentFlags().Bool("verbose", false, "verbose output")

	rootCmd.AddCommand(publishCmd)
	rootCmd.AddCommand(promoteCmd)
	rootCmd.AddCommand(auditCmd)
	rootCmd.AddCommand(bootstrapCmd)
	rootCmd.AddCommand(matrixCmd)
	rootCmd.AddCommand(configCmd)
}

func initConfig() {
	cfgFile, _ := rootCmd.PersistentFlags().GetString("config")
	if cfgFile != "" {
		viper.SetConfigFile(cfgFile)
	} else {
		home, _ := os.UserHomeDir()
		viper.AddConfigPath(home + "/.config/pheno")
		viper.SetConfigName("config")
		viper.SetConfigType("toml")
	}
	viper.AutomaticEnv()
	viper.SetEnvPrefix("PHENO")
	_ = viper.ReadInConfig()
}

var publishCmd = &cobra.Command{
	Use:   "publish",
	Short: "Publish packages to their registries",
	RunE: func(cmd *cobra.Command, args []string) error {
		fmt.Println("publish: not yet implemented")
		return nil
	},
}

var promoteCmd = &cobra.Command{
	Use:   "promote [channel]",
	Short: "Promote packages to a release channel with gate checks",
	Args:  cobra.ExactArgs(1),
	RunE: func(cmd *cobra.Command, args []string) error {
		fmt.Printf("promote to %s: not yet implemented\n", args[0])
		return nil
	},
}

var auditCmd = &cobra.Command{
	Use:   "audit",
	Short: "Audit release status across repositories",
	RunE: func(cmd *cobra.Command, args []string) error {
		fmt.Println("audit: not yet implemented")
		return nil
	},
}

var bootstrapCmd = &cobra.Command{
	Use:   "bootstrap",
	Short: "Bootstrap governance artifacts for a repository",
	RunE: func(cmd *cobra.Command, args []string) error {
		fmt.Println("bootstrap: not yet implemented")
		return nil
	},
}

var matrixCmd = &cobra.Command{
	Use:   "matrix",
	Short: "Generate release matrix",
	RunE: func(cmd *cobra.Command, args []string) error {
		fmt.Println("matrix: not yet implemented")
		return nil
	},
}

var configCmd = &cobra.Command{
	Use:   "config",
	Short: "Manage CLI configuration",
	RunE: func(cmd *cobra.Command, args []string) error {
		fmt.Println("config: not yet implemented")
		return nil
	},
}
