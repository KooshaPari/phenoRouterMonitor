// Package cloud — Netlify provider implementation.
// wraps: Netlify REST API v1 (https://open-api.netlify.com)
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

const netlifyAPIBase = "https://api.netlify.com/api/v1"

// NetlifyProvider implements CloudProvider for Netlify site deployments.
// Credentials.Data must contain "token" (personal access token).
type NetlifyProvider struct {
	credentials Credentials
	metadata    ProviderMetadata
	httpClient  *http.Client
	token       string
}

// compile-time interface assertion
var _ CloudProvider = (*NetlifyProvider)(nil)

// NewNetlifyProvider constructs a NetlifyProvider.
// Required credential key: "token".
func NewNetlifyProvider(credentials Credentials) (CloudProvider, error) {
	token := credentials.Data["token"]
	if token == "" {
		return nil, fmt.Errorf("netlify: credential key 'token' is required — generate one at https://app.netlify.com/user/applications")
	}

	return &NetlifyProvider{
		credentials: credentials,
		token:       token,
		httpClient:  &http.Client{Timeout: 30 * time.Second},
		metadata: ProviderMetadata{
			Name:    "netlify",
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
				{ID: "us-east-1", Name: "US East", Location: "Virginia, USA", Available: true},
			},
			AuthTypes:   []string{"token"},
			Description: "Netlify — JAMstack and serverless deployment platform",
		},
	}, nil
}

func (p *NetlifyProvider) GetMetadata() ProviderMetadata  { return p.metadata }
func (p *NetlifyProvider) GetCapabilities() []Capability  { return p.metadata.Capabilities }

func (p *NetlifyProvider) SupportsResource(resourceType ResourceType) bool {
	for _, t := range p.metadata.SupportedResources {
		if t == resourceType {
			return true
		}
	}
	return false
}

// Initialize re-sets credentials and validates them.
func (p *NetlifyProvider) Initialize(ctx context.Context, credentials Credentials) error {
	p.credentials = credentials
	p.token = credentials.Data["token"]
	return p.ValidateCredentials(ctx)
}

// ValidateCredentials calls GET /user to confirm the token is valid.
func (p *NetlifyProvider) ValidateCredentials(ctx context.Context) error {
	if p.token == "" {
		return fmt.Errorf("netlify: token is empty — cannot validate credentials")
	}
	var user struct {
		ID string `json:"id"`
	}
	if err := p.apiGET(ctx, "/user", &user); err != nil {
		return fmt.Errorf("netlify: credential validation failed: %w", err)
	}
	if user.ID == "" {
		return fmt.Errorf("netlify: token valid but returned empty user ID")
	}
	return nil
}

// CreateResource creates a Netlify site.
// config.Spec keys:
//
//	"custom_domain" — custom domain to attach (optional)
//	"repo_url"      — GitHub/GitLab repository URL for CI (optional)
//	"branch"        — branch to deploy from (default "main")
//	"build_command" — build command override
//	"publish_dir"   — publish directory override
func (p *NetlifyProvider) CreateResource(ctx context.Context, config ResourceConfig) (*Resource, error) {
	if !p.SupportsResource(config.Type) {
		return nil, NewNotSupportedError("netlify", string(config.Type))
	}

	body := map[string]any{
		"name": config.Name,
	}
	if domain, ok := config.Spec["custom_domain"].(string); ok && domain != "" {
		body["custom_domain"] = domain
	}
	if repoURL, ok := config.Spec["repo_url"].(string); ok && repoURL != "" {
		branch := "main"
		if b, ok := config.Spec["branch"].(string); ok && b != "" {
			branch = b
		}
		repoBody := map[string]any{
			"repo_url":     repoURL,
			"repo_branch":  branch,
			"provider":     "github",
		}
		if bc, ok := config.Spec["build_command"].(string); ok && bc != "" {
			repoBody["build_command"] = bc
		}
		if pd, ok := config.Spec["publish_dir"].(string); ok && pd != "" {
			repoBody["dir"] = pd
		}
		body["repo"] = repoBody
	}

	var site netlifySite
	if err := p.apiPOST(ctx, "/sites", body, &site); err != nil {
		return nil, fmt.Errorf("netlify: create site %q failed: %w", config.Name, err)
	}
	return netlifySiteToResource(site), nil
}

// GetResource retrieves a Netlify site by ID.
func (p *NetlifyProvider) GetResource(ctx context.Context, id string) (*Resource, error) {
	var site netlifySite
	if err := p.apiGET(ctx, "/sites/"+id, &site); err != nil {
		return nil, fmt.Errorf("netlify: get site %q failed: %w", id, err)
	}
	return netlifySiteToResource(site), nil
}

// UpdateResource is not currently implemented.
func (p *NetlifyProvider) UpdateResource(ctx context.Context, id string, config ResourceConfig) (*Resource, error) {
	return nil, NewNotSupportedError("netlify", "UpdateResource")
}

// DeleteResource deletes a Netlify site.
func (p *NetlifyProvider) DeleteResource(ctx context.Context, id string) error {
	if err := p.apiDELETE(ctx, "/sites/"+id); err != nil {
		return fmt.Errorf("netlify: delete site %q failed: %w", id, err)
	}
	return nil
}

// ListResources lists all sites for the authenticated user.
func (p *NetlifyProvider) ListResources(ctx context.Context, filter ResourceFilter) ([]*Resource, error) {
	var sites []netlifySite
	if err := p.apiGET(ctx, "/sites", &sites); err != nil {
		return nil, fmt.Errorf("netlify: list sites failed: %w", err)
	}
	resources := make([]*Resource, 0, len(sites))
	for _, s := range sites {
		s := s
		resources = append(resources, netlifySiteToResource(s))
	}
	return resources, nil
}

// Deploy triggers a new deploy for an existing site.
// config.ResourceID must be the Netlify site ID.
// config.Config keys:
//
//	"clear_cache" — "true" to clear CDN cache before deploying
func (p *NetlifyProvider) Deploy(ctx context.Context, config DeploymentConfig) (*Deployment, error) {
	if config.ResourceID == "" {
		return nil, fmt.Errorf("netlify: Deploy requires ResourceID (site ID)")
	}

	body := map[string]any{}
	if clear, ok := config.Config["clear_cache"].(string); ok && clear == "true" {
		body["clear_cache"] = true
	}
	if len(config.Env) > 0 {
		body["build_env"] = config.Env
	}

	var dep netlifyDeploy
	if err := p.apiPOST(ctx, "/sites/"+config.ResourceID+"/deploys", body, &dep); err != nil {
		return nil, fmt.Errorf("netlify: deploy site %q failed: %w", config.ResourceID, err)
	}
	return netlifyDeployToCloudDep(dep), nil
}

// GetDeploymentStatus fetches the status of a specific deploy.
func (p *NetlifyProvider) GetDeploymentStatus(ctx context.Context, id string) (*DeploymentStatus, error) {
	var dep netlifyDeploy
	if err := p.apiGET(ctx, "/deploys/"+id, &dep); err != nil {
		return nil, fmt.Errorf("netlify: get deploy %q failed: %w", id, err)
	}
	cloudDep := netlifyDeployToCloudDep(dep)
	return &DeploymentStatus{
		Deployment: cloudDep,
		Health:     netlifyStateToHealth(dep.State),
	}, nil
}

// RollbackDeployment is not directly supported; use a prior deploy ID via Deploy.
func (p *NetlifyProvider) RollbackDeployment(ctx context.Context, id string) error {
	// Netlify supports "restoring" a deploy by posting to /sites/{site_id}/deploys/{deploy_id}/restore
	// Caller must pass the site+deploy ID combo — we don't have site ID here so we can't implement cleanly.
	return NewNotSupportedError("netlify", "RollbackDeployment — call POST /sites/:site_id/deploys/:deploy_id/restore directly")
}

// GetLogs is not available via the Netlify management API for BytePort's use case.
func (p *NetlifyProvider) GetLogs(ctx context.Context, resource *Resource, opts LogOptions) (LogStream, error) {
	return nil, NewNotSupportedError("netlify", "GetLogs")
}

// GetMetrics is not available via the Netlify management API.
func (p *NetlifyProvider) GetMetrics(ctx context.Context, resource *Resource, opts MetricOptions) ([]Metric, error) {
	return nil, NewNotSupportedError("netlify", "GetMetrics")
}

// EstimateCost returns zero-cost (Netlify free tier assumed).
func (p *NetlifyProvider) EstimateCost(ctx context.Context, config ResourceConfig) (*CostEstimate, error) {
	return &CostEstimate{
		HourlyUSD:   0,
		DailyUSD:    0,
		MonthlyUSD:  0,
		Breakdown:   map[string]float64{"hosting": 0},
		Confidence:  "high",
		Currency:    "USD",
		LastUpdated: time.Now(),
	}, nil
}

// GetActualCost is not available via the Netlify management API.
func (p *NetlifyProvider) GetActualCost(ctx context.Context, resource *Resource, timeRange TimeRange) (*Cost, error) {
	return nil, NewNotSupportedError("netlify", "GetActualCost")
}

// ---------------------------------------------------------------------------
// Internal Netlify API types
// ---------------------------------------------------------------------------

type netlifySite struct {
	ID          string `json:"id"`
	Name        string `json:"name"`
	URL         string `json:"url"`
	AdminURL    string `json:"admin_url"`
	State       string `json:"state"`
	CreatedAt   string `json:"created_at"`
	UpdatedAt   string `json:"updated_at"`
}

type netlifyDeploy struct {
	ID        string `json:"id"`
	SiteID    string `json:"site_id"`
	State     string `json:"state"` // new, pending_review, enqueued, building, uploading, uploaded, ready, error
	URL       string `json:"url"`
	CreatedAt string `json:"created_at"`
	UpdatedAt string `json:"updated_at"`
}

func netlifySiteToResource(s netlifySite) *Resource {
	created := parseRFC3339OrNow(s.CreatedAt)
	updated := parseRFC3339OrNow(s.UpdatedAt)
	var endpoints []Endpoint
	if s.URL != "" {
		endpoints = append(endpoints, Endpoint{Type: "https", URL: s.URL, Primary: true})
	}
	return &Resource{
		ID:           s.ID,
		Name:         s.Name,
		Type:         ResourceTypeComputeEdge,
		Provider:     "netlify",
		Status:       DeploymentStateActive,
		HealthStatus: HealthStatusHealthy,
		Endpoints:    endpoints,
		CreatedAt:    created,
		UpdatedAt:    updated,
	}
}

func netlifyDeployToCloudDep(d netlifyDeploy) *Deployment {
	return &Deployment{
		ID:        d.ID,
		State:     netlifyStateToDeploymentState(d.State),
		Message:   d.URL,
		StartedAt: parseRFC3339OrNow(d.CreatedAt),
		UpdatedAt: parseRFC3339OrNow(d.UpdatedAt),
	}
}

func netlifyStateToDeploymentState(s string) DeploymentState {
	switch s {
	case "ready":
		return DeploymentStateActive
	case "error":
		return DeploymentStateFailed
	case "building", "uploading", "uploaded":
		return DeploymentStateBuilding
	case "new", "pending_review", "enqueued":
		return DeploymentStatePending
	default:
		return DeploymentStateDeploying
	}
}

func netlifyStateToHealth(s string) HealthStatus {
	switch s {
	case "ready":
		return HealthStatusHealthy
	case "error":
		return HealthStatusUnhealthy
	default:
		return HealthStatusChecking
	}
}

// ---------------------------------------------------------------------------
// HTTP helpers
// ---------------------------------------------------------------------------

func (p *NetlifyProvider) doRequest(ctx context.Context, method, path string, body, dest any) error {
	url := netlifyAPIBase + path
	var bodyReader io.Reader
	if body != nil {
		b, err := json.Marshal(body)
		if err != nil {
			return fmt.Errorf("netlify: marshal request body: %w", err)
		}
		bodyReader = bytes.NewReader(b)
	}

	req, err := http.NewRequestWithContext(ctx, method, url, bodyReader)
	if err != nil {
		return fmt.Errorf("netlify: build request: %w", err)
	}
	req.Header.Set("Authorization", "Bearer "+p.token)
	req.Header.Set("Content-Type", "application/json")
	req.Header.Set("User-Agent", "BytePort/1.0")

	resp, err := p.httpClient.Do(req)
	if err != nil {
		return fmt.Errorf("netlify: %s %s — HTTP transport error: %w", method, path, err)
	}
	defer resp.Body.Close()

	if resp.StatusCode == http.StatusUnauthorized {
		return fmt.Errorf("netlify: 401 Unauthorized — check your personal access token")
	}
	if resp.StatusCode == http.StatusNotFound {
		return fmt.Errorf("netlify: 404 Not Found — resource does not exist")
	}
	if resp.StatusCode < 200 || resp.StatusCode >= 300 {
		raw, _ := io.ReadAll(resp.Body)
		return fmt.Errorf("netlify: %s %s returned %d: %s", method, path, resp.StatusCode, string(raw))
	}
	if dest == nil {
		return nil
	}
	return json.NewDecoder(resp.Body).Decode(dest)
}

func (p *NetlifyProvider) apiGET(ctx context.Context, path string, dest any) error {
	return p.doRequest(ctx, http.MethodGet, path, nil, dest)
}

func (p *NetlifyProvider) apiPOST(ctx context.Context, path string, body, dest any) error {
	return p.doRequest(ctx, http.MethodPost, path, body, dest)
}

func (p *NetlifyProvider) apiDELETE(ctx context.Context, path string) error {
	return p.doRequest(ctx, http.MethodDelete, path, nil, nil)
}

// ---------------------------------------------------------------------------
// Registry init
// ---------------------------------------------------------------------------

func init() {
	MustRegister(
		ProviderMetadata{
			Name:    "netlify",
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
				{ID: "us-east-1", Name: "US East", Location: "Virginia, USA", Available: true},
			},
			AuthTypes:   []string{"token"},
			Description: "Netlify — JAMstack and serverless deployment platform",
		},
		NewNetlifyProvider,
	)
}
