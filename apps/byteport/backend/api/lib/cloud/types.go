// Package cloud provides a unified abstraction layer for multi-cloud deployments
// supporting AWS, GCP, Azure, Vercel, Render, Supabase, Fly.io, Neon, and PlanetScale
package cloud

import (
	"time"
)

// ResourceType represents the category of cloud resource
type ResourceType string

const (
	// Compute resource types
	ResourceTypeComputeVM        ResourceType = "compute.vm"
	ResourceTypeComputeContainer ResourceType = "compute.container"
	ResourceTypeComputeFunction  ResourceType = "compute.function"
	ResourceTypeComputeEdge      ResourceType = "compute.edge"

	// Database resource types
	ResourceTypeDatabaseSQL        ResourceType = "database.sql"
	ResourceTypeDatabaseNoSQL      ResourceType = "database.nosql"
	ResourceTypeDatabaseServerless ResourceType = "database.serverless"

	// Storage resource types
	ResourceTypeStorageObject ResourceType = "storage.object"
	ResourceTypeStorageBlock  ResourceType = "storage.block"
	ResourceTypeStorageFile   ResourceType = "storage.file"

	// Network resource types
	ResourceTypeNetworkLoadBalancer ResourceType = "network.loadbalancer"
	ResourceTypeNetworkCDN          ResourceType = "network.cdn"
	ResourceTypeNetworkDNS          ResourceType = "network.dns"
	ResourceTypeNetworkVPC          ResourceType = "network.vpc"
)

// DeploymentState represents the current state of a deployment
type DeploymentState string

const (
	DeploymentStatePending      DeploymentState = "PENDING"
	DeploymentStateValidating   DeploymentState = "VALIDATING"
	DeploymentStateBuilding     DeploymentState = "BUILDING"
	DeploymentStateProvisioning DeploymentState = "PROVISIONING"
	DeploymentStateDeploying    DeploymentState = "DEPLOYING"
	DeploymentStateHealthCheck  DeploymentState = "HEALTH_CHECK"
	DeploymentStateActive       DeploymentState = "ACTIVE"
	DeploymentStateUpdating     DeploymentState = "UPDATING"
	DeploymentStateScaling      DeploymentState = "SCALING"
	DeploymentStateDegraded     DeploymentState = "DEGRADED"
	DeploymentStateFailed       DeploymentState = "FAILED"
	DeploymentStateRollingBack  DeploymentState = "ROLLING_BACK"
	DeploymentStateDeleting     DeploymentState = "DELETING"
	DeploymentStateDeleted      DeploymentState = "DELETED"
)

// DeploymentStrategy defines how deployments are executed
type DeploymentStrategy string

const (
	DeploymentStrategyRolling   DeploymentStrategy = "rolling"
	DeploymentStrategyBlueGreen DeploymentStrategy = "bluegreen"
	DeploymentStrategyCanary    DeploymentStrategy = "canary"
	DeploymentStrategyAtomic    DeploymentStrategy = "atomic"
	DeploymentStrategyRecreate  DeploymentStrategy = "recreate"
)

// Capability represents optional provider capabilities
type Capability string

const (
	CapabilityScalable   Capability = "scalable"
	CapabilityLoggable   Capability = "loggable"
	CapabilityExecutable Capability = "executable"
	CapabilityBackupable Capability = "backupable"
	CapabilityMonitoring Capability = "monitoring"
	CapabilityAutoScale  Capability = "autoscale"
	CapabilityCustomDNS  Capability = "custom_dns"
	CapabilitySSH        Capability = "ssh"
)

// HealthStatus represents the health state of a resource
type HealthStatus string

const (
	HealthStatusUnknown   HealthStatus = "UNKNOWN"
	HealthStatusHealthy   HealthStatus = "HEALTHY"
	HealthStatusDegraded  HealthStatus = "DEGRADED"
	HealthStatusUnhealthy HealthStatus = "UNHEALTHY"
	HealthStatusChecking  HealthStatus = "CHECKING"
)

// ResourceConfig represents the configuration for creating/updating a resource
type ResourceConfig struct {
	Name     string            `json:"name" yaml:"name"`
	Type     ResourceType      `json:"type" yaml:"type"`
	Provider string            `json:"provider" yaml:"provider"`
	Region   string            `json:"region,omitempty" yaml:"region,omitempty"`
	Tags     map[string]string `json:"tags,omitempty" yaml:"tags,omitempty"`
	Spec     map[string]any    `json:"spec" yaml:"spec"` // Provider-specific configuration
	Deploy   *DeployConfig     `json:"deploy,omitempty" yaml:"deploy,omitempty"`
}

// DeployConfig contains deployment-specific settings
type DeployConfig struct {
	Strategy    DeploymentStrategy `json:"strategy" yaml:"strategy"`
	HealthCheck *HealthCheckConfig `json:"health_check,omitempty" yaml:"health_check,omitempty"`
	Timeout     time.Duration      `json:"timeout,omitempty" yaml:"timeout,omitempty"`
	Rollback    *RollbackConfig    `json:"rollback,omitempty" yaml:"rollback,omitempty"`
}

// HealthCheckConfig defines health check parameters
type HealthCheckConfig struct {
	Type             string        `json:"type" yaml:"type"` // http, tcp, command, custom
	Path             string        `json:"path,omitempty" yaml:"path,omitempty"`
	Port             int           `json:"port,omitempty" yaml:"port,omitempty"`
	Command          string        `json:"command,omitempty" yaml:"command,omitempty"`
	Interval         time.Duration `json:"interval" yaml:"interval"`
	Timeout          time.Duration `json:"timeout" yaml:"timeout"`
	Retries          int           `json:"retries" yaml:"retries"`
	InitialDelay     time.Duration `json:"initial_delay" yaml:"initial_delay"`
	SuccessThreshold int           `json:"success_threshold" yaml:"success_threshold"`
	FailureThreshold int           `json:"failure_threshold" yaml:"failure_threshold"`
}

// RollbackConfig defines rollback behavior
type RollbackConfig struct {
	Enabled       bool          `json:"enabled" yaml:"enabled"`
	MaxRetries    int           `json:"max_retries" yaml:"max_retries"`
	RetryInterval time.Duration `json:"retry_interval" yaml:"retry_interval"`
}

// Resource represents a deployed cloud resource
type Resource struct {
	ID             string            `json:"id"`
	Name           string            `json:"name"`
	Type           ResourceType      `json:"type"`
	Provider       string            `json:"provider"`
	Region         string            `json:"region,omitempty"`
	Status         DeploymentState   `json:"status"`
	HealthStatus   HealthStatus      `json:"health_status"`
	Tags           map[string]string `json:"tags,omitempty"`
	Endpoints      []Endpoint        `json:"endpoints,omitempty"`
	Metadata       map[string]any    `json:"metadata,omitempty"` // Provider-specific data
	CreatedAt      time.Time         `json:"created_at"`
	StartedAt      time.Time         `json:"started_at,omitempty"`
	UpdatedAt      time.Time         `json:"updated_at"`
	LastDeployedAt *time.Time        `json:"last_deployed_at,omitempty"`
}

// Endpoint represents a network endpoint for a resource
type Endpoint struct {
	Type     string `json:"type"` // http, https, tcp, grpc
	URL      string `json:"url"`
	Port     int    `json:"port,omitempty"`
	Protocol string `json:"protocol,omitempty"`
	Primary  bool   `json:"primary"`
}

// DeploymentConfig contains full deployment configuration
type DeploymentConfig struct {
	ResourceID string             `json:"resource_id"`
	Version    string             `json:"version,omitempty"`
	Source     *DeploymentSource  `json:"source"`
	Env        map[string]string  `json:"env,omitempty"`
	Secrets    map[string]string  `json:"secrets,omitempty"`
	Strategy   DeploymentStrategy `json:"strategy"`
	Config     map[string]any     `json:"config,omitempty"` // Provider-specific
}

// DeploymentSource defines where the deployment code comes from
type DeploymentSource struct {
	Type       string            `json:"type"` // git, docker, archive, inline
	Repository string            `json:"repository,omitempty"`
	Branch     string            `json:"branch,omitempty"`
	Commit     string            `json:"commit,omitempty"`
	Image      string            `json:"image,omitempty"`
	Tag        string            `json:"tag,omitempty"`
	Path       string            `json:"path,omitempty"`
	Metadata   map[string]string `json:"metadata,omitempty"`
}

// Deployment represents a deployment instance
type Deployment struct {
	ID         string             `json:"id"`
	ResourceID string             `json:"resource_id"`
	Version    string             `json:"version"`
	State      DeploymentState    `json:"state"`
	Strategy   DeploymentStrategy `json:"strategy"`
	Progress   int                `json:"progress"` // 0-100
	Message    string             `json:"message,omitempty"`
	StartedAt  time.Time          `json:"started_at"`
	UpdatedAt  time.Time          `json:"updated_at"`
	FinishedAt *time.Time         `json:"finished_at,omitempty"`
	Error      *DeploymentError   `json:"error,omitempty"`
}

// DeploymentError contains error details from a failed deployment
type DeploymentError struct {
	Code    string `json:"code"`
	Message string `json:"message"`
	Details any    `json:"details,omitempty"`
}

// DeploymentStatus provides detailed status information
type DeploymentStatus struct {
	Deployment      *Deployment    `json:"deployment"`
	Health          HealthStatus   `json:"health"`
	Instances       []InstanceInfo `json:"instances,omitempty"`
	LastHealthCheck *time.Time     `json:"last_health_check,omitempty"`
}

// InstanceInfo represents information about a running instance
type InstanceInfo struct {
	ID          string       `json:"id"`
	State       string       `json:"state"`
	Health      HealthStatus `json:"health"`
	Region      string       `json:"region,omitempty"`
	StartedAt   time.Time    `json:"started_at"`
	CPUUsage    *float64     `json:"cpu_usage,omitempty"`
	MemoryUsage *float64     `json:"memory_usage,omitempty"`
}

// ResourceFilter defines criteria for filtering resources
type ResourceFilter struct {
	Types     []ResourceType    `json:"types,omitempty"`
	Providers []string          `json:"providers,omitempty"`
	Regions   []string          `json:"regions,omitempty"`
	Tags      map[string]string `json:"tags,omitempty"`
	States    []DeploymentState `json:"states,omitempty"`
}

// LogOptions defines parameters for retrieving logs
type LogOptions struct {
	Since      *time.Time `json:"since,omitempty"`
	Until      *time.Time `json:"until,omitempty"`
	Tail       int        `json:"tail,omitempty"`        // Number of lines from end
	Follow     bool       `json:"follow,omitempty"`      // Stream logs
	Filter     string     `json:"filter,omitempty"`      // Filter pattern
	InstanceID string     `json:"instance_id,omitempty"` // Specific instance
}

// LogEntry represents a single log entry
type LogEntry struct {
	Timestamp  time.Time         `json:"timestamp"`
	Level      string            `json:"level,omitempty"`
	Message    string            `json:"message"`
	InstanceID string            `json:"instance_id,omitempty"`
	Metadata   map[string]string `json:"metadata,omitempty"`
}

// LogStream represents a stream of log entries
type LogStream interface {
	Next() (*LogEntry, error)
	Close() error
}

// MetricOptions defines parameters for retrieving metrics
type MetricOptions struct {
	Since       *time.Time `json:"since,omitempty"`
	Until       *time.Time `json:"until,omitempty"`
	Granularity string     `json:"granularity,omitempty"` // 1m, 5m, 1h, 1d
	InstanceID  string     `json:"instance_id,omitempty"`
	MetricNames []string   `json:"metric_names,omitempty"`
}

// Metric represents a time-series metric
type Metric struct {
	Name      string            `json:"name"`
	Value     float64           `json:"value"`
	Unit      string            `json:"unit"`
	Timestamp time.Time         `json:"timestamp"`
	Tags      map[string]string `json:"tags,omitempty"`
}

// CostEstimate represents estimated costs for a resource
type CostEstimate struct {
	HourlyUSD   float64            `json:"hourly_usd"`
	DailyUSD    float64            `json:"daily_usd"`
	MonthlyUSD  float64            `json:"monthly_usd"`
	Breakdown   map[string]float64 `json:"breakdown"`  // component -> cost
	Confidence  string             `json:"confidence"` // high, medium, low
	Currency    string             `json:"currency"`
	LastUpdated time.Time          `json:"last_updated"`
}

// Cost represents actual incurred costs
type Cost struct {
	TotalUSD  float64            `json:"total_usd"`
	Breakdown map[string]float64 `json:"breakdown"`
	StartTime time.Time          `json:"start_time"`
	EndTime   time.Time          `json:"end_time"`
	Currency  string             `json:"currency"`
}

// TimeRange represents a time range for queries
type TimeRange struct {
	Start time.Time `json:"start"`
	End   time.Time `json:"end"`
}

// Credentials represents authentication credentials for a provider
type Credentials struct {
	Type     string            `json:"type"` // iam, token, api_key, oauth, service_account
	Data     map[string]string `json:"data"` // Provider-specific credential data
	Region   string            `json:"region,omitempty"`
	Endpoint string            `json:"endpoint,omitempty"` // Custom API endpoint
}

// ProviderMetadata describes a cloud provider's capabilities
type ProviderMetadata struct {
	Name               string         `json:"name"`
	Version            string         `json:"version"`
	SupportedResources []ResourceType `json:"supported_resources"`
	Capabilities       []Capability   `json:"capabilities"`
	Regions            []Region       `json:"regions"`
	AuthTypes          []string       `json:"auth_types"`
	Description        string         `json:"description"`
}

// Region represents a geographic region
type Region struct {
	ID         string   `json:"id"`
	Name       string   `json:"name"`
	Location   string   `json:"location"` // e.g., "US West (Oregon)"
	Available  bool     `json:"available"`
	Deprecated bool     `json:"deprecated"`
	Zones      []string `json:"zones,omitempty"`
}

// ScaleConfig represents scaling configuration
type ScaleConfig struct {
	MinInstances    int          `json:"min_instances"`
	MaxInstances    int          `json:"max_instances"`
	TargetCPU       *int         `json:"target_cpu,omitempty"`      // 0-100
	TargetMemory    *int         `json:"target_memory,omitempty"`   // 0-100
	TargetRequests  *int         `json:"target_requests,omitempty"` // requests per second
	ScaleUpPolicy   *ScalePolicy `json:"scale_up_policy,omitempty"`
	ScaleDownPolicy *ScalePolicy `json:"scale_down_policy,omitempty"`
}

// ScalePolicy defines scaling behavior
type ScalePolicy struct {
	Cooldown  time.Duration `json:"cooldown"`
	Step      int           `json:"step"`      // Number of instances to add/remove
	Threshold int           `json:"threshold"` // Percentage threshold
}

// BackupConfig represents backup configuration
type BackupConfig struct {
	Enabled             bool   `json:"enabled"`
	Schedule            string `json:"schedule,omitempty"` // Cron expression
	RetentionDays       int    `json:"retention_days"`
	BackupWindow        string `json:"backup_window,omitempty"`
	PointInTimeRecovery bool   `json:"point_in_time_recovery"`
}

// Backup represents a backup instance
type Backup struct {
	ID          string     `json:"id"`
	ResourceID  string     `json:"resource_id"`
	Type        string     `json:"type"` // full, incremental, snapshot
	Status      string     `json:"status"`
	SizeBytes   int64      `json:"size_bytes"`
	StartedAt   time.Time  `json:"started_at"`
	CompletedAt *time.Time `json:"completed_at,omitempty"`
}

// parseRFC3339OrNow parses an RFC3339 timestamp string, returning time.Now() on failure.
func parseRFC3339OrNow(s string) time.Time {
	if s == "" {
		return time.Now()
	}
	t, err := time.Parse(time.RFC3339, s)
	if err != nil {
		t, err = time.Parse(time.RFC3339Nano, s)
		if err != nil {
			return time.Now()
		}
	}
	return t
}
