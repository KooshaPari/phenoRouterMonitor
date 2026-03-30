package cloud

import (
	"testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

// Test all the enum types and constants

func TestResourceType_Constants(t *testing.T) {
	// Test all compute types
	assert.Equal(t, ResourceType("compute.vm"), ResourceTypeComputeVM)
	assert.Equal(t, ResourceType("compute.container"), ResourceTypeComputeContainer)
	assert.Equal(t, ResourceType("compute.function"), ResourceTypeComputeFunction)
	assert.Equal(t, ResourceType("compute.edge"), ResourceTypeComputeEdge)

	// Test all database types
	assert.Equal(t, ResourceType("database.sql"), ResourceTypeDatabaseSQL)
	assert.Equal(t, ResourceType("database.nosql"), ResourceTypeDatabaseNoSQL)
	assert.Equal(t, ResourceType("database.serverless"), ResourceTypeDatabaseServerless)

	// Test all storage types
	assert.Equal(t, ResourceType("storage.object"), ResourceTypeStorageObject)
	assert.Equal(t, ResourceType("storage.block"), ResourceTypeStorageBlock)
	assert.Equal(t, ResourceType("storage.file"), ResourceTypeStorageFile)

	// Test all network types
	assert.Equal(t, ResourceType("network.loadbalancer"), ResourceTypeNetworkLoadBalancer)
	assert.Equal(t, ResourceType("network.cdn"), ResourceTypeNetworkCDN)
	assert.Equal(t, ResourceType("network.dns"), ResourceTypeNetworkDNS)
	assert.Equal(t, ResourceType("network.vpc"), ResourceTypeNetworkVPC)
}

func TestDeploymentState_Constants(t *testing.T) {
	states := []DeploymentState{
		DeploymentStatePending,
		DeploymentStateValidating,
		DeploymentStateBuilding,
		DeploymentStateProvisioning,
		DeploymentStateDeploying,
		DeploymentStateHealthCheck,
		DeploymentStateActive,
		DeploymentStateUpdating,
		DeploymentStateScaling,
		DeploymentStateDegraded,
		DeploymentStateFailed,
		DeploymentStateRollingBack,
		DeploymentStateDeleting,
		DeploymentStateDeleted,
	}

	expectedValues := []string{
		"PENDING", "VALIDATING", "BUILDING", "PROVISIONING",
		"DEPLOYING", "HEALTH_CHECK", "ACTIVE", "UPDATING",
		"SCALING", "DEGRADED", "FAILED", "ROLLING_BACK",
		"DELETING", "DELETED",
	}

	require.Len(t, states, len(expectedValues))
	for i, state := range states {
		assert.Equal(t, expectedValues[i], string(state))
	}
}

func TestDeploymentStrategy_Constants(t *testing.T) {
	strategies := []DeploymentStrategy{
		DeploymentStrategyRolling,
		DeploymentStrategyBlueGreen,
		DeploymentStrategyCanary,
		DeploymentStrategyAtomic,
		DeploymentStrategyRecreate,
	}

	expectedValues := []string{
		"rolling", "bluegreen", "canary", "atomic", "recreate",
	}

	require.Len(t, strategies, len(expectedValues))
	for i, strategy := range strategies {
		assert.Equal(t, expectedValues[i], string(strategy))
	}
}

func TestCapability_Constants(t *testing.T) {
	capabilities := []Capability{
		CapabilityScalable,
		CapabilityLoggable,
		CapabilityExecutable,
		CapabilityBackupable,
		CapabilityMonitoring,
		CapabilityAutoScale,
		CapabilityCustomDNS,
		CapabilitySSH,
	}

	expectedValues := []string{
		"scalable", "loggable", "executable", "backupable",
		"monitoring", "autoscale", "custom_dns", "ssh",
	}

	require.Len(t, capabilities, len(expectedValues))
	for i, capability := range capabilities {
		assert.Equal(t, expectedValues[i], string(capability))
	}
}

func TestHealthStatus_Constants(t *testing.T) {
	statuses := []HealthStatus{
		HealthStatusUnknown,
		HealthStatusHealthy,
		HealthStatusDegraded,
		HealthStatusUnhealthy,
		HealthStatusChecking,
	}

	expectedValues := []string{
		"UNKNOWN", "HEALTHY", "DEGRADED", "UNHEALTHY", "CHECKING",
	}

	require.Len(t, statuses, len(expectedValues))
	for i, status := range statuses {
		assert.Equal(t, expectedValues[i], string(status))
	}
}

// Test struct initialization and field access

func TestResourceConfig_Creation(t *testing.T) {
	config := ResourceConfig{
		Name:     "test-resource",
		Type:     ResourceTypeComputeContainer,
		Provider: "aws",
		Region:   "us-east-1",
		Tags: map[string]string{
			"environment": "test",
			"team":        "platform",
		},
		Spec: map[string]any{
			"memory": 512,
			"cpu":    1.0,
		},
	}

	assert.Equal(t, "test-resource", config.Name)
	assert.Equal(t, ResourceTypeComputeContainer, config.Type)
	assert.Equal(t, "aws", config.Provider)
	assert.Equal(t, "us-east-1", config.Region)
	assert.Equal(t, "test", config.Tags["environment"])
	assert.Equal(t, "platform", config.Tags["team"])
	assert.Equal(t, 512, config.Spec["memory"])
	assert.Equal(t, 1.0, config.Spec["cpu"])
}

func TestResource_Creation(t *testing.T) {
	now := time.Now()
	deployedAt := now.Add(-time.Hour)

	resource := Resource{
		ID:             "resource-123",
		Name:           "test-app",
		Type:           ResourceTypeComputeFunction,
		Provider:       "vercel",
		Region:         "us-east-1",
		Status:         DeploymentStateActive,
		HealthStatus:   HealthStatusHealthy,
		Tags:           map[string]string{"env": "prod"},
		Endpoints: []Endpoint{
			{
				Type:    "https",
				URL:     "https://test-app.vercel.app",
				Primary: true,
			},
		},
		Metadata: map[string]any{
			"build_id": "abc123",
			"commit":   "def456",
		},
		CreatedAt:      now.Add(-2 * time.Hour),
		UpdatedAt:      now.Add(-30 * time.Minute),
		LastDeployedAt: &deployedAt,
	}

	assert.Equal(t, "resource-123", resource.ID)
	assert.Equal(t, "test-app", resource.Name)
	assert.Equal(t, ResourceTypeComputeFunction, resource.Type)
	assert.Equal(t, "vercel", resource.Provider)
	assert.Equal(t, DeploymentStateActive, resource.Status)
	assert.Equal(t, HealthStatusHealthy, resource.HealthStatus)
	assert.Len(t, resource.Endpoints, 1)
	assert.True(t, resource.Endpoints[0].Primary)
	assert.Equal(t, "abc123", resource.Metadata["build_id"])
	assert.NotNil(t, resource.LastDeployedAt)
}

func TestDeploymentConfig_Creation(t *testing.T) {
	config := DeploymentConfig{
		ResourceID: "resource-456",
		Version:    "v1.2.3",
		Source: &DeploymentSource{
			Type:       "git",
			Repository: "https://github.com/user/repo.git",
			Branch:     "main",
			Commit:     "abc123def",
		},
		Env: map[string]string{
			"NODE_ENV": "production",
			"PORT":     "3000",
		},
		Secrets: map[string]string{
			"DATABASE_URL": "postgres://...",
			"API_KEY":      "secret123",
		},
		Strategy: DeploymentStrategyRolling,
		Config: map[string]any{
			"timeout":   300,
			"instances": 3,
		},
	}

	assert.Equal(t, "resource-456", config.ResourceID)
	assert.Equal(t, "v1.2.3", config.Version)
	assert.Equal(t, DeploymentStrategyRolling, config.Strategy)
	require.NotNil(t, config.Source)
	assert.Equal(t, "git", config.Source.Type)
	assert.Equal(t, "https://github.com/user/repo.git", config.Source.Repository)
	assert.Equal(t, "main", config.Source.Branch)
	assert.Equal(t, "production", config.Env["NODE_ENV"])
	assert.Equal(t, "secret123", config.Secrets["API_KEY"])
	assert.Equal(t, 300, config.Config["timeout"])
}

func TestDeployment_Creation(t *testing.T) {
	startedAt := time.Now().Add(-time.Hour)
	updatedAt := time.Now().Add(-30 * time.Minute)
	finishedAt := time.Now().Add(-15 * time.Minute)

	deployment := Deployment{
		ID:         "deploy-789",
		ResourceID: "resource-456",
		Version:    "v1.2.4",
		State:      DeploymentStateActive,
		Strategy:   DeploymentStrategyBlueGreen,
		Progress:   100,
		Message:    "Deployment completed successfully",
		StartedAt:  startedAt,
		UpdatedAt:  updatedAt,
		FinishedAt: &finishedAt,
		Error:      nil,
	}

	assert.Equal(t, "deploy-789", deployment.ID)
	assert.Equal(t, "resource-456", deployment.ResourceID)
	assert.Equal(t, DeploymentStateActive, deployment.State)
	assert.Equal(t, DeploymentStrategyBlueGreen, deployment.Strategy)
	assert.Equal(t, 100, deployment.Progress)
	assert.Equal(t, startedAt, deployment.StartedAt)
	assert.NotNil(t, deployment.FinishedAt)
	assert.Equal(t, finishedAt, *deployment.FinishedAt)
	assert.Nil(t, deployment.Error)
}

func TestDeployment_WithError(t *testing.T) {
	deployment := Deployment{
		ID:         "deploy-failed",
		ResourceID: "resource-456",
		Version:    "v1.2.5",
		State:      DeploymentStateFailed,
		Strategy:   DeploymentStrategyAtomic,
		Progress:   45,
		Message:    "Deployment failed during health check",
		StartedAt:  time.Now().Add(-time.Hour),
		UpdatedAt:  time.Now().Add(-30 * time.Minute),
		FinishedAt: nil,
		Error: &DeploymentError{
			Code:    "HEALTH_CHECK_FAILED",
			Message: "Service failed to respond within timeout",
			Details: map[string]any{
				"timeout":  "30s",
				"endpoint": "/health",
				"retries":  3,
			},
		},
	}

	assert.Equal(t, DeploymentStateFailed, deployment.State)
	assert.Equal(t, 45, deployment.Progress)
	assert.Nil(t, deployment.FinishedAt)
	require.NotNil(t, deployment.Error)
	assert.Equal(t, "HEALTH_CHECK_FAILED", deployment.Error.Code)
	assert.Equal(t, "Service failed to respond within timeout", deployment.Error.Message)
	
	details := deployment.Error.Details.(map[string]any)
	assert.Equal(t, "30s", details["timeout"])
	assert.Equal(t, "/health", details["endpoint"])
	assert.Equal(t, 3, details["retries"])
}

func TestProviderMetadata_Creation(t *testing.T) {
	metadata := ProviderMetadata{
		Name:    "aws-provider",
		Version: "2.1.0",
		SupportedResources: []ResourceType{
			ResourceTypeComputeVM,
			ResourceTypeComputeContainer,
			ResourceTypeDatabaseSQL,
			ResourceTypeStorageObject,
		},
		Capabilities: []Capability{
			CapabilityScalable,
			CapabilityLoggable,
			CapabilityBackupable,
			CapabilityMonitoring,
		},
		Regions: []Region{
			{
				ID:         "us-east-1",
				Name:       "N. Virginia",
				Location:   "US East (N. Virginia)",
				Available:  true,
				Deprecated: false,
				Zones:      []string{"us-east-1a", "us-east-1b", "us-east-1c"},
			},
			{
				ID:         "eu-west-1",
				Name:       "Ireland",
				Location:   "Europe (Ireland)",
				Available:  true,
				Deprecated: false,
				Zones:      []string{"eu-west-1a", "eu-west-1b", "eu-west-1c"},
			},
		},
		AuthTypes:   []string{"iam", "api_key"},
		Description: "Amazon Web Services cloud provider",
	}

	assert.Equal(t, "aws-provider", metadata.Name)
	assert.Equal(t, "2.1.0", metadata.Version)
	assert.Len(t, metadata.SupportedResources, 4)
	assert.Contains(t, metadata.SupportedResources, ResourceTypeComputeVM)
	assert.Contains(t, metadata.SupportedResources, ResourceTypeDatabaseSQL)
	assert.Len(t, metadata.Capabilities, 4)
	assert.Contains(t, metadata.Capabilities, CapabilityScalable)
	assert.Contains(t, metadata.Capabilities, CapabilityMonitoring)
	assert.Len(t, metadata.Regions, 2)
	assert.Equal(t, "us-east-1", metadata.Regions[0].ID)
	assert.True(t, metadata.Regions[0].Available)
	assert.False(t, metadata.Regions[0].Deprecated)
	assert.Len(t, metadata.Regions[0].Zones, 3)
	assert.Contains(t, metadata.AuthTypes, "iam")
	assert.Contains(t, metadata.AuthTypes, "api_key")
}

func TestCredentials_Creation(t *testing.T) {
	creds := Credentials{
		Type: "service_account",
		Data: map[string]string{
			"client_id":     "123456789",
			"client_secret": "abcdef123456",
			"account_id":    "acc-789",
		},
		Region:   "us-west-2",
		Endpoint: "https://api.custom-cloud.com/v1",
	}

	assert.Equal(t, "service_account", creds.Type)
	assert.Equal(t, "123456789", creds.Data["client_id"])
	assert.Equal(t, "abcdef123456", creds.Data["client_secret"])
	assert.Equal(t, "acc-789", creds.Data["account_id"])
	assert.Equal(t, "us-west-2", creds.Region)
	assert.Equal(t, "https://api.custom-cloud.com/v1", creds.Endpoint)
}

func TestHealthCheckConfig_Creation(t *testing.T) {
	config := HealthCheckConfig{
		Type:             "http",
		Path:             "/health",
		Port:             8080,
		Command:          "",
		Interval:         30 * time.Second,
		Timeout:          5 * time.Second,
		Retries:          3,
		InitialDelay:     10 * time.Second,
		SuccessThreshold: 1,
		FailureThreshold: 3,
	}

	assert.Equal(t, "http", config.Type)
	assert.Equal(t, "/health", config.Path)
	assert.Equal(t, 8080, config.Port)
	assert.Equal(t, 30*time.Second, config.Interval)
	assert.Equal(t, 5*time.Second, config.Timeout)
	assert.Equal(t, 3, config.Retries)
	assert.Equal(t, 10*time.Second, config.InitialDelay)
	assert.Equal(t, 1, config.SuccessThreshold)
	assert.Equal(t, 3, config.FailureThreshold)
}

func TestScaleConfig_Creation(t *testing.T) {
	targetCPU := 70
	targetMemory := 80
	targetRequests := 100

	config := ScaleConfig{
		MinInstances:   1,
		MaxInstances:   10,
		TargetCPU:      &targetCPU,
		TargetMemory:   &targetMemory,
		TargetRequests: &targetRequests,
		ScaleUpPolicy: &ScalePolicy{
			Cooldown:  5 * time.Minute,
			Step:      2,
			Threshold: 80,
		},
		ScaleDownPolicy: &ScalePolicy{
			Cooldown:  10 * time.Minute,
			Step:      1,
			Threshold: 30,
		},
	}

	assert.Equal(t, 1, config.MinInstances)
	assert.Equal(t, 10, config.MaxInstances)
	require.NotNil(t, config.TargetCPU)
	assert.Equal(t, 70, *config.TargetCPU)
	require.NotNil(t, config.TargetMemory)
	assert.Equal(t, 80, *config.TargetMemory)
	require.NotNil(t, config.TargetRequests)
	assert.Equal(t, 100, *config.TargetRequests)

	require.NotNil(t, config.ScaleUpPolicy)
	assert.Equal(t, 5*time.Minute, config.ScaleUpPolicy.Cooldown)
	assert.Equal(t, 2, config.ScaleUpPolicy.Step)
	assert.Equal(t, 80, config.ScaleUpPolicy.Threshold)

	require.NotNil(t, config.ScaleDownPolicy)
	assert.Equal(t, 10*time.Minute, config.ScaleDownPolicy.Cooldown)
	assert.Equal(t, 1, config.ScaleDownPolicy.Step)
	assert.Equal(t, 30, config.ScaleDownPolicy.Threshold)
}

func TestCostEstimate_Creation(t *testing.T) {
	lastUpdated := time.Now().Add(-time.Hour)

	estimate := CostEstimate{
		HourlyUSD:  0.12,
		DailyUSD:   2.88,
		MonthlyUSD: 87.60,
		Breakdown: map[string]float64{
			"compute": 60.00,
			"storage": 15.60,
			"network": 12.00,
		},
		Confidence:  "high",
		Currency:    "USD",
		LastUpdated: lastUpdated,
	}

	assert.Equal(t, 0.12, estimate.HourlyUSD)
	assert.Equal(t, 2.88, estimate.DailyUSD)
	assert.Equal(t, 87.60, estimate.MonthlyUSD)
	assert.Equal(t, 60.00, estimate.Breakdown["compute"])
	assert.Equal(t, 15.60, estimate.Breakdown["storage"])
	assert.Equal(t, 12.00, estimate.Breakdown["network"])
	assert.Equal(t, "high", estimate.Confidence)
	assert.Equal(t, "USD", estimate.Currency)
	assert.Equal(t, lastUpdated, estimate.LastUpdated)
}

func TestProjectConfig_Creation(t *testing.T) {
	config := ProjectConfig{
		Name:        "my-web-app",
		Version:     "1.0.0",
		Environment: "production",
		Region:      "us-east-1",
		Tags: map[string]string{
			"team":    "backend",
			"project": "ecommerce",
		},
		Providers: map[string]Credentials{
			"aws": {
				Type: "iam",
				Data: map[string]string{
					"access_key": "AKIA...",
					"secret_key": "secret...",
				},
				Region: "us-east-1",
			},
			"vercel": {
				Type: "token",
				Data: map[string]string{
					"token": "vercel_token...",
				},
			},
		},
		Resources: []ResourceConfig{
			{
				Name:     "frontend",
				Type:     ResourceTypeComputeFunction,
				Provider: "vercel",
			},
			{
				Name:     "database",
				Type:     ResourceTypeDatabaseSQL,
				Provider: "aws",
				Region:   "us-east-1",
			},
		},
		Dependencies: []ResourceDependency{
			{
				Resource:  "frontend",
				DependsOn: []string{"database"},
				WaitFor:   "ACTIVE",
			},
		},
	}

	assert.Equal(t, "my-web-app", config.Name)
	assert.Equal(t, "1.0.0", config.Version)
	assert.Equal(t, "production", config.Environment)
	assert.Equal(t, "backend", config.Tags["team"])
	assert.Len(t, config.Providers, 2)
	assert.Equal(t, "iam", config.Providers["aws"].Type)
	assert.Equal(t, "token", config.Providers["vercel"].Type)
	assert.Len(t, config.Resources, 2)
	assert.Equal(t, "frontend", config.Resources[0].Name)
	assert.Equal(t, ResourceTypeComputeFunction, config.Resources[0].Type)
	assert.Len(t, config.Dependencies, 1)
	assert.Equal(t, "frontend", config.Dependencies[0].Resource)
	assert.Contains(t, config.Dependencies[0].DependsOn, "database")
	assert.Equal(t, "ACTIVE", config.Dependencies[0].WaitFor)
}