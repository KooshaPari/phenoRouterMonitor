package state

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"sort"
	"strings"
	"sync"
	"time"
)

// Channel represents a release channel.
type Channel int

const (
	ChannelAlpha Channel = iota
	ChannelCanary
	ChannelBeta
	ChannelRC
	ChannelProd
)

func (c Channel) String() string {
	switch c {
	case ChannelAlpha:
		return "alpha"
	case ChannelCanary:
		return "canary"
	case ChannelBeta:
		return "beta"
	case ChannelRC:
		return "rc"
	case ChannelProd:
		return "prod"
	default:
		return "unknown"
	}
}

// ChannelFromString converts a string to a Channel.
func ChannelFromString(s string) (Channel, bool) {
	switch s {
	case "alpha":
		return ChannelAlpha, true
	case "canary":
		return ChannelCanary, true
	case "beta":
		return ChannelBeta, true
	case "rc":
		return ChannelRC, true
	case "prod":
		return ChannelProd, true
	default:
		return ChannelAlpha, false
	}
}

// RepoState represents the state of a repository in a release train.
type RepoState struct {
	Name      string    `json:"name"`
	Channel   string    `json:"channel"`
	Version   string    `json:"version,omitempty"`
	UpdatedAt time.Time `json:"updated_at"`
	Error     string    `json:"error,omitempty"`
}

// ReleaseTrain represents a release train with multiple repositories.
type ReleaseTrain struct {
	ID            string                 `json:"id"`
	Name          string                 `json:"name"`
	Description   string                 `json:"description,omitempty"`
	Repos         []string               `json:"repos"`
	Dependencies  map[string][]string    `json:"dependencies,omitempty"`
	Status        map[string]RepoState   `json:"status"`
	CreatedAt     time.Time              `json:"created_at"`
	UpdatedAt     time.Time              `json:"updated_at"`
	TargetChannel string                 `json:"target_channel,omitempty"`
	Metadata      map[string]interface{} `json:"metadata,omitempty"`
}

// TrainManager manages release trains state.
type TrainManager struct {
	mu       sync.RWMutex
	stateDir string
	trains   map[string]*ReleaseTrain
}

// NewTrainManager creates a new train manager.
func NewTrainManager(stateDir string) *TrainManager {
	mgr := &TrainManager{
		stateDir: stateDir,
		trains:   make(map[string]*ReleaseTrain),
	}
	// Load existing state
	_ = mgr.LoadState()
	return mgr
}

// CreateTrain creates a new release train.
func (m *TrainManager) CreateTrain(name string, repos []string, deps map[string][]string) (*ReleaseTrain, error) {
	m.mu.Lock()
	defer m.mu.Unlock()

	// Check if train already exists
	for _, t := range m.trains {
		if t.Name == name {
			return nil, fmt.Errorf("train with name %s already exists", name)
		}
	}

	train := &ReleaseTrain{
		ID:           generateID(),
		Name:         name,
		Repos:        repos,
		Dependencies: deps,
		Status:       make(map[string]RepoState),
		CreatedAt:    time.Now(),
		UpdatedAt:    time.Now(),
	}

	// Initialize status for all repos
	for _, repo := range repos {
		train.Status[repo] = RepoState{
			Name:    repo,
			Channel: "alpha",
		}
	}

	m.trains[train.ID] = train

	// Persist state
	if err := m.saveState(); err != nil {
		return nil, fmt.Errorf("failed to save state: %w", err)
	}

	return train, nil
}

// GetTrain retrieves a train by ID.
func (m *TrainManager) GetTrain(id string) (*ReleaseTrain, bool) {
	m.mu.RLock()
	defer m.mu.RUnlock()

	train, ok := m.trains[id]
	return train, ok
}

// GetTrainByName retrieves a train by name.
func (m *TrainManager) GetTrainByName(name string) (*ReleaseTrain, bool) {
	m.mu.RLock()
	defer m.mu.RUnlock()

	for _, train := range m.trains {
		if train.Name == name {
			return train, true
		}
	}
	return nil, false
}

// ListTrains returns all release trains.
func (m *TrainManager) ListTrains() []*ReleaseTrain {
	m.mu.RLock()
	defer m.mu.RUnlock()

	var result []*ReleaseTrain
	for _, train := range m.trains {
		result = append(result, train)
	}

	// Sort by created time
	sort.Slice(result, func(i, j int) bool {
		return result[i].CreatedAt.Before(result[j].CreatedAt)
	})

	return result
}

// PromoteTrain promotes a train to a specific channel.
func (m *TrainManager) PromoteTrain(trainID string, toChannel string) error {
	m.mu.Lock()
	defer m.mu.Unlock()

	train, ok := m.trains[trainID]
	if !ok {
		return fmt.Errorf("train %s not found", trainID)
	}

	// Validate channel
	targetCh, ok := ChannelFromString(toChannel)
	if !ok {
		return fmt.Errorf("invalid channel: %s", toChannel)
	}

	// Check dependencies
	for _, repo := range train.Repos {
		if deps, ok := train.Dependencies[repo]; ok {
			for _, dep := range deps {
				depState, ok := train.Status[dep]
				if !ok {
					return fmt.Errorf("dependency %s for repo %s not found in train", dep, repo)
				}
				depCh, _ := ChannelFromString(depState.Channel)
				if depCh < targetCh {
					return fmt.Errorf("repo %s depends on %s which is at channel %s (need >= %s)", repo, dep, depState.Channel, toChannel)
				}
			}
		}
	}

	// Update status
	for _, repo := range train.Repos {
		state := train.Status[repo]
		state.Channel = toChannel
		state.UpdatedAt = time.Now()
		train.Status[repo] = state
	}

	train.TargetChannel = toChannel
	train.UpdatedAt = time.Now()

	// Persist state
	if err := m.saveState(); err != nil {
		return fmt.Errorf("failed to save state: %w", err)
	}

	return nil
}

// DeleteTrain deletes a release train.
func (m *TrainManager) DeleteTrain(trainID string) error {
	m.mu.Lock()
	defer m.mu.Unlock()

	if _, ok := m.trains[trainID]; !ok {
		return fmt.Errorf("train %s not found", trainID)
	}

	delete(m.trains, trainID)

	// Persist state
	if err := m.saveState(); err != nil {
		return fmt.Errorf("failed to save state: %w", err)
	}

	return nil
}

// GetTrainStatus returns the status of all repos in a train.
func (m *TrainManager) GetTrainStatus(trainID string) (map[string]RepoState, error) {
	m.mu.RLock()
	defer m.mu.RUnlock()

	train, ok := m.trains[trainID]
	if !ok {
		return nil, fmt.Errorf("train %s not found", trainID)
	}

	return train.Status, nil
}

// UpdateRepoStatus updates the status of a specific repo in a train.
func (m *TrainManager) UpdateRepoStatus(trainID, repo string, state RepoState) error {
	m.mu.Lock()
	defer m.mu.Unlock()

	train, ok := m.trains[trainID]
	if !ok {
		return fmt.Errorf("train %s not found", trainID)
	}

	train.Status[repo] = state
	train.UpdatedAt = time.Now()

	return m.saveState()
}

// saveState persists the state to disk.
func (m *TrainManager) saveState() error {
	if err := os.MkdirAll(m.stateDir, 0755); err != nil {
		return fmt.Errorf("failed to create state directory: %w", err)
	}

	stateFile := filepath.Join(m.stateDir, "trains.json")
	data, err := json.MarshalIndent(m.trains, "", "  ")
	if err != nil {
		return fmt.Errorf("failed to marshal state: %w", err)
	}

	if err := os.WriteFile(stateFile, data, 0644); err != nil {
		return fmt.Errorf("failed to write state file: %w", err)
	}

	return nil
}

// LoadState loads state from disk.
func (m *TrainManager) LoadState() error {
	stateFile := filepath.Join(m.stateDir, "trains.json")
	data, err := os.ReadFile(stateFile)
	if err != nil {
		if os.IsNotExist(err) {
			return nil
		}
		return fmt.Errorf("failed to read state file: %w", err)
	}

	if err := json.Unmarshal(data, &m.trains); err != nil {
		return fmt.Errorf("failed to unmarshal state: %w", err)
	}

	return nil
}

// generateID generates a unique train ID.
func generateID() string {
	return fmt.Sprintf("train_%d", time.Now().UnixNano())
}

// FormatTrainSummary returns a human-readable summary of a train.
func FormatTrainSummary(train *ReleaseTrain) string {
	var sb strings.Builder

	sb.WriteString(fmt.Sprintf("Train: %s (ID: %s)\n", train.Name, train.ID))
	if train.Description != "" {
		sb.WriteString(fmt.Sprintf("Description: %s\n", train.Description))
	}
	sb.WriteString(fmt.Sprintf("Created: %s\n", train.CreatedAt.Format(time.RFC3339)))
	sb.WriteString(fmt.Sprintf("Updated: %s\n", train.UpdatedAt.Format(time.RFC3339)))
	if train.TargetChannel != "" {
		sb.WriteString(fmt.Sprintf("Target Channel: %s\n", train.TargetChannel))
	}
	sb.WriteString(fmt.Sprintf("Repositories (%d):\n", len(train.Repos)))

	for _, repo := range train.Repos {
		status := train.Status[repo]
		sb.WriteString(fmt.Sprintf("  - %s: %s", repo, status.Channel))
		if status.Version != "" {
			sb.WriteString(fmt.Sprintf(" (v%s)", status.Version))
		}
		if status.Error != "" {
			sb.WriteString(fmt.Sprintf(" [ERROR: %s]", status.Error))
		}
		sb.WriteString("\n")
	}

	return sb.String()
}
