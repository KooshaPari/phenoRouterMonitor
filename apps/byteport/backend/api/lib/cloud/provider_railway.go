// Package cloud — Railway provider implementation.
// wraps: Railway GraphQL API v2 (https://docs.railway.app/reference/public-api)
package cloud

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"time"
)

const railwayGraphQLEndpoint = "https://backboard.railway.app/graphql/v2"

// RailwayProvider implements CloudProvider for Railway service deployments.
// Credentials.Data must contain "token" (Railway API token).
type RailwayProvider struct {
	credentials Credentials
	metadata    ProviderMetadata
	httpClient  *http.Client
	token       string
}

// compile-time interface assertion
var _ CloudProvider = (*RailwayProvider)(nil)

// NewRailwayProvider constructs a RailwayProvider.
// Required credential key: "token".
func NewRailwayProvider(credentials Credentials) (CloudProvider, error) {
	token := credentials.Data["token"]
	if token == "" {
		return nil, fmt.Errorf("railway: credential key 'token' is required — generate one at https://railway.app/account/tokens")
	}

	return &RailwayProvider{
		credentials: credentials,
		token:       token,
		httpClient:  &http.Client{Timeout: 30 * time.Second},
		metadata: ProviderMetadata{
			Name:    "railway",
			Version: "1.0.0",
			SupportedResources: []ResourceType{
				ResourceTypeComputeContainer,
				ResourceTypeDatabaseServerless,
			},
			Capabilities: []Capability{
				CapabilityLoggable,
				CapabilityScalable,
			},
			Regions: []Region{
				{ID: "us-west2", Name: "US West", Location: "California, USA", Available: true},
				{ID: "us-east4", Name: "US East", Location: "Virginia, USA", Available: true},
				{ID: "europe-west4", Name: "Europe West", Location: "Netherlands", Available: true},
				{ID: "asia-southeast1", Name: "Asia Southeast", Location: "Singapore", Available: true},
			},
			AuthTypes:   []string{"token"},
			Description: "Railway — infrastructure deployment platform",
		},
	}, nil
}

func (p *RailwayProvider) GetMetadata() ProviderMetadata { return p.metadata }
func (p *RailwayProvider) GetCapabilities() []Capability  { return p.metadata.Capabilities }

func (p *RailwayProvider) SupportsResource(resourceType ResourceType) bool {
	for _, t := range p.metadata.SupportedResources {
		if t == resourceType {
			return true
		}
	}
	return false
}

// Initialize re-sets credentials and validates them.
func (p *RailwayProvider) Initialize(ctx context.Context, credentials Credentials) error {
	p.credentials = credentials
	p.token = credentials.Data["token"]
	return p.ValidateCredentials(ctx)
}

// ValidateCredentials queries the Railway me { id } GraphQL field.
func (p *RailwayProvider) ValidateCredentials(ctx context.Context) error {
	if p.token == "" {
		return fmt.Errorf("railway: token is empty — cannot validate credentials")
	}
	var resp struct {
		Data struct {
			Me struct {
				ID string `json:"id"`
			} `json:"me"`
		} `json:"data"`
		Errors []struct{ Message string } `json:"errors"`
	}
	if err := p.graphQL(ctx, `{ me { id } }`, nil, &resp); err != nil {
		return fmt.Errorf("railway: credential validation failed: %w", err)
	}
	if len(resp.Errors) > 0 {
		return fmt.Errorf("railway: credential validation error: %s", resp.Errors[0].Message)
	}
	if resp.Data.Me.ID == "" {
		return fmt.Errorf("railway: token valid but returned empty user ID")
	}
	return nil
}

// CreateResource creates a Railway project.
// config.Spec keys:
//
//	"description" — project description (optional)
//	"team_id"     — associate with a team (optional)
func (p *RailwayProvider) CreateResource(ctx context.Context, config ResourceConfig) (*Resource, error) {
	if !p.SupportsResource(config.Type) {
		return nil, NewNotSupportedError("railway", string(config.Type))
	}

	vars := map[string]any{
		"input": map[string]any{
			"name": config.Name,
		},
	}
	if desc, ok := config.Spec["description"].(string); ok && desc != "" {
		vars["input"].(map[string]any)["description"] = desc
	}
	if teamID, ok := config.Spec["team_id"].(string); ok && teamID != "" {
		vars["input"].(map[string]any)["teamId"] = teamID
	}

	const mutation = `
mutation ProjectCreate($input: ProjectCreateInput!) {
  projectCreate(input: $input) {
    id
    name
    description
    createdAt
    updatedAt
  }
}`

	var resp struct {
		Data struct {
			ProjectCreate railwayProject `json:"projectCreate"`
		} `json:"data"`
		Errors []struct{ Message string } `json:"errors"`
	}
	if err := p.graphQL(ctx, mutation, vars, &resp); err != nil {
		return nil, fmt.Errorf("railway: create project %q failed: %w", config.Name, err)
	}
	if len(resp.Errors) > 0 {
		return nil, fmt.Errorf("railway: create project %q — GraphQL error: %s", config.Name, resp.Errors[0].Message)
	}
	return railwayProjectToResource(resp.Data.ProjectCreate), nil
}

// GetResource retrieves a Railway project by ID.
func (p *RailwayProvider) GetResource(ctx context.Context, id string) (*Resource, error) {
	const query = `
query Project($id: String!) {
  project(id: $id) {
    id
    name
    description
    createdAt
    updatedAt
  }
}`
	var resp struct {
		Data struct {
			Project railwayProject `json:"project"`
		} `json:"data"`
		Errors []struct{ Message string } `json:"errors"`
	}
	if err := p.graphQL(ctx, query, map[string]any{"id": id}, &resp); err != nil {
		return nil, fmt.Errorf("railway: get project %q failed: %w", id, err)
	}
	if len(resp.Errors) > 0 {
		return nil, fmt.Errorf("railway: get project %q — GraphQL error: %s", id, resp.Errors[0].Message)
	}
	return railwayProjectToResource(resp.Data.Project), nil
}

// UpdateResource is not currently implemented.
func (p *RailwayProvider) UpdateResource(ctx context.Context, id string, config ResourceConfig) (*Resource, error) {
	return nil, NewNotSupportedError("railway", "UpdateResource")
}

// DeleteResource deletes a Railway project.
func (p *RailwayProvider) DeleteResource(ctx context.Context, id string) error {
	const mutation = `
mutation ProjectDelete($id: String!) {
  projectDelete(id: $id)
}`
	var resp struct {
		Errors []struct{ Message string } `json:"errors"`
	}
	if err := p.graphQL(ctx, mutation, map[string]any{"id": id}, &resp); err != nil {
		return fmt.Errorf("railway: delete project %q failed: %w", id, err)
	}
	if len(resp.Errors) > 0 {
		return fmt.Errorf("railway: delete project %q — GraphQL error: %s", id, resp.Errors[0].Message)
	}
	return nil
}

// ListResources lists all Railway projects for the authenticated user.
func (p *RailwayProvider) ListResources(ctx context.Context, filter ResourceFilter) ([]*Resource, error) {
	const query = `
{
  me {
    projects {
      edges {
        node {
          id
          name
          description
          createdAt
          updatedAt
        }
      }
    }
  }
}`
	var resp struct {
		Data struct {
			Me struct {
				Projects struct {
					Edges []struct {
						Node railwayProject `json:"node"`
					} `json:"edges"`
				} `json:"projects"`
			} `json:"me"`
		} `json:"data"`
		Errors []struct{ Message string } `json:"errors"`
	}
	if err := p.graphQL(ctx, query, nil, &resp); err != nil {
		return nil, fmt.Errorf("railway: list projects failed: %w", err)
	}
	if len(resp.Errors) > 0 {
		return nil, fmt.Errorf("railway: list projects — GraphQL error: %s", resp.Errors[0].Message)
	}
	resources := make([]*Resource, 0, len(resp.Data.Me.Projects.Edges))
	for _, edge := range resp.Data.Me.Projects.Edges {
		edge := edge
		resources = append(resources, railwayProjectToResource(edge.Node))
	}
	return resources, nil
}

// Deploy triggers a Railway service deployment.
// config.ResourceID must be the Railway service ID.
// config.Config keys:
//
//	"environment_id" — target environment ID (required for Railway deployments)
func (p *RailwayProvider) Deploy(ctx context.Context, config DeploymentConfig) (*Deployment, error) {
	if config.ResourceID == "" {
		return nil, fmt.Errorf("railway: Deploy requires ResourceID (service ID)")
	}

	environmentID, _ := config.Config["environment_id"].(string)
	if environmentID == "" {
		return nil, fmt.Errorf("railway: Deploy requires config.Config['environment_id'] — find it in the Railway dashboard")
	}

	vars := map[string]any{
		"serviceId":     config.ResourceID,
		"environmentId": environmentID,
	}
	if config.Source != nil && config.Source.Commit != "" {
		vars["commitHash"] = config.Source.Commit
	}

	const mutation = `
mutation ServiceInstanceDeploy($serviceId: String!, $environmentId: String!) {
  serviceInstanceDeploy(serviceId: $serviceId, environmentId: $environmentId)
}`

	var resp struct {
		Data   struct{ ServiceInstanceDeploy bool } `json:"data"`
		Errors []struct{ Message string }           `json:"errors"`
	}
	if err := p.graphQL(ctx, mutation, vars, &resp); err != nil {
		return nil, fmt.Errorf("railway: deploy service %q failed: %w", config.ResourceID, err)
	}
	if len(resp.Errors) > 0 {
		return nil, fmt.Errorf("railway: deploy service %q — GraphQL error: %s", config.ResourceID, resp.Errors[0].Message)
	}

	return &Deployment{
		ID:        config.ResourceID + "-" + environmentID,
		ResourceID: config.ResourceID,
		State:     DeploymentStateDeploying,
		StartedAt: time.Now(),
		UpdatedAt: time.Now(),
	}, nil
}

// GetDeploymentStatus fetches the latest deployment for a service.
// id format: "<service_id>-<environment_id>"
func (p *RailwayProvider) GetDeploymentStatus(ctx context.Context, id string) (*DeploymentStatus, error) {
	return nil, NewNotSupportedError("railway", "GetDeploymentStatus — query serviceInstance via GraphQL directly")
}

// RollbackDeployment is not currently supported.
func (p *RailwayProvider) RollbackDeployment(ctx context.Context, id string) error {
	return NewNotSupportedError("railway", "RollbackDeployment")
}

// GetLogs is not available in this implementation.
func (p *RailwayProvider) GetLogs(ctx context.Context, resource *Resource, opts LogOptions) (LogStream, error) {
	return nil, NewNotSupportedError("railway", "GetLogs")
}

// GetMetrics is not available via Railway's public API.
func (p *RailwayProvider) GetMetrics(ctx context.Context, resource *Resource, opts MetricOptions) ([]Metric, error) {
	return nil, NewNotSupportedError("railway", "GetMetrics")
}

// EstimateCost returns zero-cost (Railway hobby plan assumed).
func (p *RailwayProvider) EstimateCost(ctx context.Context, config ResourceConfig) (*CostEstimate, error) {
	return &CostEstimate{
		HourlyUSD:   0,
		DailyUSD:    0,
		MonthlyUSD:  0,
		Breakdown:   map[string]float64{"compute": 0},
		Confidence:  "medium",
		Currency:    "USD",
		LastUpdated: time.Now(),
	}, nil
}

// GetActualCost is not available via Railway's public API.
func (p *RailwayProvider) GetActualCost(ctx context.Context, resource *Resource, timeRange TimeRange) (*Cost, error) {
	return nil, NewNotSupportedError("railway", "GetActualCost")
}

// ---------------------------------------------------------------------------
// Internal Railway types
// ---------------------------------------------------------------------------

type railwayProject struct {
	ID          string `json:"id"`
	Name        string `json:"name"`
	Description string `json:"description"`
	CreatedAt   string `json:"createdAt"`
	UpdatedAt   string `json:"updatedAt"`
}

func railwayProjectToResource(p railwayProject) *Resource {
	return &Resource{
		ID:           p.ID,
		Name:         p.Name,
		Type:         ResourceTypeComputeContainer,
		Provider:     "railway",
		Status:       DeploymentStateActive,
		HealthStatus: HealthStatusHealthy,
		Endpoints: []Endpoint{
			{Type: "https", URL: fmt.Sprintf("https://%s.up.railway.app", p.Name), Primary: true},
		},
		Metadata:  map[string]any{"description": p.Description},
		CreatedAt: parseRFC3339OrNow(p.CreatedAt),
		UpdatedAt: parseRFC3339OrNow(p.UpdatedAt),
	}
}

// ---------------------------------------------------------------------------
// GraphQL helper
// ---------------------------------------------------------------------------

type graphQLRequest struct {
	Query     string         `json:"query"`
	Variables map[string]any `json:"variables,omitempty"`
}

func (p *RailwayProvider) graphQL(ctx context.Context, query string, variables map[string]any, dest any) error {
	payload := graphQLRequest{Query: query, Variables: variables}
	body, err := json.Marshal(payload)
	if err != nil {
		return fmt.Errorf("railway: marshal GraphQL request: %w", err)
	}

	req, err := http.NewRequestWithContext(ctx, http.MethodPost, railwayGraphQLEndpoint, bytes.NewReader(body))
	if err != nil {
		return fmt.Errorf("railway: build request: %w", err)
	}
	req.Header.Set("Authorization", "Bearer "+p.token)
	req.Header.Set("Content-Type", "application/json")
	req.Header.Set("User-Agent", "BytePort/1.0")

	resp, err := p.httpClient.Do(req)
	if err != nil {
		return fmt.Errorf("railway: GraphQL HTTP transport error: %w", err)
	}
	defer resp.Body.Close()

	if resp.StatusCode == http.StatusUnauthorized {
		return fmt.Errorf("railway: 401 Unauthorized — check your API token at https://railway.app/account/tokens")
	}
	if resp.StatusCode < 200 || resp.StatusCode >= 300 {
		raw, _ := io.ReadAll(resp.Body)
		return fmt.Errorf("railway: GraphQL endpoint returned %d: %s", resp.StatusCode, string(raw))
	}

	return json.NewDecoder(resp.Body).Decode(dest)
}

// ---------------------------------------------------------------------------
// Registry init
// ---------------------------------------------------------------------------

func init() {
	MustRegister(
		ProviderMetadata{
			Name:    "railway",
			Version: "1.0.0",
			SupportedResources: []ResourceType{
				ResourceTypeComputeContainer,
				ResourceTypeDatabaseServerless,
			},
			Capabilities: []Capability{
				CapabilityLoggable,
				CapabilityScalable,
			},
			Regions: []Region{
				{ID: "us-west2", Name: "US West", Location: "California, USA", Available: true},
				{ID: "us-east4", Name: "US East", Location: "Virginia, USA", Available: true},
			},
			AuthTypes:   []string{"token"},
			Description: "Railway — infrastructure deployment platform",
		},
		NewRailwayProvider,
	)
}
