# Cloud Orchestration Implementation Templates

Ready-to-use code templates for common event-driven agent automation patterns.

---

## Template 1: GitHub PR Review Agent (AWS Lambda + Claude)

### Overview
Trigger on GitHub PR webhook, analyze code, post intelligent review comment.

### Setup

**1. Lambda Function (Python 3.12)**

```python
# lambda_function.py
import json
import hmac
import hashlib
import boto3
import httpx
from anthropic import Anthropic

# GitHub webhook secret from environment
GITHUB_SECRET = os.getenv('GITHUB_WEBHOOK_SECRET')
GITHUB_TOKEN = os.getenv('GITHUB_TOKEN')

def verify_github_signature(request_body: bytes, signature: str) -> bool:
    """Validate GitHub webhook HMAC signature."""
    expected = 'sha256=' + hmac.new(
        GITHUB_SECRET.encode(),
        request_body,
        hashlib.sha256
    ).hexdigest()
    return hmac.compare_digest(signature, expected)

def fetch_pr_diff(owner: str, repo: str, pr_number: int) -> str:
    """Fetch PR diff from GitHub API."""
    url = f"https://api.github.com/repos/{owner}/{repo}/pulls/{pr_number}/files"
    headers = {
        "Authorization": f"token {GITHUB_TOKEN}",
        "Accept": "application/vnd.github.v3+raw"
    }

    files = []
    response = httpx.get(url, headers=headers)
    response.raise_for_status()

    for file in response.json():
        files.append(f"File: {file['filename']}\n{file['patch']}\n")

    return "".join(files[:10])  # Limit to first 10 files

def post_review_comment(owner: str, repo: str, pr_number: int,
                       comment: str) -> None:
    """Post review comment to GitHub PR."""
    url = f"https://api.github.com/repos/{owner}/{repo}/issues/{pr_number}/comments"
    headers = {
        "Authorization": f"token {GITHUB_TOKEN}",
        "Accept": "application/vnd.github.v3+json"
    }

    payload = {"body": comment}
    response = httpx.post(url, headers=headers, json=payload)
    response.raise_for_status()

def analyze_with_claude(pr_diff: str, pr_title: str) -> str:
    """Use Claude Agent to analyze PR."""
    client = Anthropic()

    response = client.messages.create(
        model="claude-opus-4-6",
        max_tokens=2048,
        system="""You are an expert code reviewer. Analyze pull requests for:
        - Security vulnerabilities
        - Code quality and style issues
        - Performance concerns
        - Test coverage gaps
        - Documentation improvements

        Provide constructive, actionable feedback. Be concise (max 500 words).""",
        messages=[{
            "role": "user",
            "content": f"PR Title: {pr_title}\n\nDiff:\n{pr_diff}"
        }]
    )

    return response.content[0].text

def lambda_handler(event, context):
    """Lambda handler for GitHub webhook."""

    # Validate webhook signature
    signature = event['headers'].get('X-Hub-Signature-256', '')
    body = event['body'].encode() if isinstance(event['body'], str) else event['body']

    if not verify_github_signature(body, signature):
        return {
            'statusCode': 401,
            'body': json.dumps({'error': 'Invalid signature'})
        }

    # Parse webhook payload
    payload = json.loads(event['body'])

    # Only process PR opened/synchronize events
    if payload['action'] not in ['opened', 'synchronize']:
        return {'statusCode': 200, 'body': json.dumps({'skipped': True})}

    # Extract PR details
    pr = payload['pull_request']
    owner = payload['repository']['owner']['login']
    repo = payload['repository']['name']
    pr_number = pr['number']
    pr_title = pr['title']

    try:
        # Fetch PR diff
        print(f"Fetching diff for PR #{pr_number}")
        diff = fetch_pr_diff(owner, repo, pr_number)

        # Analyze with Claude
        print("Analyzing with Claude...")
        review = analyze_with_claude(diff, pr_title)

        # Post review comment
        print("Posting review comment...")
        post_review_comment(owner, repo, pr_number, f"## Code Review\n\n{review}")

        return {
            'statusCode': 200,
            'body': json.dumps({
                'success': True,
                'reviewed_pr': pr_number,
                'comment_posted': True
            })
        }

    except Exception as e:
        print(f"Error: {str(e)}")

        # Post error comment
        error_msg = f"Code review failed: {str(e)}"
        try:
            post_review_comment(owner, repo, pr_number,
                              f"❌ {error_msg}")
        except:
            pass

        return {
            'statusCode': 500,
            'body': json.dumps({'error': str(e)})
        }
```

**2. CloudFormation Template (IaC)**

```yaml
# template.yaml
AWSTemplateFormatVersion: '2010-09-09'
Description: 'GitHub PR Review Agent Lambda'

Parameters:
  GitHubToken:
    Type: String
    NoEcho: true
    Description: GitHub Personal Access Token

  GitHubWebhookSecret:
    Type: String
    NoEcho: true
    Description: Webhook secret for signature validation

Resources:
  # Lambda Execution Role
  LambdaExecutionRole:
    Type: AWS::IAM::Role
    Properties:
      AssumeRolePolicyDocument:
        Version: '2012-10-17'
        Statement:
          - Effect: Allow
            Principal:
              Service: lambda.amazonaws.com
            Action: sts:AssumeRole
      ManagedPolicyArns:
        - arn:aws:iam::aws:policy/service-role/AWSLambdaBasicExecutionRole

  # Lambda Function
  PRReviewFunction:
    Type: AWS::Lambda::Function
    Properties:
      FunctionName: github-pr-review-agent
      Runtime: python3.12
      Role: !GetAtt LambdaExecutionRole.Arn
      Timeout: 60
      MemorySize: 256
      Environment:
        Variables:
          GITHUB_TOKEN: !Ref GitHubToken
          GITHUB_WEBHOOK_SECRET: !Ref GitHubWebhookSecret
      Code:
        ZipFile: |
          # [Paste lambda_function.py code here]

  # API Gateway
  APIGateway:
    Type: AWS::ApiGatewayV2::Api
    Properties:
      Name: github-pr-review-webhook
      ProtocolType: HTTP
      Target: !Sub 'arn:aws:lambda:${AWS::Region}:${AWS::AccountId}:function:${PRReviewFunction}'

  # Lambda Permission for API Gateway
  LambdaPermission:
    Type: AWS::Lambda::Permission
    Properties:
      FunctionName: !Ref PRReviewFunction
      Action: lambda:InvokeFunction
      Principal: apigateway.amazonaws.com
      SourceArn: !Sub 'arn:aws:execute-api:${AWS::Region}:${AWS::AccountId}:${APIGateway}/*/*'

Outputs:
  WebhookURL:
    Description: GitHub Webhook URL
    Value: !Sub 'https://${APIGateway}.execute-api.${AWS::Region}.amazonaws.com/pr-review'
```

**3. Deployment**

```bash
# Install dependencies
pip install -r requirements.txt -t package/
cd package && zip -r ../deployment.zip . && cd ..
zip deployment.zip lambda_function.py

# Deploy with CloudFormation
aws cloudformation create-stack \
  --stack-name github-pr-review \
  --template-body file://template.yaml \
  --parameters \
    ParameterKey=GitHubToken,ParameterValue=ghp_xxxxx \
    ParameterKey=GitHubWebhookSecret,ParameterValue=your_secret

# Get webhook URL
aws cloudformation describe-stacks --stack-name github-pr-review \
  --query 'Stacks[0].Outputs[0].OutputValue' --output text
```

**4. GitHub Webhook Setup**

```
Repository Settings → Webhooks → Add webhook
  Payload URL: [Copy from CloudFormation Output]
  Content type: application/json
  Secret: [Same as GITHUB_WEBHOOK_SECRET]
  Events: Pull requests
  Active: ✓
```

---

## Template 2: CI Failure Analyzer (n8n Workflow)

### Overview
Triggered by GitHub Actions workflow failure. Fetches logs, analyzes with AI, creates Jira ticket.

### n8n Workflow JSON

```json
{
  "name": "CI Failure Analyzer",
  "nodes": [
    {
      "parameters": {
        "httpMethod": "POST",
        "path": "a1b2c3d4"
      },
      "name": "Webhook",
      "type": "n8n-nodes-base.webhook",
      "typeVersion": 1,
      "position": [
        250,
        300
      ]
    },
    {
      "parameters": {
        "method": "GET",
        "url": "=`https://api.github.com/repos/${$json.payload.repository.owner.login}/${$json.payload.repository.name}/actions/runs/${$json.payload.workflow_run.id}/attempts/${$json.payload.workflow_run.run_attempt}/logs`",
        "authentication": "predefinedCredentialType",
        "nodeCredentialType": "githubApi",
        "options": {
          "headers": {
            "parameters": [
              {
                "name": "Accept",
                "value": "application/vnd.github.v3+raw"
              }
            ]
          }
        }
      },
      "name": "Fetch CI Logs",
      "type": "n8n-nodes-base.httpRequest",
      "typeVersion": 4.2,
      "position": [
        450,
        300
      ]
    },
    {
      "parameters": {
        "model": "claude-opus-4-6",
        "messages": {
          "values": [
            {
              "contentType": "text",
              "textContent": "=Analyze these CI logs and identify the root cause. Suggest a fix.\n\nLogs:\n{{ $node[\"Fetch CI Logs\"].json.body }}"
            }
          ]
        },
        "options": {
          "maxTokens": 1024
        }
      },
      "name": "AI Agent: Analyze Logs",
      "type": "@n8n/nodes-langchain.agent",
      "typeVersion": 1,
      "position": [
        650,
        300
      ]
    },
    {
      "parameters": {
        "resource": "issue",
        "operation": "create",
        "project": "INFRA",
        "summary": "=CI Failure: {{ $json.payload.workflow_run.name }}",
        "description": "=**Workflow:** {{ $json.payload.workflow_run.name }}\n**Run ID:** {{ $json.payload.workflow_run.id }}\n**Repository:** {{ $json.payload.repository.full_name }}\n\n**AI Analysis:**\n{{ $node[\"AI Agent: Analyze Logs\"].json.output }}"
      },
      "name": "Create Jira Ticket",
      "type": "n8n-nodes-base.jira",
      "typeVersion": 2,
      "position": [
        850,
        300
      ]
    },
    {
      "parameters": {
        "channel": "#ci-failures",
        "messageType": "blocks",
        "blocks": "=[\n  {\n    \"type\": \"section\",\n    \"text\": {\n      \"type\": \"mrkdwn\",\n      \"text\": \"❌ CI Failure in {{ $json.payload.repository.name }}\\n*Workflow:* {{ $json.payload.workflow_run.name }}\\n*Jira:* {{ $node[\\\"Create Jira Ticket\\\"].json.key }}\"\n    }\n  },\n  {\n    \"type\": \"section\",\n    \"text\": {\n      \"type\": \"mrkdwn\",\n      \"text\": \"{{ $node[\\\"AI Agent: Analyze Logs\\\"].json.output }}\"\n    }\n  }\n]"
      },
      "name": "Slack Notification",
      "type": "n8n-nodes-base.slack",
      "typeVersion": 2,
      "position": [
        1050,
        300
      ]
    }
  ],
  "connections": {
    "Webhook": {
      "main": [
        [
          {
            "node": "Fetch CI Logs",
            "type": "main",
            "index": 0
          }
        ]
      ]
    },
    "Fetch CI Logs": {
      "main": [
        [
          {
            "node": "AI Agent: Analyze Logs",
            "type": "main",
            "index": 0
          }
        ]
      ]
    },
    "AI Agent: Analyze Logs": {
      "main": [
        [
          {
            "node": "Create Jira Ticket",
            "type": "main",
            "index": 0
          }
        ]
      ]
    },
    "Create Jira Ticket": {
      "main": [
        [
          {
            "node": "Slack Notification",
            "type": "main",
            "index": 0
          }
        ]
      ]
    }
  }
}
```

### GitHub Actions Trigger

```yaml
# .github/workflows/ci.yml
name: CI

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Run tests
        run: cargo test

  notify-failure:
    needs: test
    if: failure()
    runs-on: ubuntu-latest
    steps:
      - name: Notify n8n of failure
        uses: fjogeleit/http-request-action@v1
        with:
          url: 'https://n8n.example.com/webhook/ci-failure'
          method: 'POST'
          data: |
            {
              "repository": "${{ github.repository }}",
              "workflow": "${{ github.workflow }}",
              "run_id": "${{ github.run_id }}",
              "branch": "${{ github.ref }}"
            }
```

---

## Template 3: Security Alert Responder (Custom n8n Scenario)

### Overview
GitHub secret/CodeQL alert → Auto-create fix PR, run CI, notify team.

### Workflow Steps

```json
{
  "name": "Security Alert Auto-Responder",
  "nodes": [
    {
      "name": "GitHub Security Alert Webhook",
      "type": "webhook"
    },
    {
      "name": "Parse Alert",
      "type": "code",
      "code": "const alert = $json.payload.alert;\nreturn {\n  type: alert.rule.id,\n  severity: alert.rule.severity,\n  file: alert.most_recent_instance.location.path\n};"
    },
    {
      "name": "Create Branch",
      "type": "githubApi",
      "operation": "createBranch",
      "branchName": "=security-fix-{{ $json.type }}-{{ Date.now() }}"
    },
    {
      "name": "Commit Fix",
      "type": "githubApi",
      "operation": "commitFile",
      "message": "=security: fix {{ $json.type }} in {{ $json.file }}"
    },
    {
      "name": "Create PR",
      "type": "githubApi",
      "operation": "createPullRequest",
      "title": "=🔒 Security Fix: {{ $json.type }}",
      "body": "Auto-generated PR to fix security alert.\n\nAlert: {{ $json.severity }}"
    },
    {
      "name": "Notify Team",
      "type": "slack",
      "channel": "#security",
      "message": "🔒 Security alert fixed: {{ $json.type }} in {{ $json.file }}"
    }
  ]
}
```

---

## Template 4: Multi-Repo Orchestrator (Subagent Pattern)

### Python Implementation

```python
# orchestrator.py
"""Multi-repo orchestrator using Claude subagents."""

import asyncio
from anthropic import Anthropic

client = Anthropic()

REPOS = [
    "phenotype-infrakit",
    "heliosCLI",
    "AgilePlus",
    "phenotype-design",
]

async def repo_agent(repo_name: str, task: str) -> str:
    """Subagent for individual repo."""
    response = client.messages.create(
        model="claude-opus-4-6",
        max_tokens=1024,
        messages=[{
            "role": "user",
            "content": f"""You are an agent responsible for {repo_name}.

Task: {task}

Check the current state of the repo and report:
1. Current version
2. Dependency status
3. Any breaking changes
4. Recommended actions"""
        }]
    )

    return response.content[0].text

async def orchestrate(task: str) -> dict:
    """Orchestrate subagents in parallel."""

    # Launch subagents in parallel
    results = await asyncio.gather(*[
        asyncio.to_thread(repo_agent, repo, task)
        for repo in REPOS
    ])

    # Consolidate results
    consolidated = client.messages.create(
        model="claude-opus-4-6",
        max_tokens=2048,
        messages=[{
            "role": "user",
            "content": f"""Consolidate these repo reports into a unified action plan.

{chr(10).join(f'## {repo}:\n{result}' for repo, result in zip(REPOS, results))}

Create a summary with:
1. Cross-repo dependencies
2. Action items per repo
3. Execution order"""
        }]
    )

    return {
        "repo_reports": dict(zip(REPOS, results)),
        "consolidated_plan": consolidated.content[0].text
    }

# Usage
if __name__ == "__main__":
    result = asyncio.run(
        orchestrate("Upgrade all repos to latest Rust edition 2024")
    )
    print(result["consolidated_plan"])
```

---

## Template 5: Scheduled Report Generator

### AWS Lambda + CloudWatch Events

```python
# lambda_function.py
"""Generate weekly health report."""

import json
from datetime import datetime, timedelta
import boto3
import httpx
from anthropic import Anthropic

def get_repo_health(owner: str, repo: str, token: str) -> dict:
    """Fetch repo health metrics."""
    headers = {"Authorization": f"token {token}"}
    base = f"https://api.github.com/repos/{owner}/{repo}"

    # Fetch metrics
    prs = httpx.get(f"{base}/pulls?state=open", headers=headers).json()
    issues = httpx.get(f"{base}/issues?state=open", headers=headers).json()
    commits = httpx.get(
        f"{base}/commits?since={(datetime.now() - timedelta(weeks=1)).isoformat()}",
        headers=headers
    ).json()

    return {
        "open_prs": len(prs),
        "open_issues": len(issues),
        "commits_this_week": len(commits)
    }

def generate_report(owner: str, repos: list, token: str) -> str:
    """Generate weekly report with Claude."""

    # Gather metrics
    metrics = {repo: get_repo_health(owner, repo, token) for repo in repos}

    # Use Claude to generate summary
    client = Anthropic()

    response = client.messages.create(
        model="claude-opus-4-6",
        max_tokens=2048,
        messages=[{
            "role": "user",
            "content": f"""Generate a weekly health report for our repos:

{json.dumps(metrics, indent=2)}

Format: Markdown with:
- Overall health status
- Per-repo highlights
- Recommendations
- Action items"""
        }]
    )

    return response.content[0].text

def lambda_handler(event, context):
    """Lambda handler for scheduled report."""

    repos = ["phenotype-infrakit", "heliosCLI", "AgilePlus"]
    report = generate_report("KooshaPari", repos, os.getenv("GITHUB_TOKEN"))

    # Save to S3
    s3 = boto3.client("s3")
    s3.put_object(
        Bucket="repo-reports",
        Key=f"weekly/{datetime.now().date()}.md",
        Body=report,
        ContentType="text/markdown"
    )

    # Email report
    ses = boto3.client("ses")
    ses.send_email(
        Source="noreply@example.com",
        Destination={"ToAddresses": ["team@example.com"]},
        Message={
            "Subject": {"Data": f"Weekly Repo Report - {datetime.now().date()}"},
            "Body": {"Html": {"Data": f"<pre>{report}</pre>"}}
        }
    )

    return {"statusCode": 200}
```

### EventBridge Trigger

```yaml
# CloudFormation snippet
ScheduledReportRule:
  Type: AWS::Events::Rule
  Properties:
    ScheduleExpression: "cron(0 9 ? * MON *)"  # 9 AM Monday
    State: ENABLED
    Targets:
      - Arn: !GetAtt ReportGeneratorFunction.Arn
        RoleArn: !GetAtt EventBridgeRole.Arn
```

---

## Common Environment Variables

All templates expect these secrets in AWS Secrets Manager / Lambda environment:

```env
GITHUB_TOKEN=ghp_xxxxx                    # GitHub Personal Access Token
GITHUB_WEBHOOK_SECRET=your_secret         # Webhook signature secret
ANTHROPIC_API_KEY=sk-ant-xxxxx            # Claude API key
JIRA_HOST=https://jira.company.com        # Jira instance
JIRA_USER=automation@company.com          # Jira bot user
JIRA_TOKEN=xxxxx                          # Jira API token
SLACK_WEBHOOK_URL=https://hooks.slack.com/services/T/B/X  # Slack incoming webhook
```

---

## Deployment Checklists

### Before Going to Production

- [ ] **Security:** Webhook signatures verified (HMAC)
- [ ] **Secrets:** All credentials in Secrets Manager (not hardcoded)
- [ ] **Logging:** CloudWatch logs configured, retention set
- [ ] **Error Handling:** Try-catch around all API calls
- [ ] **Rate Limiting:** Backoff/retry logic for external APIs
- [ ] **Monitoring:** CloudWatch alarms for failures
- [ ] **Testing:** Tested on non-production repo first
- [ ] **Permissions:** IAM roles scoped to least privilege
- [ ] **Documentation:** Runbooks for troubleshooting
- [ ] **Cost Monitoring:** CloudWatch dashboard for usage/cost

### Lambda-Specific

- [ ] **Memory:** Sized appropriately (256MB is typical)
- [ ] **Timeout:** Set high enough (60s recommended)
- [ ] **Layers:** Dependencies packaged correctly
- [ ] **VPC:** If accessing private resources, VPC configured
- [ ] **Cold Start:** Monitored (usually <500ms)

### n8n-Specific

- [ ] **Database:** PostgreSQL configured (not SQLite)
- [ ] **Backups:** Automated backup schedule
- [ ] **Update:** Latest n8n version running
- [ ] **Monitoring:** Health check endpoint configured
- [ ] **Load Balancing:** If multiple instances, load balancer in place

---

## Testing Templates

### Unit Test (Lambda)

```python
import pytest
from unittest.mock import patch, MagicMock
from lambda_function import verify_github_signature, analyze_with_claude

def test_github_signature_validation():
    """Test webhook signature validation."""
    secret = "test_secret"
    body = b"test_body"

    import hmac, hashlib
    sig = 'sha256=' + hmac.new(secret.encode(), body, hashlib.sha256).hexdigest()

    assert verify_github_signature(body, sig) == True
    assert verify_github_signature(body, "invalid") == False

@patch('lambda_function.Anthropic')
def test_claude_analysis(mock_anthropic):
    """Test Claude analysis."""
    mock_response = MagicMock()
    mock_response.content[0].text = "Code looks good!"

    mock_anthropic.return_value.messages.create.return_value = mock_response

    result = analyze_with_claude("diff content", "PR Title")
    assert "Code looks good!" in result
```

---

## Troubleshooting

### Lambda Issues

**Problem:** Function timeout
**Solution:** Increase timeout, check API call latency, add timeout context

**Problem:** Cold start slow
**Solution:** Use provisioned concurrency, or accept cold start for async

**Problem:** Out of memory
**Solution:** Increase memory allocation, stream large responses

### GitHub API Issues

**Problem:** Rate limit hit
**Solution:** Implement exponential backoff, cache results, use GraphQL pagination

**Problem:** Webhook not firing
**Solution:** Check event filters, verify webhook delivery in Settings

### n8n Issues

**Problem:** Workflow slow
**Solution:** Add parallelization, cache API results, check database performance

**Problem:** PostgreSQL connection error
**Solution:** Check DB credentials, verify security group rules, test connectivity

---

**Version:** 1.0
**Last Updated:** March 30, 2026
