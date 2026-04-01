# Phenotype Tooling Stack — Complete Category Audit

**Generated:** 2026-03-31
**Agent:** Forge

---

## Current State Summary

| Category | Tool | Status | Notes |
|----------|------|--------|-------|
| Error Tracking | Sentry | ✅ LIVE | 3 projects configured |
| Code Quality | SonarQube/SonarCloud | ✅ Running/Lite | localhost:9000 / sonarcloud.io |
| SAST | Semgrep + CodeQL | ✅ Deployed | Phase 1 complete |
| Dependency Scanning | Snyk | ✅ Token Ready | Needs AgilePlus spec |
| CI/CD | GitHub Actions + CircleCI | ✅ Configured | CircleCI config created |
| Docker | OrbStack | ✅ Running | Docker context: orbstack |
| Coverage | sonar-scanner | ✅ Installed | CLI ready |
| Pre-commit hooks | 4-layer pipeline | ✅ Deployed | Phase 1 complete |
| Secret scanning | gitleaks | ✅ In CI | Some repos |
| License checking | cargo-deny | ✅ In CI | deny.toml |
| Monitoring | Sentry | ✅ | Error tracking only |

---

## Missing Categories — Priority Order

### 🔴 P0 — Critical Gaps

| Category | Tools | Why | Setup Effort |
|----------|-------|-----|---------------|
| **DAST** | OWASP ZAP | Web API security testing | 30 min |
| **Fuzzing** | cargo-fuzz, AFL++ | Find bugs before users | 1 hour |
| **IaC Scanning** | tfsec, checkov | Terraform/K8s security | 30 min |
| **Container Scanning** | Trivy, Grype | Image vulnerabilities | 30 min |
| **License Compliance** | FOSSA, license-check | Legal risk | 1 hour |

### 🟠 P1 — High Value

| Category | Tools | Why | Setup Effort |
|----------|-------|-----|---------------|
| **Supply Chain Security** | SLSA, Sigstore | Prevent tampering | 2 hours |
| **Secrets Management** | Vault, 1Password | Centralized secrets | 1 hour |
| **Performance Testing** | k6, Gatling | Load/stress testing | 2 hours |
| **API Testing** | Newman, Insomnia | Endpoint validation | 1 hour |
| **Contract Testing** | Pact | Microservice contracts | 2 hours |

### 🟡 P2 — Nice to Have

| Category | Tools | Why | Setup Effort |
|----------|-------|-----|---------------|
| **Log Management** | Loki, ELK Stack | Centralized logs | 3 hours |
| **Metrics** | Prometheus + Grafana | System observability | 2 hours |
| **Chat Notifications** | Slack, Discord | Team alerts | 30 min |
| **Feature Flags** | Unleash, Flagsmith | Gradual rollouts | 2 hours |
| **Chaos Engineering** | Litmus, Chaos Monkey | Resilience testing | 3 hours |
| **API Docs** | Scalar, Redocly | API documentation | 1 hour |

### 🟢 P3 — Advanced

| Category | Tools | Why | Setup Effort |
|----------|-------|-----|---------------|
| **Mutation Testing** | cargo-mutants | Test quality | 1 hour |
| **Documentation Lint** | cargo-spellcheck | Doc quality | 30 min |
| **Dependency Pins** | depcheck, pip-tools | Reproducibility | 1 hour |
| **Visual Testing** | Percy, Chromatic | UI regression | 2 hours |
| **CDN/Edge** | Cloudflare, Fastly | Performance | Varies |

---

## Detailed Gap Analysis

### 1. 🔴 DAST — Dynamic Application Security Testing

**What it does:** Tests running applications for vulnerabilities by sending malicious requests.

**Why you need it:**
- SAST only finds static vulnerabilities
- DAST finds runtime issues (SQL injection, XSS, CSRF)
- Completes the security testing picture

**Tools:**
| Tool | License | Best For |
|------|---------|----------|
| **OWASP ZAP** | Apache 2.0 | Free, comprehensive, GitHub Actions |
| **Burp Suite** | Paid | Professional pen testing |
| **Nuclei** | MIT | Fast template-based scanning |

**Setup:**
```yaml
# .github/workflows/dast.yml
- name: OWASP ZAP Scan
  uses: zaproxy/action-baseline@v0.9.0
  with:
    target: 'https://api.yoursite.com'
    docker_name: 'owasp/zap2docker-stable'
```

---

### 2. 🔴 Fuzzing — Automated Bug Finding

**What it does:** Generates random inputs to find crashes and undefined behavior.

**Why you need it:**
- Finds bugs that unit tests miss
- Rust has first-class fuzzing support
- Critical for security-critical code

**Tools:**
| Tool | Language | License |
|------|----------|---------|
| **cargo-fuzz** | Rust | Apache 2.0 |
| **AFL++** | C/C++ | Apache 2.0 |
| **libFuzzer** | C/C++ | LLVM |
| **Go Fuzz** | Go | Apache 2.0 |

**Setup:**
```bash
# Install
cargo install cargo-fuzz

# Add to project
cargo fuzz init

# Run
cargo fuzz run fuzz_target_1
```

---

### 3. 🔴 IaC Scanning — Infrastructure as Code Security

**What it does:** Scans Terraform, Kubernetes, Dockerfiles for misconfigurations.

**Why you need it:**
- Prevent cloud misconfigs before deployment
- Catch security issues in infrastructure code
- Works with your multi-cloud setup (AWS/GCP/Azure)

**Tools:**
| Tool | Scans | License |
|------|-------|---------|
| **tfsec** | Terraform | MIT |
| **checkov** | Terraform, K8s, Dockerfile | Apache 2.0 |
| **KICS** | 20+ IaC types | Apache 2.0 |
| **Hadolint** | Dockerfiles | Apache 2.0 |

**Setup:**
```yaml
# .github/workflows/iac-scan.yml
- name: tfsec
  uses: aquasecurity/tfsec-action@v1
```

---

### 4. 🔴 Container Scanning — Image Security

**What it does:** Scans Docker images for vulnerabilities before deployment.

**Why you need it:**
- Images have CVEs just like code
- Prevent vulnerable images from reaching production
- Part of supply chain security

**Tools:**
| Tool | License | Best For |
|------|---------|----------|
| **Trivy** | Apache 2.0 | All-in-one, GitHub Actions |
| **Grype** | Apache 2.0 | Fast, simple |
| **Anchore** | GPL | Enterprise |

**Setup:**
```yaml
# .github/workflows/container-scan.yml
- name: Trivy Scan
  uses: aquasecurity/trivy-action@master
  with:
    image-ref: 'your-image:tag'
    format: 'sarif'
```

---

### 5. 🔴 License Compliance — Legal Risk Management

**What it does:** Ensures dependencies use approved licenses.

**Why you need it:**
- GPL licenses can force open-sourcing your code
- Corporate compliance requirements
- Already partially covered with cargo-deny

**Tools:**
| Tool | Languages | License |
|------|----------|---------|
| **FOSSA** | 20+ | Freemium |
| **license-check** | JS/TS | MIT |
| **cargo-deny** | Rust | Apache 2.0 ✅ |
| **scancode** | Multi | Apache 2.0 |

**Setup:**
```yaml
# .github/workflows/license-check.yml
- name: FOSSA
  uses: fossa/fossa-action@v1
```

---

### 6. 🟠 Supply Chain Security — SLSA/Sigstore

**What it does:** Verifies artifact integrity and prevents tampering.

**Why you need it:**
- Protect against supply chain attacks
- Verify builds haven't been tampered with
- Required for enterprise compliance

**Tools:**
| Tool | Purpose | License |
|------|---------|---------|
| **Sigstore Cosign** | Image signing | Apache 2.0 |
| **SLSA Verifier** | Build provenance | Apache 2.0 |
| **Tekton Chains** | K8s supply chain | Apache 2.0 |

---

### 7. 🟠 Secrets Management — Vault/1Password

**What it does:** Centralized secrets storage and rotation.

**Why you need it:**
- GitHub Secrets has limits
- Need secret rotation
- Multi-environment secrets
- Audit trail for secret access

**Tools:**
| Tool | Type | License |
|------|------|---------|
| **HashiCorp Vault** | Self-hosted | BSL (free tier) |
| **1Password Connect** | Hybrid | Paid |
| **AWS Secrets Manager** | Cloud | Pay per use |
| ** Doppler** | SaaS | Freemium |

---

### 8. 🟠 Performance Testing — k6/Gatling

**What it does:** Load testing, stress testing, performance benchmarks.

**Why you need it:**
- Find bottlenecks before users
- Validate scaling assumptions
- SLA compliance

**Tools:**
| Tool | Language | License |
|------|----------|---------|
| **k6** | JS | AGPL (free) |
| **Gatling** | Scala | Apache 2.0 (free) |
| **Locust** | Python | MIT |
| **oha** | Rust | MIT |

---

### 9. 🟠 API Testing — Postman/Newman

**What it does:** API endpoint validation and automated testing.

**Why you need it:**
- Validate API contracts
- Integration testing
- CI/CD API validation

**Tools:**
| Tool | Type | License |
|------|------|---------|
| **Newman** | CLI | Apache 2.0 |
| **Insomnia** | GUI | MIT |
| **Bruno** | GUI/CLI | MIT |
| **Hurl** | CLI | Apache 2.0 |

---

### 10. 🟠 Contract Testing — Pact

**What it does:** Verify microservice API compatibility.

**Why you need it:**
- Prevent breaking changes between services
- Independent service deployment
- Consumer-driven contracts

---

### 11. 🟡 Log Management — Loki/ELK

**What it does:** Centralized log aggregation and search.

**Why you need it:**
- Debug across services
- Audit trails
- Compliance requirements

**Tools:**
| Tool | Type | License |
|------|------|---------|
| **Loki** | Self-hosted | AGPL |
| **ELK Stack** | Self-hosted | ELastic License |
| **Grafana Cloud** | SaaS | Free tier |
| **Datadog** | SaaS | Paid |

---

### 12. 🟡 Metrics — Prometheus + Grafana

**What it does:** System and application metrics collection and visualization.

**Why you need it:**
- Infrastructure monitoring
- Alerting on anomalies
- Capacity planning

**Setup:**
```yaml
# docker-compose.yml
prometheus:
  image: prom/prometheus
grafana:
  image: grafana/grafana
```

---

### 13. 🟡 Chat Notifications — Slack/Discord

**What it does:** Real-time alerts to team.

**Why you need it:**
- CI/CD failures
- Security alerts
- Deployment notifications

**Tools:**
| Platform | GitHub Integration | Free Tier |
|----------|-------------------|-----------|
| Slack | ✅ Native | 90 days |
| Discord | ✅ Via webhook | Unlimited |
| Microsoft Teams | ✅ Native | Limited |

---

### 14. 🟢 Chaos Engineering — Litmus

**What it does:** Deliberately break things to test resilience.

**Why you need it:**
- Validate failure modes
- Test incident response
- Improve reliability

**Tools:**
| Tool | K8s Native | License |
|------|-----------|---------|
| **Litmus** | Yes | Apache 2.0 |
| **Chaos Monkey** | No | Apache 2.0 |
| **AWS Fault Injection Simulator** | AWS only | Pay per use |

---

## Priority Roadmap

### Week 1 — Security Hardening
1. OWASP ZAP DAST scan (30 min)
2. Trivy container scanning (30 min)
3. tfsec IaC scanning (30 min)

### Week 2 — Quality Gates
1. k6 performance tests (2 hours)
2. License compliance (FOSSA) (1 hour)
3. Contract testing (Pact) (2 hours)

### Week 3 — Observability
1. Loki log aggregation (3 hours)
2. Prometheus metrics (2 hours)
3. Slack notifications (30 min)

### Week 4 — Supply Chain
1. Sigstore image signing (2 hours)
2. SLSA provenance (2 hours)
3. Vault secrets management (1 hour)

---

## Quick Start Commands

```bash
# DAST - OWASP ZAP
docker run -t owasp/zap2docker-stable zap-baseline.py -t https://yoursite.com

# Container Scanning - Trivy
trivy image your-image:tag

# IaC Scanning - tfsec
tfsec .

# License Check - FOSSA
fossa init
fossa analyze

# Performance Testing - k6
k6 run script.js

# Secrets Scanning - Trivy
trivy config .
```

---

## Already Covered (Phase 1)

| Category | Tool | Status |
|----------|------|--------|
| SAST | Semgrep + CodeQL | ✅ |
| Error Tracking | Sentry | ✅ |
| Dependency Scanning | Snyk | ✅ |
| Coverage | sonar-scanner | ✅ |
| CI/CD | GitHub Actions | ✅ |
| Pre-commit | 4-layer hooks | ✅ |
| Linting | clippy, ruff, golangci | ✅ |
| Secret Detection | gitleaks | ✅ |
| License Checking | cargo-deny | ✅ |
| Docker | OrbStack | ✅ |

---

## Summary: 14 Categories Missing

| Priority | Count | Key Tools |
|----------|-------|-----------|
| P0 Critical | 5 | ZAP, Fuzzing, IaC, Container, License |
| P1 High Value | 5 | Supply Chain, Vault, k6, API, Pact |
| P2 Nice | 4 | Loki, Prometheus, Slack, Feature Flags |
| P3 Advanced | 5 | Mutation, Spellcheck, Visual, CDN, Chaos |

**Recommendation:** Start with P0 items — they provide the most security value with minimal setup time (combined ~3 hours).
