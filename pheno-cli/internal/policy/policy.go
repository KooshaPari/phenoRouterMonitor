// Package policy provides centralized policy management for pheno-cli.
package policy

import (
	"context"
	"fmt"
	"os"
	"path/filepath"
	"strings"

	"github.com/pelletier/go-toml/v2"
)

// OrgConfig represents organization-wide policy configuration.
type OrgConfig struct {
	Organization    OrganizationInfo    `toml:"organization"`
	RequiredStandards RequiredStandards `toml:"required_standards"`
	Gating          GatingConfig        `toml:"gating"`
	Notifications   NotificationConfig  `toml:"notifications"`
}

// OrganizationInfo contains org metadata.
type OrganizationInfo struct {
	Name        string `toml:"name"`
	Domain      string `toml:"domain"`
	ContactEmail string `toml:"contact_email"`
}

// RequiredStandards defines mandatory files and configurations.
type RequiredStandards struct {
	Files  []string `toml:"files"`
	Hooks  []string `toml:"hooks"`
	CI     CIConfig `toml:"ci"`
}

// CIConfig defines CI/CD requirements.
type CIConfig struct {
	RequiredWorkflows []string `toml:"required_workflows"`
	RequiredChecks    []string `toml:"required_checks"`
}

// GatingConfig defines promotion gate rules.
type GatingConfig struct {
	Enabled       bool              `toml:"enabled"`
	RiskProfiles  []RiskProfile     `toml:"risk_profiles"`
}

// RiskProfile defines gate requirements for a risk level.
type RiskProfile struct {
	Name        string       `toml:"name"`
	Description string       `toml:"description"`
	Gates       []GateConfig `toml:"gates"`
}

// GateConfig defines a single gate.
type GateConfig struct {
	ID          string `toml:"id"`
	Name        string `toml:"name"`
	Description string `toml:"description"`
	Required    bool   `toml:"required"`
}

// NotificationConfig defines alert settings.
type NotificationConfig struct {
	SlackWebhook string `toml:"slack_webhook,omitempty"`
	Email      string `toml:"email,omitempty"`
}

// DefaultConfig returns a default organization configuration.
func DefaultConfig() *OrgConfig {
	return &OrgConfig{
		Organization: OrganizationInfo{
			Name:         "Phenotype Org",
			Domain:       "phenotype.io",
			ContactEmail: "admin@phenotype.io",
		},
		RequiredStandards: RequiredStandards{
			Files: []string{
				"README.md",
				"LICENSE",
				"SECURITY.md",
				"CODEOWNERS",
				"CONTRIBUTING.md",
			},
			Hooks: []string{
				"pre-commit",
				"pre-push",
			},
			CI: CIConfig{
				RequiredWorkflows: []string{
					"ci.yml",
					"release.yml",
					"security.yml",
				},
				RequiredChecks: []string{
					"test",
					"lint",
					"build",
				},
			},
		},
		Gating: GatingConfig{
			Enabled: true,
			RiskProfiles: []RiskProfile{
				{
					Name:        "low",
					Description: "Low risk - minimal gates",
					Gates: []GateConfig{
						{ID: "lint", Name: "Linting", Description: "Code must pass linting", Required: true},
						{ID: "test", Name: "Unit Tests", Description: "Unit tests must pass", Required: true},
					},
				},
				{
					Name:        "medium",
					Description: "Medium risk - standard gates",
					Gates: []GateConfig{
						{ID: "lint", Name: "Linting", Description: "Code must pass linting", Required: true},
						{ID: "test", Name: "Unit Tests", Description: "Unit tests must pass", Required: true},
						{ID: "integration", Name: "Integration Tests", Description: "Integration tests must pass", Required: true},
						{ID: "security", Name: "Security Scan", Description: "Security audit must pass", Required: true},
					},
				},
				{
					Name:        "high",
					Description: "High risk - all gates required",
					Gates: []GateConfig{
						{ID: "lint", Name: "Linting", Description: "Code must pass linting", Required: true},
						{ID: "test", Name: "Unit Tests", Description: "Unit tests must pass", Required: true},
						{ID: "integration", Name: "Integration Tests", Description: "Integration tests must pass", Required: true},
						{ID: "security", Name: "Security Scan", Description: "Security audit must pass", Required: true},
						{ID: "review", Name: "Code Review", Description: "Must have 2+ approvals", Required: true},
						{ID: "docs", Name: "Documentation", Description: "Docs must be updated", Required: true},
					},
				},
			},
		},
	}
}

// LoadConfig loads organization policy from a TOML file.
func LoadConfig(path string) (*OrgConfig, error) {
	data, err := os.ReadFile(path)
	if err != nil {
		if os.IsNotExist(err) {
			return DefaultConfig(), nil
		}
		return nil, fmt.Errorf("failed to read config: %w", err)
	}

	var cfg OrgConfig
	if err := toml.Unmarshal(data, &cfg); err != nil {
		return nil, fmt.Errorf("failed to parse config: %w", err)
	}

	return &cfg, nil
}

// SaveConfig saves organization policy to a TOML file.
func SaveConfig(path string, cfg *OrgConfig) error {
	data, err := toml.Marshal(cfg)
	if err != nil {
		return fmt.Errorf("failed to marshal config: %w", err)
	}

	dir := filepath.Dir(path)
	if err := os.MkdirAll(dir, 0755); err != nil {
		return fmt.Errorf("failed to create config dir: %w", err)
	}

	if err := os.WriteFile(path, data, 0644); err != nil {
		return fmt.Errorf("failed to write config: %w", err)
	}

	return nil
}

// Validate checks if the organization configuration is valid.
func (c *OrgConfig) Validate() error {
	if c.Organization.Name == "" {
		return fmt.Errorf("organization name is required")
	}

	for _, profile := range c.Gating.RiskProfiles {
		if profile.Name == "" {
			return fmt.Errorf("risk profile name is required")
		}
		for _, gate := range profile.Gates {
			if gate.ID == "" {
				return fmt.Errorf("gate ID is required in profile %s", profile.Name)
			}
		}
	}

	return nil
}

// FindRiskProfile returns a risk profile by name.
func (c *OrgConfig) FindRiskProfile(name string) (*RiskProfile, bool) {
	for i := range c.Gating.RiskProfiles {
		if strings.EqualFold(c.Gating.RiskProfiles[i].Name, name) {
			return &c.Gating.RiskProfiles[i], true
		}
	}
	return nil, false
}

// Context carries policy context through operations.
type Context struct {
	Config  *OrgConfig
	RepoPath string
	RepoName string
}

// NewContext creates a new policy context.
func NewContext(cfg *OrgConfig, repoPath string) *Context {
	return &Context{
		Config:   cfg,
		RepoPath: repoPath,
		RepoName: filepath.Base(repoPath),
	}
}

// EvaluateGate evaluates a specific gate against the repository.
func (ctx *Context) EvaluateGate(ctx2 context.Context, gateID string) (bool, error) {
	switch gateID {
	case "lint":
		return ctx.checkLinting()
	case "test":
		return ctx.checkTests()
	case "integration":
		return ctx.checkIntegrationTests()
	case "security":
		return ctx.checkSecurity()
	case "review":
		return ctx.checkReviews()
	case "docs":
		return ctx.checkDocumentation()
	default:
		return false, fmt.Errorf("unknown gate: %s", gateID)
	}
}

// checkLinting checks if the code passes linting.
func (ctx *Context) checkLinting() (bool, error) {
	// This would run the actual linter based on language
	// For now, return true as placeholder
	return true, nil
}

// checkTests checks if unit tests pass.
func (ctx *Context) checkTests() (bool, error) {
	// This would run unit tests
	return true, nil
}

// checkIntegrationTests checks if integration tests pass.
func (ctx *Context) checkIntegrationTests() (bool, error) {
	// This would run integration tests
	return true, nil
}

// checkSecurity checks if security scan passes.
func (ctx *Context) checkSecurity() (bool, error) {
	// This would run security scan
	return true, nil
}

// checkReviews checks if code review requirements are met.
func (ctx *Context) checkReviews() (bool, error) {
	// This would check GitHub PR approvals
	return true, nil
}

// checkDocumentation checks if documentation is updated.
func (ctx *Context) checkDocumentation() (bool, error) {
	// This would check if docs are current
	return true, nil
}
