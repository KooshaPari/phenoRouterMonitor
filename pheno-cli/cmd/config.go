package cmd

import (
	"fmt"
	"os"
	"path/filepath"

	"github.com/spf13/cobra"
	"github.com/spf13/viper"
)

var configCmd = &cobra.Command{
	Use:   "config",
	Short: "Manage CLI configuration",
	Long:  "Manage pheno-cli configuration settings and credentials.",
	RunE:  runConfig,
}

var (
	configSetKey   string
	configSetValue string
	configGetKey   string
)

func init() {
	configCmd.Flags().StringVar(&configSetKey, "set-key", "", "Configuration key to set")
	configCmd.Flags().StringVar(&configSetValue, "set-value", "", "Value to set for the key")
	configCmd.Flags().StringVar(&configGetKey, "get", "", "Configuration key to get")
}

func runConfig(cmd *cobra.Command, args []string) error {
	// Get config directory
	homeDir, err := os.UserHomeDir()
	if err != nil {
		return fmt.Errorf("failed to get home directory: %w", err)
	}
	configDir := filepath.Join(homeDir, ".config", "pheno")

	// Handle --get
	if configGetKey != "" {
		value := viper.GetString(configGetKey)
		if value == "" {
			fmt.Printf("%s is not set\n", configGetKey)
		} else {
			fmt.Printf("%s=%s\n", configGetKey, value)
		}
		return nil
	}

	// Handle --set-key
	if configSetKey != "" {
		viper.Set(configSetKey, configSetValue)
		configFile := filepath.Join(configDir, "config.toml")
		if err := viper.WriteConfigAs(configFile); err != nil {
			// Try creating the directory first
			if err := os.MkdirAll(configDir, 0755); err != nil {
				return fmt.Errorf("failed to create config directory: %w", err)
			}
			if err := viper.WriteConfigAs(configFile); err != nil {
				return fmt.Errorf("failed to write config: %w", err)
			}
		}
		fmt.Printf("Set %s=%s\n", configSetKey, configSetValue)
		return nil
	}

	// Show config path
	fmt.Printf("Config directory: %s\n", configDir)
	fmt.Printf("Config file: %s\n", filepath.Join(configDir, "config.toml"))
	fmt.Println("\nUse --get <key> to retrieve a value")
	fmt.Println("Use --set-key <key> --set-value <value> to set a value")

	return nil
}
