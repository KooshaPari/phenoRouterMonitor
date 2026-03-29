# Module Federation: Production Deployment Guide

## Deployment Architecture

### Overview
```
┌─────────────────────────────────────────────────────┐
│                  Production Environment              │
├─────────────────────────────────────────────────────┤
│                                                       │
│  ┌──────────────────────────────────────────────┐   │
│  │  CDN / Static Hosting (AWS S3, Cloudflare)   │   │
│  │                                              │   │
│  │  agileplus.example.com/                     │   │
│  │  ├── index.html (host shell)                │   │
│  │  ├── remoteEntry.js (host federation)       │   │
│  │  ├── assets/                                │   │
│  │  │   ├── main.js                            │   │
│  │  │   └── ...                                │   │
│  │                                              │   │
│  │  remote-helios.example.com/                 │   │
│  │  ├── remoteEntry.js (heliosApp federation)  │   │
│  │  ├── assets/                                │   │
│  │  │   ├── dashboard.js                       │   │
│  │  │   └── ...                                │   │
│  │                                              │   │
│  │  remote-agent-wave.example.com/             │   │
│  │  ├── remoteEntry.js (agent-wave federation) │   │
│  │  ├── assets/                                │   │
│  │  │   ├── forecast.js                        │   │
│  │  │   └── ...                                │   │
│  └──────────────────────────────────────────────┘   │
│                                                       │
│  ┌──────────────────────────────────────────────┐   │
│  │  Optional: API / Backend Services             │   │
│  │  ├── auth.example.com/api/auth               │   │
│  │  ├── data.example.com/api/data               │   │
│  │  └── ai.example.com/api/predict              │   │
│  └──────────────────────────────────────────────┘   │
│                                                       │
└─────────────────────────────────────────────────────┘

User Browser
    ↓
  [1] Load https://agileplus.example.com/
    ↓
  [2] Fetch remoteEntry.js from host
    ↓
  [3] Click "Helios Dashboard"
    ↓
  [4] Fetch remoteEntry.js from remote-helios.example.com/
    ↓
  [5] Render module in host shell
```

## Pre-Deployment Checklist

### Code Quality
- [ ] All modules pass lint: `npm run lint`
- [ ] All tests pass: `npm run test`
- [ ] No console warnings or errors
- [ ] TypeScript strict mode passes: `tsc --noEmit`
- [ ] Code coverage >= 80%

### Dependency Alignment
- [ ] All modules use same React version (18.2.0)
- [ ] All modules use same @phenotype/docs version
- [ ] All modules use same phenotype-shared version
- [ ] No version conflicts: `npm ls` shows no duplicates

### Security
- [ ] No secrets in code (run gitleaks or similar)
- [ ] All dependencies updated: `npm audit`
- [ ] CORS headers configured correctly
- [ ] API keys/tokens from environment variables only
- [ ] CSP (Content Security Policy) headers set

### Module Federation Config
- [ ] Module names match across all builds
- [ ] Shared library list is identical
- [ ] Singleton flags correct for shared libs
- [ ] Eager loading enabled for critical libs (@phenotype/docs)
- [ ] Remote URLs point to production domains

### Documentation
- [ ] README.md updated with deployment info
- [ ] Environment variables documented
- [ ] Rollback procedure documented
- [ ] Health check endpoints defined
- [ ] Monitoring/alerting configured

## Build Process

### Step 1: Build Each Module

**AgilePlus Host**:
```bash
cd repos  # or agileplus directory
npm install
npm run lint
npm run test
npm run build
# Output: dist/
```

**heliosApp Remote**:
```bash
cd repos  # or heliosApp directory
npm install
npm run lint
npm run test
npm run build:remote
# Output: dist/
# Key: remoteEntry.js must be in dist/assets/
```

**agent-wave Remote**:
```bash
cd repos  # or agent-wave directory
npm install
npm run lint
npm run test
npm run build:remote
# Output: dist/
# Key: remoteEntry.js must be in dist/assets/
```

### Step 2: Generate Build Manifest

Create a manifest file to track builds:
```bash
# In CI/CD pipeline, after each build
cat > build-manifest.json <<EOF
{
  "timestamp": "$(date -Iseconds)",
  "agileplus": {
    "commit": "$(git -C repos rev-parse HEAD)",
    "version": "$(npm -C repos view . version)",
    "remoteEntry": "agileplus-v$(npm -C repos view . version).js"
  },
  "heliosApp": {
    "commit": "$(git -C heliosApp rev-parse HEAD)",
    "version": "$(npm -C heliosApp view . version)",
    "remoteEntry": "helios-v$(npm -C heliosApp view . version).js"
  },
  "agent-wave": {
    "commit": "$(git -C agent-wave rev-parse HEAD)",
    "version": "$(npm -C agent-wave view . version)",
    "remoteEntry": "wave-v$(npm -C agent-wave view . version).js"
  }
}
EOF
```

## Deployment Steps

### Option 1: AWS S3 + CloudFront

#### Step 1: Create S3 Buckets

```bash
# Host bucket
aws s3 mb s3://agileplus.example.com --region us-east-1

# Remote buckets
aws s3 mb s3://remote-helios.example.com --region us-east-1
aws s3 mb s3://remote-agent-wave.example.com --region us-east-1
```

#### Step 2: Configure S3 for Static Hosting

```bash
# For each bucket, set as public static hosting
aws s3 website s3://agileplus.example.com \
  --index-document index.html \
  --error-document index.html  # For SPA routing

# Repeat for remote buckets
```

#### Step 3: Upload Build Artifacts

```bash
# Upload AgilePlus
aws s3 sync repos/dist/ s3://agileplus.example.com/ \
  --delete \
  --cache-control "public, max-age=31536000" \
  --exclude "index.html" \
  --exclude "remoteEntry.js"

# Upload index.html with no-cache
aws s3 cp repos/dist/index.html \
  s3://agileplus.example.com/index.html \
  --cache-control "no-cache, no-store, must-revalidate"

# Upload remoteEntry.js with short TTL (revalidate often)
aws s3 cp repos/dist/assets/remoteEntry.js \
  s3://agileplus.example.com/assets/remoteEntry.js \
  --cache-control "public, max-age=300"  # 5 minute TTL

# Repeat for heliosApp and agent-wave remotes
```

#### Step 4: Create CloudFront Distribution

```bash
# Host distribution
aws cloudfront create-distribution \
  --origin-domain-name agileplus.example.com.s3.amazonaws.com \
  --default-root-object index.html \
  --default-cache-behavior '{
    "TargetOriginId": "S3Origin",
    "ViewerProtocolPolicy": "redirect-to-https",
    "AllowedMethods": ["GET", "HEAD", "OPTIONS"],
    "Compress": true,
    "ForwardedValues": { "QueryString": false }
  }'

# Repeat for remote distributions
```

### Option 2: Cloudflare Pages / Workers

#### Step 1: Setup Cloudflare Project

```bash
# Install Wrangler CLI
npm install -g wrangler

# Login
wrangler login

# Create project
wrangler pages project create agileplus-host
wrangler pages project create agileplus-helios-remote
wrangler pages project create agileplus-wave-remote
```

#### Step 2: Deploy to Cloudflare

```bash
# AgilePlus
wrangler pages deploy repos/dist --project-name agileplus-host

# heliosApp
wrangler pages deploy heliosApp/dist --project-name agileplus-helios-remote

# agent-wave
wrangler pages deploy agent-wave/dist --project-name agileplus-wave-remote
```

#### Step 3: Configure Custom Domains

```bash
# In Cloudflare dashboard or via API
# Map agileplus.example.com → agileplus-host.pages.dev
# Map remote-helios.example.com → agileplus-helios-remote.pages.dev
# Map remote-agent-wave.example.com → agileplus-wave-remote.pages.dev
```

## Environment Configuration

### Set Environment Variables for Production

**In CI/CD Pipeline** (before build):

```bash
# AgilePlus / .env.production
VITE_HELIOS_REMOTE=https://remote-helios.example.com/assets/remoteEntry.js
VITE_AGENT_WAVE_REMOTE=https://remote-agent-wave.example.com/assets/remoteEntry.js
VITE_API_BASE_URL=https://api.example.com
VITE_AUTH_PROVIDER=auth0
```

**In heliosApp** / .env.production:
```bash
VITE_API_BASE_URL=https://api.example.com
VITE_TELEMETRY_ENABLED=true
```

**In agent-wave** / .env.production:
```bash
VITE_API_BASE_URL=https://api.example.com
VITE_MODEL_ENDPOINT=https://model.example.com/predict
```

### Runtime Configuration

After deploying, applications can read config from:

```typescript
// src/config.ts
export const config = {
  // From build-time env vars
  heliosRemote: import.meta.env.VITE_HELIOS_REMOTE,
  agentWaveRemote: import.meta.env.VITE_AGENT_WAVE_REMOTE,

  // From runtime window object (set by CDN or API)
  apiBaseUrl: (window as any).__CONFIG__?.API_BASE_URL || 'https://api.example.com',
  telemetryEnabled: (window as any).__CONFIG__?.TELEMETRY || true,
};
```

Or inject config at runtime:

```html
<!-- In index.html (served by CDN) -->
<script>
  window.__CONFIG__ = {
    API_BASE_URL: 'https://api.example.com',
    TELEMETRY: true,
    // ... more config
  };
</script>
```

## CORS & Security Headers

### CORS Configuration

**For remoteEntry.js access**:

```bash
# In nginx (if proxying) or via S3 CORS policy
# Allow Cross-Origin Resource Sharing for remoteEntry.js

# AWS S3 CORS Policy
{
  "CORSRules": [
    {
      "AllowedOrigins": [
        "https://agileplus.example.com",
        "https://example.com"
      ],
      "AllowedMethods": ["GET", "HEAD"],
      "AllowedHeaders": ["*"],
      "ExposeHeaders": ["x-amz-version-id"]
    }
  ]
}
```

### Security Headers

```bash
# Set these headers on all responses (CDN, nginx, etc.)

# Content Security Policy
Content-Security-Policy: default-src 'self'; script-src 'self' https://remote-helios.example.com https://remote-agent-wave.example.com 'unsafe-inline'; style-src 'self' 'unsafe-inline'

# HSTS
Strict-Transport-Security: max-age=31536000; includeSubDomains

# Prevent clickjacking
X-Frame-Options: DENY

# XSS Protection
X-Content-Type-Options: nosniff

# CORS
Access-Control-Allow-Origin: https://agileplus.example.com

# Referrer Policy
Referrer-Policy: strict-origin-when-cross-origin
```

## Version Management

### Semantic Versioning for Modules

```json
{
  "name": "agileplus",
  "version": "1.0.0"
}
```

**When deploying**:
1. Increment version: `npm version major|minor|patch`
2. Tag in git: `git tag v1.0.0`
3. Include version in remoteEntry.js filename
4. Update build manifest

### Backwards Compatibility

**Keep old remoteEntry.js versions available**:

```
s3://remote-helios.example.com/
├── remoteEntry.js → points to latest
├── remoteEntry-v1.0.0.js
├── remoteEntry-v1.1.0.js
└── remoteEntry-v2.0.0.js
```

This allows:
1. Host can reference specific version if needed
2. Gradual rollout of new module versions
3. Easy rollback to previous version

## Health Checks & Monitoring

### Add Health Check Endpoint

**In heliosApp**:

```typescript
// src/routes/health.ts
export async function handleHealthCheck(req: Request) {
  return new Response(
    JSON.stringify({
      status: 'ok',
      timestamp: new Date().toISOString(),
      version: import.meta.env.VITE_VERSION || '1.0.0',
      checks: {
        remoteEntry: 'ok',
        dependencies: 'ok',
      },
    }),
    {
      status: 200,
      headers: { 'Content-Type': 'application/json' },
    },
  );
}

// Add route: GET /health
app.get('/health', handleHealthCheck);
```

### Monitor Module Load Times

```typescript
// In host (AgilePlus)
const measureModuleLoad = async (moduleName: string, url: string) => {
  const start = performance.now();
  try {
    const response = await fetch(url);
    const duration = performance.now() - start;

    // Send to monitoring service
    await fetch('https://telemetry.example.com/metrics', {
      method: 'POST',
      body: JSON.stringify({
        event: 'module_load',
        module: moduleName,
        duration,
        success: response.ok,
        timestamp: new Date().toISOString(),
      }),
    });

    return response.ok;
  } catch (error) {
    // Alert on failure
    console.error(`Failed to load ${moduleName}`, error);
    return false;
  }
};
```

### Alerting

**Setup alerts for**:
- Module load time > 5s
- Module load failure (HTTP 500)
- High error rate in console
- Health check endpoint down

**Using services**:
- Sentry for error tracking
- New Relic for performance monitoring
- Datadog for infrastructure metrics
- PagerDuty for critical alerts

## Rollback Procedure

### If New Deployment Has Issues

```bash
# Step 1: Identify current version
aws cloudfront get-distribution-config --id E123 | grep -i version

# Step 2: Identify previous stable version
# Check git tags: git tag -l | sort -V | tail -5

# Step 3: Rebuild previous version
git checkout v1.0.0  # stable tag
npm install
npm run build

# Step 4: Deploy previous build
aws s3 sync dist/ s3://agileplus.example.com/ --delete

# Step 5: Invalidate CloudFront cache
aws cloudfront create-invalidation \
  --distribution-id E123 \
  --paths "/*"

# Step 6: Verify in browser
# Clear browser cache and reload
```

### Gradual Rollout Strategy (Canary Deployment)

Instead of all-or-nothing:

```bash
# Deploy to canary domain first
aws s3 sync dist/ s3://canary-agileplus.example.com/ --delete

# Point 5% of traffic to canary via CloudFront
# Monitor metrics for 1 hour
# If stable, scale to 100%

# Use weighted routing in Route53
{
  "Name": "agileplus.example.com",
  "SetIdentifier": "Production",
  "Weight": 95,
  "AliasTarget": { "DNSName": "production-cf.example.com" }
},
{
  "Name": "agileplus.example.com",
  "SetIdentifier": "Canary",
  "Weight": 5,
  "AliasTarget": { "DNSName": "canary-cf.example.com" }
}
```

## Monitoring Checklist

### Post-Deployment Verification

- [ ] Host loads at https://agileplus.example.com
- [ ] Host → click "Helios Dashboard" → module loads from remote-helios.example.com
- [ ] Host → click "Agent Wave" → module loads from remote-agent-wave.example.com
- [ ] All modules use correct @phenotype/docs version
- [ ] No console errors or warnings
- [ ] Module load times < 3 seconds
- [ ] CSS loads correctly (theme applied)
- [ ] API calls to backend working
- [ ] Auth/login working
- [ ] Network tab shows no 404s

### Ongoing Monitoring

**Daily**:
- Check error logs for any spikes
- Verify module load times acceptable
- Check backend API response times

**Weekly**:
- Review performance metrics
- Check for unused code/dead links
- Audit dependency security updates

**Monthly**:
- Release notes / changelog
- Plan next version / features
- Review cost (CDN bandwidth, storage)

## CI/CD Pipeline Integration

### GitHub Actions Example

```yaml
# .github/workflows/deploy-agileplus.yml
name: Deploy AgilePlus

on:
  push:
    branches: [main]

jobs:
  build-and-deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Setup Node
        uses: actions/setup-node@v3
        with:
          node-version: '18'
          cache: 'npm'

      - name: Build
        run: |
          npm install
          npm run lint
          npm run test
          npm run build

      - name: Upload to S3
        run: |
          aws s3 sync dist/ s3://agileplus.example.com/ --delete
        env:
          AWS_ACCESS_KEY_ID: ${{ secrets.AWS_ACCESS_KEY_ID }}
          AWS_SECRET_ACCESS_KEY: ${{ secrets.AWS_SECRET_ACCESS_KEY }}

      - name: Invalidate CloudFront
        run: |
          aws cloudfront create-invalidation \
            --distribution-id E123 \
            --paths "/*"
        env:
          AWS_ACCESS_KEY_ID: ${{ secrets.AWS_ACCESS_KEY_ID }}
          AWS_SECRET_ACCESS_KEY: ${{ secrets.AWS_SECRET_ACCESS_KEY }}
```

### Trigger Deployment

```bash
# Just commit to main
git commit -m "feat: add new dashboard feature"
git push origin main
# GitHub Actions automatically builds and deploys
```

## Troubleshooting Production Issues

### Module fails to load in production

**Check**:
1. Remote URL is correct in vite.config.ts
2. remoteEntry.js accessible: `curl https://remote-helios.example.com/assets/remoteEntry.js`
3. CORS headers correct
4. Module versions match

**Fix**:
1. Check CloudFront invalidation queued
2. Clear browser cache (Cmd+Shift+Delete)
3. Check S3 bucket public access policy
4. Verify SecurityPolicy allows remote origin

### High error rate after deployment

**Check**:
1. Health check endpoints responding
2. API endpoints accessible
3. Backend services running
4. Database connections stable

**Fix**:
1. Review recent code changes
2. Check server logs for errors
3. Roll back to previous stable version
4. Page through error tracking (Sentry)

### Slow module loads in production

**Check**:
1. Network tab in DevTools
2. CDN performance (use tools like WebPageTest)
3. Bundle size (should be < 100KB for modules)

**Fix**:
1. Enable compression (gzip/brotli) on CDN
2. Code split large modules
3. Remove unused dependencies
4. Use tree-shaking / minification

## Cost Optimization

### Reduce CDN Costs

1. **Enable compression**: gzip/brotli reduces payload 70-80%
2. **Cache headers**: Long TTL for versioned assets
3. **Edge caching**: CloudFront edge locations
4. **Select appropriate region**: S3 region close to users

### Example Cost (AWS)

```
AgilePlus + 2 Remotes
- S3 storage: ~$1/month (10GB)
- CloudFront: ~$20/month (1TB CDN)
- Route53: ~$0.50/month (DNS)
Total: ~$21/month

Scaling to 100K users: ~$50-100/month
```

## Deployment Checklist

- [ ] All modules built successfully
- [ ] All tests passing
- [ ] No linting errors
- [ ] Security audit clean
- [ ] Environment variables set
- [ ] S3 buckets created
- [ ] CloudFront distributions created
- [ ] DNS records pointing to CloudFront
- [ ] SSL/TLS certificates valid
- [ ] CORS policies configured
- [ ] Security headers set
- [ ] Health checks configured
- [ ] Monitoring alerts set up
- [ ] Rollback procedure documented
- [ ] Team notified of deployment
- [ ] Deployment window scheduled
- [ ] Post-deployment verification checklist ready

---

**Document Version**: 1.0
**Last Updated**: 2026-03-29
**Status**: Production Deployment Guide (Ready for Use)
