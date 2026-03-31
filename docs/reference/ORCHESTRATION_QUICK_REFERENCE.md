# Cloud Orchestration Quick Reference

**TL;DR:** For intelligent, cost-effective, multi-platform event-driven automation at Phenotype scale.

---

## Platform Choice Matrix

### Use GitHub Actions If:
- ✅ Build/test/deploy pipeline (standard CI/CD)
- ✅ Code-native workflows (within repo)
- ✅ Public repo (free execution)
- ✅ <100 concurrent jobs
- ❌ NOT intelligent agents or complex multi-step workflows

**Cost:** Free (public) or $0.002/min (private, with concurrency control)

---

### Use n8n Self-Hosted If:
- ✅ Complex multi-step workflow (AI agent + 5+ tools)
- ✅ High volume automation (1M+ events/month)
- ✅ Data privacy critical (on-premises)
- ✅ Budget conscious (unlimited execution cost)
- ❌ NOT quick, simple integrations

**Cost:** $0 (code) + $100-500/month (infrastructure)

**Setup:** `docker run -d -p 5678:5678 n8nio/n8n`

---

### Use Zapier If:
- ✅ Quick integrations (2-3 steps)
- ✅ Non-technical user
- ✅ Low volume (<10k tasks/month)
- ❌ NOT complex workflows or high volume

**Cost:** $20/month (750 tasks) → $500+/month (high volume)

---

### Use Make If:
- ✅ Complex workflow (10+ steps, deep app integrations)
- ✅ High volume operations (Zapier too expensive)
- ✅ Budget ~$100-300/month
- ❌ NOT on-premises or data-sensitive

**Cost:** $9/month (10k ops) → $200+/month (high volume)

---

### Use Custom + Claude Agent SDK If:
- ✅ Intelligent agent reasoning required
- ✅ Multi-platform orchestration (GitHub + Slack + Jira + custom tools)
- ✅ Precise control over agent behavior
- ✅ Cost optimization ($0.20 per 1M simple invocations)
- ✅ Security/compliance critical

**Cost:** $200-500/month (AWS Lambda + API Gateway)

---

## Common Use Cases → Recommended Platform

| Use Case | Platform | Why |
|----------|----------|-----|
| **PR Code Review** | Custom (Claude Agent SDK) | Intelligent reasoning over code diff |
| **CI Failure Triage** | n8n (AI Agent node) | Complex multi-step, analyze logs, create ticket |
| **Security Alert Response** | n8n or Custom | Auto-patch, test, create PR |
| **Scheduled Report** | Custom Lambda + Claude | Cost-effective for periodic intelligent summaries |
| **Multi-Repo Coordination** | Custom (subagents) | Parallel repo agents, consolidate results |
| **Slack → Jira** | Zapier | Simple, quick, non-technical |
| **GitHub → Slack → Email** | Make | More complex than Zapier, but cheaper than niche tools |
| **Standard CI/CD** | GitHub Actions | Native, cheap, already in use |
| **High-Volume Processing** | n8n self-hosted | Unlimited scale, cost-effective |

---

## Cost Comparison (1M Events/Month Baseline)

| Platform | Cost | Notes |
|----------|------|-------|
| **GitHub Actions** | $14,400+ | 1M tasks × $0.002/min; avg 1.2 min/task |
| **n8n Self-Hosted** | $100-300 | Zero execution cost; infrastructure only |
| **Zapier** | $600-5,000 | 1M tasks / 750/month → ~$800-5k depending on plan |
| **Make** | $100-300 | 1M operations / 10k ops = 100 scenarios → $100-300 |
| **Custom (Lambda)** | $200-500 | ~$0.0002 per invocation + $0.0000167/vCPU-sec |

**Winner at Scale:** n8n self-hosted (free) or Custom Lambda ($200-500)

---

## Implementation Quickstart

### Option 1: PR Review Agent (Custom Lambda)

**5 minutes to deploy:**

```bash
# 1. Create Lambda function (Python)
# 2. Add Claude Agent SDK dependency
# 3. Deploy zip to Lambda
# 4. Create API Gateway endpoint
# 5. Add GitHub webhook: Settings → Webhooks → {API Gateway URL}

# Example code:
from anthropic import Anthropic

def lambda_handler(event, context):
    webhook = json.loads(event['body'])

    client = Anthropic()
    response = client.messages.create(
        model="claude-opus-4-6",
        max_tokens=2048,
        system="You are a code reviewer.",
        messages=[{
            "role": "user",
            "content": f"Review this PR:\n{webhook['pull_request']['diff']}"
        }]
    )

    # Post review to GitHub
    return {'statusCode': 200}
```

**Cost:** ~$0.20 per 1M PRs

---

### Option 2: CI Failure Handler (n8n Self-Hosted)

**30 minutes to deploy:**

```bash
# 1. Launch n8n: docker run -p 5678:5678 n8nio/n8n
# 2. Access http://localhost:5678
# 3. Create workflow:
#    - Trigger: Webhook (GitHub posts CI failure)
#    - Node 1: Fetch CI logs (GitHub API)
#    - Node 2: AI Agent (Claude, analyze logs)
#    - Node 3: Create Jira ticket
#    - Node 4: Slack notification
# 4. Configure GitHub webhook to n8n URL

Cost: $0 (code) + compute
```

---

### Option 3: Complex Workflow (Make)

**20 minutes to set up:**

```
1. Go to Make.com
2. Create scenario:
   - Trigger: GitHub PR created
   - Step 1: HTTP GET (fetch diff)
   - Step 2: Code module (JavaScript logic)
   - Step 3: Jira create issue
   - Step 4: Slack send message
3. Test, then activate

Cost: $9/month + operations
```

---

## Decision Flowchart

```
START
 ↓
Is it simple (2-3 steps, single platform)?
 → YES → Is budget the issue?
         → YES → Use Zapier
         → NO  → Use Zapier (simplest)
 → NO  → Is data privacy critical?
         → YES → Use n8n self-hosted
         → NO  → Next step...

Does it need intelligent agent reasoning?
 → YES → Use Custom (Claude Agent SDK + Lambda)
 → NO  → Is it multi-step complex workflow?
         → YES → Use n8n self-hosted (unlimited, cost-effective)
         → NO  → Is it very high volume (1M+/month)?
                 → YES → Use n8n self-hosted or Custom Lambda
                 → NO  → Use Make or Zapier

Is it standard CI/CD (build/test/deploy)?
 → YES → Use GitHub Actions
 → NO  → See above
```

---

## Phenotype Recommendation

### Immediate (Weeks 1-2)

**Deploy Custom Lambda PR Review Agent:**
```
GitHub PR webhook → Lambda (Claude Agent) → Post review to GitHub
```

- **Setup Time:** 2-4 hours
- **Cost:** ~$100/month (even at high volume)
- **Benefit:** Intelligent PR review, reduces manual overhead

### Short-term (Weeks 3-4)

**Deploy n8n Self-Hosted CI Failure Handler:**
```
CI failure → n8n webhook → AI Agent → Analyze + Jira + Slack
```

- **Setup Time:** 4-6 hours
- **Cost:** $0 (code) + $50-100/month (t3.small EC2)
- **Benefit:** Automated triage, reduces MTTR

### Medium-term (Weeks 5-6)

**Multi-Repo Orchestrator (Subagent Pattern):**
```
Manual trigger → Claude Agent orchestrator
                 ↓ (spawns 10 subagents in parallel)
                 ├─ phenotype-infrakit agent
                 ├─ heliosCLI agent
                 ├─ AgilePlus agent
                 └─ ... (more repos)
                 ↓
                 Consolidate results → Create cross-repo PRs
```

- **Cost:** $200-300/month
- **Benefit:** Coordinate changes across polyrepo

### Long-term (Months 2+)

**Expand to security alert response, dependency management, scheduled reports.**

---

## Gotchas & Tips

### GitHub Actions
- ⚠️ **Billing Trap:** $0.002/min adds up fast (1k min/month = $20/month on top of runner costs)
- ✅ **Tip:** Use concurrency limits to control costs
- ✅ **Tip:** Prefer self-hosted runners for high-volume, save $0.002/min

### n8n
- ⚠️ **Learning Curve:** Visual builder simpler than code, but workflows can get complex
- ✅ **Tip:** Use "Google Sheets" as poor-man's database for simple state
- ✅ **Tip:** Self-hosted = no execution limits; cloud = metered

### Custom Lambda
- ⚠️ **Cold Start:** 100-500ms first invocation (use provisioned concurrency if <100ms needed)
- ✅ **Tip:** Use SQS for async, Lambda for event-driven (natural fit)
- ✅ **Tip:** Logs automatically in CloudWatch; export to Datadog for observability

### Webhook Reliability
- ⚠️ **Webhook Delivery:** Not guaranteed (use idempotency keys)
- ✅ **Tip:** Store incoming webhook in DLQ; process async from queue
- ✅ **Tip:** GitHub retries 25 times over 2.5 hours (good default)

---

## Key Metrics to Track

Once deployed, monitor:

1. **Execution Success Rate** (target: >99%)
2. **Response Latency** (target: <5 sec for interactive, <1 min for async)
3. **Cost per Event** (target: <$0.001)
4. **Agent Accuracy** (% of decisions human reviewers agree with)
5. **Time to Resolution** (MTTR improvement)

---

## Reference Links

- **Claude Agent SDK:** https://www.anthropic.com/engineering/building-agents-with-the-claude-agent-sdk
- **n8n Docs:** https://docs.n8n.io/
- **GitHub Webhooks:** https://docs.github.com/en/webhooks
- **AWS Lambda:** https://docs.aws.amazon.com/lambda/
- **Zapier vs Make:** https://zapier.com/blog/zapier-vs-make/

---

**Next Steps:**
1. Choose a single use case (recommend: PR review agent)
2. Deploy proof of concept (2-4 hours)
3. Test on low-volume, non-production repo
4. Monitor costs and accuracy
5. Scale to main repos
