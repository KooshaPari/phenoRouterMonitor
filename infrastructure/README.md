# Infrastructure as Code

Modern infrastructure configuration for all languages and platforms.

## Structure

```
infrastructure/
├── docker/           # Docker & Docker Compose
├── kubernetes/       # Kubernetes manifests
└── terraform/        # Terraform modules
```

## Docker

```bash
# Build
docker build -t app:latest .

# Compose
docker compose up -d

# Multi-stage build for Bun
docker build --target runtime -t app:runtime .
```

## Kubernetes

```bash
# Apply manifests
kubectl apply -f kubernetes/

# Scale
kubectl scale deployment app --replicas=5

# Check status
kubectl get pods -l app=app
```

## Terraform

```bash
# Initialize
terraform init

# Plan
terraform plan -out=tfplan

# Apply
terraform apply tfplan
```

## Monitoring Stack

- Prometheus: http://localhost:9090
- Grafana: http://localhost:3001 (admin/admin)
- Jaeger: http://localhost:16686

## Security

- All secrets via Kubernetes Secrets or Vault
- Container scanning with Trivy
- Runtime security with Falco
