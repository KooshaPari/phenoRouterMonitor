// Package cloud — Vercel provider implementation.
// wraps: Vercel REST API v13 (https://vercel.com/docs/rest-api)
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

const vercelAPIBase = "https://api.vercel.com"

// VercelProvider implements CloudProvider for Vercel deployments.
// Credentials.Data must contain "token" (personal access token or team token).
// Optionally set "team_id" for team-scoped operations.
type VercelProvider struct {
	credentials Credentials
	metadata    ProviderMetadata
	httpClient  *http.Client
	token       string
	teamID      string
}

// compile-time interface assertion
var _ CloudProvider = (*VercelProvider)(nil)

// NewVercelProvider constructs a VercelProvider from the supplied credentials.
// Required credential keys: "token".
// Optional credential keys: "team_id".
func NewVercelProvider(credentials Credentials) (CloudProvider, error) {
	token := credentials.Data["token"]
	if token == "" {
		return nil, fmt.Errorf("vercel: credential key 'token' is required — generate one at https://vercel.com/account/tokens")
	}

	return &VercelProvider{
		credentials: credentials,
		token:       token,
		teamID:      credentials.Data["team_id"],
		httpClient:  &http.Client{Timeout: 30 * time.Second},
		metadata: ProviderMetadata{
			Name:    "vercel",
			Version: "1.0.0",
			SupportedResources: []ResourceType{
				ResourceTypeComputeEdge,
				ResourceTypeComputeFunction,
			},
			Capabilities: []Capability{
				CapabilityLoggable,
				CapabilityCustomDNS,
			},
			Regions: []Region{
				{ID: "iad1", Name: "Washington, D.C., USA", Location: "US East", Available: true},
				{ID: "sfo1", Name: "San Francisco, CA, USA", Location: "US West", Available: true},
				{ID: "cdg1", Name: "Paris, France", Location: "Europe West", Available: true},
				{ID: "hnd1", Name: "Tokyo, Japan", Location: "Asia Pacific", Available: true},
			},
			AuthTypes:   []string{"token"},
			Description: "Vercel — frontend/serverless deployment platform",
		},
	}, nil
}

// GetMetadata returns provider metadata.
func (p *VercelProvider) GetMetadata() ProviderMetadata { return p.metadata }

// SupportsResource returns true for edge and function resource types.
func (p *VercelProvider) SupportsResource(resourceType ResourceType) bool {
	for _, t := range p.metadata.SupportedResources {
		if t == resourceType {
			return true
		}
	}
	return false
}

// GetCapabilities returns supported capability list.
func (p *VercelProvider) GetCapabilities() []Capability { return p.metadata.Capabilities }

// Initialize validates the token by calling the Vercel /v2/user endpoint.
func (p *VercelProvider) Initialize(ctx context.Context, credentials Credentials) error {
	p.credentials = credentials
	p.token = credentials.Data["token"]
	p.teamID = credentials.Data["team_id"]
	return p.ValidateCredentials(ctx)
}

// ValidateCredentials calls GET /v2/user to confirm the token is valid.
func (p *VercelProvider) ValidateCredentials(ctx context.Context) error {
	if p.token == "" {
		return fmt.Errorf("vercel: token is empty — cannot validate credentials")
	}
	var user struct {
		ID string `json:"id"`
	}
	if err := p.apiGET(ctx, "/v2/user", &user); err != nil {
		return fmt.Errorf("vercel: credential validation failed: %w", err)
	}
	if user.ID == "" {
		return fmt.Errorf("vercel: token appears valid but returned empty user ID")
	}
	return nil
}

// CreateResource creates a Vercel project (ResourceTypeComputeEdge / ResourceTypeComputeFunction).
// config.Spec keys:
//
//	"git_repository"  — HTTPS clone URL (optional, creates blank project if omitted)
//	"framework"       — framework preset: "nextjs", "gatsby", "create-react-app", etc.
//	"build_command"   — override build command
//	"output_directory"— override output directory
//	"root_directory"  — root inside monorepo
func (p *VercelProvider) CreateResource(ctx context.Context, config ResourceConfig) (*Resource, error) {
	if !p.SupportsResource(config.Type) {
		return nil, NewNotSupportedError("vercel", string(config.Type))
	}

	type gitRepo struct {
		Type string `json:"type"`
		Repo string `json:"repo"`
	}
	body := map[string]any{
		"name": config.Name,
	}
	if fw, ok := config.Spec["framework"].(string); ok && fw != "" {
		body["framework"] = fw
	}
	if gitURL, ok := config.Spec["git_repository"].(string); ok && gitURL != "" {
		body["gitRepository"] = gitRepo{Type: "github", Repo: gitURL}
	}
	if bc, ok := config.Spec["build_command"].(string); ok && bc != "" {
		body["buildCommand"] = bc
	}
	if od, ok := config.Spec["output_directory"].(string); ok && od != "" {
		body["outputDirectory"] = od
	}
	if rd, ok := config.Spec["root_directory"].(string); ok && rd != "" {
		body["rootDirectory"] = rd
	}

	var project vercelProject
	if err := p.apiPOST(ctx, "/v9/projects", body, &project); err != nil {
		return nil, fmt.Errorf("vercel: create project %q failed: %w", config.Name, err)
	}

	return vercelProjectToResource(project), nil
}

// GetResource retrieves a Vercel project by name or ID.
func (p *VercelProvider) GetResource(ctx context.Context, id string) (*Resource, error) {
	var project vercelProject
	if err := p.apiGET(ctx, "/v9/projects/"+id, &project); err != nil {
		return nil, fmt.Errorf("vercel: get project %q failed: %w", id, err)
	}
	return vercelProjectToResource(project), nil
}

// UpdateResource is not currently supported via project-level updates in this implementation.
func (p *VercelProvider) UpdateResource(ctx context.Context, id string, config ResourceConfig) (*Resource, error) {
	return nil, NewNotSupportedError("vercel", "UpdateResource")
}

// DeleteResource removes a Vercel project entirely.
func (p *VercelProvider) DeleteResource(ctx context.Context, id string) error {
	if err := p.apiDELETE(ctx, "/v9/projects/"+id); err != nil {
		return fmt.Errorf("vercel: delete project %q failed: %w", id, err)
	}
	return nil
}

// ListResources returns all Vercel projects for the authenticated user/team.
func (p *VercelProvider) ListResources(ctx context.Context, filter ResourceFilter) ([]*Resource, error) {
	var resp struct {
		Projects []vercelProject `json:"projects"`
	}
	if err := p.apiGET(ctx, "/v9/projects", &resp); err != nil {
		return nil, fmt.Errorf("vercel: list projects failed: %w", err)
	}
	resources := make([]*Resource, 0, len(resp.Projects))
	for _, proj := range resp.Projects {
		proj := proj
		resources = append(resources, vercelProjectToResource(proj))
	}
	return resources, nil
}

// Deploy triggers a new deployment for a Vercel project.
// config.Config keys:
//
//	"target"  — "production" (default) or "preview"
//	"git_sha" — specific commit SHA to deploy
func (p *VercelProvider) Deploy(ctx context.Context, config DeploymentConfig) (*Deployment, error) {
	target := "production"
	if t, ok := config.Config["target"].(string); ok && t != "" {
		target = t
	}

	type deployFile struct {
		File string `json:"file"`
		Data string `json:"data"`
	}
	body := map[string]any{
		"name":   config.ResourceID,
		"target": target,
	}
	if config.Source != nil {
		if config.Source.Repository != "" {
			body["gitSource"] = map[string]string{
				"type":   "github",
				"repoID": config.Source.Repository,
				"ref":    config.Source.Branch,
			}
		}
		if config.Source.Commit != "" {
			if gs, ok := body["gitSource"].(map[string]string); ok {
				gs["sha"] = config.Source.Commit
			}
		}
	}
	if len(config.Env) > 0 {
		envList := make([]map[string]string, 0, len(config.Env))
		for k, v := range config.Env {
			envList = append(envList, map[string]string{"key": k, "value": v, "type": "plain"})
		}
		body["env"] = envList
	}

	var dep vercelDeployment
	if err := p.apiPOST(ctx, "/v13/deployments", body, &dep); err != nil {
		return nil, fmt.Errorf("vercel: create deployment for %q failed: %w", config.ResourceID, err)
	}

	return vercelDeploymentToCloudDep(dep), nil
}

// GetDeploymentStatus fetches the current state of a Vercel deployment.
func (p *VercelProvider) GetDeploymentStatus(ctx context.Context, id string) (*DeploymentStatus, error) {
	var dep vercelDeployment
	if err := p.apiGET(ctx, "/v13/deployments/"+id, &dep); err != nil {
		return nil, fmt.Errorf("vercel: get deployment %q failed: %w", id, err)
	}

	cloudDep := vercelDeploymentToCloudDep(dep)
	return &DeploymentStatus{
		Deployment: cloudDep,
		Health:     vercelStateToHealth(dep.ReadyState),
	}, nil
}

// RollbackDeployment is not directly supported by the Vercel API (re-deploy instead).
func (p *VercelProvider) RollbackDeployment(ctx context.Context, id string) error {
	return NewNotSupportedError("vercel", "RollbackDeployment — redeploy a previous deployment ID via Deploy()")
}

// GetLogs retrieves build/runtime logs for a deployment.
func (p *VercelProvider) GetLogs(ctx context.Context, resource *Resource, opts LogOptions) (LogStream, error) {
	// Vercel logs endpoint returns NDJSON.
	url := fmt.Sprintf("/v2/deployments/%s/events", resource.ID)
	req, err := p.newRequest(ctx, http.MethodGet, url, nil)
	if err != nil {
		return nil, fmt.Errorf("vercel: build log request failed: %w", err)
	}
	resp, err := p.httpClient.Do(req)
	if err != nil {
		return nil, fmt.Errorf("vercel: log fetch failed: %w", err)
	}
	if resp.StatusCode != http.StatusOK {
		resp.Body.Close()
		return nil, fmt.Errorf("vercel: log endpoint returned %d", resp.StatusCode)
	}
	return &vercelLogStream{decoder: json.NewDecoder(resp.Body), closer: resp.Body}, nil
}

// GetMetrics is not supported by Vercel's management API.
func (p *VercelProvider) GetMetrics(ctx context.Context, resource *Resource, opts MetricOptions) ([]Metric, error) {
	return nil, NewNotSupportedError("vercel", "GetMetrics")
}

// EstimateCost returns zero-cost estimate (Vercel hobby/free tier assumed).
func (p *VercelProvider) EstimateCost(ctx context.Context, config ResourceConfig) (*CostEstimate, error) {
	return &CostEstimate{
		HourlyUSD:   0,
		DailyUSD:    0,
		MonthlyUSD:  0,
		Breakdown:   map[string]float64{"compute": 0},
		Confidence:  "high",
		Currency:    "USD",
		LastUpdated: time.Now(),
	}, nil
}

// GetActualCost is not available via the Vercel management API.
func (p *VercelProvider) GetActualCost(ctx context.Context, resource *Resource, timeRange TimeRange) (*Cost, error) {
	return nil, NewNotSupportedError("vercel", "GetActualCost")
}

// ---------------------------------------------------------------------------
// Internal Vercel API types
// ---------------------------------------------------------------------------

type vercelProject struct {
	ID        string    `json:"id"`
	Name      string    `json:"name"`
	Framework string    `json:"framework"`
	CreatedAt int64     `json:"createdAt"`
	UpdatedAt int64     `json:"updatedAt"`
}

type vercelDeployment struct {
	ID         string `json:"id"`
	URL        string `json:"url"`
	Name       string `json:"name"`
	ReadyState string `json:"readyState"` // BUILDING, ERROR, INITIALIZING, QUEUED, READY, CANCELED
	CreatedAt  int64  `json:"createdAt"`
	BuildingAt int64  `json:"buildingAt"`
	Ready      int64  `json:"ready"`
}

func vercelProjectToResource(p vercelProject) *Resource {
	status := DeploymentStateActive
	now := time.Now()
	created := now
	if p.CreatedAt > 0 {
		created = time.UnixMilli(p.CreatedAt)
	}
	updated := now
	if p.UpdatedAt > 0 {
		updated = time.UnixMilli(p.UpdatedAt)
	}
	return &Resource{
		ID:           p.ID,
		Name:         p.Name,
		Type:         ResourceTypeComputeEdge,
		Provider:     "vercel",
		Status:       status,
		HealthStatus: HealthStatusHealthy,
		Endpoints: []Endpoint{
			{Type: "https", URL: fmt.Sprintf("https://%s.vercel.app", p.Name), Primary: true},
		},
		Metadata:  map[string]any{"framework": p.Framework},
		CreatedAt: created,
		UpdatedAt: updated,
	}
}

func vercelDeploymentToCloudDep(d vercelDeployment) *Deployment {
	state := vercelReadyStateToDeploymentState(d.ReadyState)
	started := time.Now()
	if d.CreatedAt > 0 {
		started = time.UnixMilli(d.CreatedAt)
	}
	dep := &Deployment{
		ID:        d.ID,
		State:     state,
		StartedAt: started,
		UpdatedAt: time.Now(),
	}
	if d.URL != "" {
		dep.Message = "https://" + d.URL
	}
	return dep
}

func vercelReadyStateToDeploymentState(s string) DeploymentState {
	switch s {
	case "READY":
		return DeploymentStateActive
	case "ERROR":
		return DeploymentStateFailed
	case "BUILDING":
		return DeploymentStateBuilding
	case "INITIALIZING", "QUEUED":
		return DeploymentStatePending
	case "CANCELED":
		return DeploymentStateDeleted
	default:
		return DeploymentStateDeploying
	}
}

func vercelStateToHealth(s string) HealthStatus {
	switch s {
	case "READY":
		return HealthStatusHealthy
	case "ERROR":
		return HealthStatusUnhealthy
	default:
		return HealthStatusChecking
	}
}

// ---------------------------------------------------------------------------
// HTTP helpers
// ---------------------------------------------------------------------------

func (p *VercelProvider) newRequest(ctx context.Context, method, path string, body any) (*http.Request, error) {
	url := vercelAPIBase + path
	if p.teamID != "" {
		sep := "?"
		for _, c := range path {
			if c == '?' {
				sep = "&"
				break
			}
		}
		url += sep + "teamId=" + p.teamID
	}

	var bodyReader io.Reader
	if body != nil {
		b, err := json.Marshal(body)
		if err != nil {
			return nil, fmt.Errorf("vercel: marshal request body: %w", err)
		}
		bodyReader = bytes.NewReader(b)
	}

	req, err := http.NewRequestWithContext(ctx, method, url, bodyReader)
	if err != nil {
		return nil, err
	}
	req.Header.Set("Authorization", "Bearer "+p.token)
	req.Header.Set("Content-Type", "application/json")
	req.Header.Set("User-Agent", "BytePort/1.0")
	return req, nil
}

func (p *VercelProvider) doRequest(ctx context.Context, method, path string, body, dest any) error {
	req, err := p.newRequest(ctx, method, path, body)
	if err != nil {
		return err
	}
	resp, err := p.httpClient.Do(req)
	if err != nil {
		return fmt.Errorf("vercel: %s %s — HTTP transport error: %w", method, path, err)
	}
	defer resp.Body.Close()

	if resp.StatusCode == http.StatusUnauthorized {
		return fmt.Errorf("vercel: 401 Unauthorized — check your token at https://vercel.com/account/tokens")
	}
	if resp.StatusCode == http.StatusNotFound {
		return fmt.Errorf("vercel: 404 Not Found — resource does not exist")
	}
	if resp.StatusCode < 200 || resp.StatusCode >= 300 {
		raw, _ := io.ReadAll(resp.Body)
		return fmt.Errorf("vercel: %s %s returned %d: %s", method, path, resp.StatusCode, string(raw))
	}
	if dest == nil {
		return nil
	}
	return json.NewDecoder(resp.Body).Decode(dest)
}

func (p *VercelProvider) apiGET(ctx context.Context, path string, dest any) error {
	return p.doRequest(ctx, http.MethodGet, path, nil, dest)
}

func (p *VercelProvider) apiPOST(ctx context.Context, path string, body, dest any) error {
	return p.doRequest(ctx, http.MethodPost, path, body, dest)
}

func (p *VercelProvider) apiDELETE(ctx context.Context, path string) error {
	return p.doRequest(ctx, http.MethodDelete, path, nil, nil)
}

// ---------------------------------------------------------------------------
// LogStream implementation for Vercel NDJSON event stream
// ---------------------------------------------------------------------------

type vercelLogStream struct {
	decoder *json.Decoder
	closer  io.Closer
}

func (s *vercelLogStream) Next() (*LogEntry, error) {
	var event struct {
		Type    string `json:"type"`
		Created int64  `json:"created"`
		Payload struct {
			Text string `json:"text"`
			ID   string `json:"id"`
		} `json:"payload"`
	}
	if err := s.decoder.Decode(&event); err != nil {
		return nil, err // io.EOF signals end of stream
	}
	ts := time.Now()
	if event.Created > 0 {
		ts = time.UnixMilli(event.Created)
	}
	return &LogEntry{
		Timestamp: ts,
		Level:     event.Type,
		Message:   event.Payload.Text,
	}, nil
}

func (s *vercelLogStream) Close() error { return s.closer.Close() }

// ---------------------------------------------------------------------------
// Registry init
// ---------------------------------------------------------------------------

func init() {
	MustRegister(
		ProviderMetadata{
			Name:    "vercel",
			Version: "1.0.0",
			SupportedResources: []ResourceType{
				ResourceTypeComputeEdge,
				ResourceTypeComputeFunction,
			},
			Capabilities: []Capability{
				CapabilityLoggable,
				CapabilityCustomDNS,
			},
			Regions: []Region{
				{ID: "iad1", Name: "Washington, D.C., USA", Location: "US East", Available: true},
				{ID: "sfo1", Name: "San Francisco, CA, USA", Location: "US West", Available: true},
			},
			AuthTypes:   []string{"token"},
			Description: "Vercel — frontend/serverless deployment platform",
		},
		NewVercelProvider,
	)
}
