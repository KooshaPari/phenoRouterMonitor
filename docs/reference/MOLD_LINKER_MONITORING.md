# Mold Linker - Monitoring & Maintenance Strategy

**Status**: Reference Documentation
**Date**: 2026-03-31
**Scope**: Ongoing monitoring, alerting, and maintenance procedures

---

## Monitoring Framework

### Key Metrics to Track

| Metric | Target | Alert Threshold | Measurement |
|--------|--------|-----------------|-------------|
| **Link Time** | 12s | >13.3s (>10% increase) | Per CI build |
| **Improvement %** | 73% | <65% or >85% | Per CI build |
| **Build Variance** | <1s | >2s (std dev) | 3-run average |
| **Linker Errors** | 0 | >0 errors | Per CI build |
| **mold Version** | Latest | >1 minor behind | Monthly audit |
| **Success Rate** | 100% | <99% | Monthly review |

---

## Real-Time Monitoring (Per-Build)

### Automated CI Alerts

**In GitHub Actions**, the mold benchmark job automatically:

1. **Measures** baseline and mold build times
2. **Calculates** improvement percentage
3. **Posts PR comment** with results

**Example PR Comment**:
```
🔗 **Mold Linker Benchmark Results**

| Metric | Time |
|--------|------|
| Baseline (GNU ld) | 45.2s |
| Mold Run 1 | 12.3s |
| Mold Run 2 | 12.1s |
| Mold Run 3 | 12.0s |
| Mold Average | 12.1s |
| **Improvement** | **73.2% faster** ✓ |
```

### Alert Condition 1: Link Time Regression

**Trigger**: If `MOLD_AVG > 13.3s` (>10% above target)

**Action in Workflow**:
```yaml
- name: Check for link time regression
  run: |
    THRESHOLD_S=13.3
    if (( $(echo "$MOLD_AVG > $THRESHOLD_S" | bc -l) )); then
      echo "⚠️ WARNING: Link time regression detected!"
      echo "Expected: ~12s, Got: ${MOLD_AVG}s"
      echo "This may indicate a workspace change or mold issue."
      exit 1  # Fail job to alert team
    fi
```

**Investigation Steps**:
1. Check recent commits (new dependencies?)
2. Review workspace Cargo.toml changes
3. Verify mold version
4. Check disk space
5. Investigate if issue is persistent

### Alert Condition 2: Improvement Below Target

**Trigger**: If improvement `< 60%` (target is 73%)

**Action**:
```yaml
- name: Validate improvement percentage
  run: |
    if (( $(echo "$IMPROVEMENT < 60" | bc -l) )); then
      echo "⚠️ WARNING: Improvement below 60%!"
      echo "Mold may not be functioning correctly."
      exit 1
    fi
```

**Investigation Steps**:
1. Verify mold is in PATH
2. Check if mold is being used: `cargo build -vv 2>&1 | grep mold`
3. Verify `.cargo/config.toml` has rustflags
4. Ensure ubuntu-latest runner (not macOS/Windows)

---

## Weekly Monitoring

### Cadence: Every Monday (or weekly review)

**Tasks**:

1. **Review last 7 CI runs**
   ```bash
   # Check GitHub Actions tab:
   # https://github.com/KooshaPari/phenotype-infrakit/actions/workflows/benchmark.yml

   # Look for:
   # - Any failed jobs
   # - Improvement % consistency
   # - Link time trend
   ```

2. **Calculate rolling average**
   ```
   Mold times from last 7 runs:
   12.3, 12.1, 12.0, 12.2, 12.1, 12.3, 12.0
   Average: 12.14s
   Std Dev: 0.12s
   Status: ✓ Stable
   ```

3. **Check for outliers**
   ```
   Outlier detection (±2 std dev):
   12.3 - outlier? No (within range)
   Latest: 12.0s - normal
   ```

4. **Document findings**
   ```
   Week of 2026-04-07:
   - 7 benchmark runs
   - Average: 12.14s ✓ On target
   - Improvement: 73.2% ✓ Matches target
   - Status: ✓ Stable
   ```

---

## Monthly Monitoring

### Cadence: Last Tuesday of each month

**Tasks**:

1. **Aggregate monthly metrics**
   ```
   April 2026 Mold Benchmark Summary
   ═════════════════════════════════════
   Total runs: 35 (5 per week × 7 weeks)

   Baseline average: 45.3s ± 1.2s
   Mold average: 12.1s ± 0.2s
   Improvement: 73.3% ± 0.8%

   All runs: ✓ PASS (no failures)
   Variance: <1s (excellent)
   Regressions: 0
   ```

2. **Review mold releases**
   ```
   Visit: https://github.com/rui314/mold/releases

   Latest stable: v2.4.1 (May 2026) - NOT YET
   Current installed: v2.4.x

   New features? Bug fixes? Security patches?
   ```

3. **Check for workspace changes**
   ```
   Workspace metrics (April):
   - Crates: 24 (no new crates)
   - Dependencies: ~150 (no major updates)
   - Workspace size: ~500MB (stable)
   ```

4. **Create monthly report**
   ```markdown
   # Mold Linker - Monthly Report (April 2026)

   ## Metrics Summary
   - Runs: 35/35 successful ✓
   - Average improvement: 73.3%
   - Variance: <1s
   - Regressions: 0

   ## mold Status
   - Version: 2.4.x
   - New releases: None
   - Security updates: None

   ## Recommendation
   - Continue using mold
   - No updates needed this month
   - Monitor next month for 2.5.x release
   ```

5. **Share with team**
   - Slack/Email summary
   - Include: Metrics, status, action items

---

## Quarterly Monitoring

### Cadence: Every 3 months (Jan 1, Apr 1, Jul 1, Oct 1)

**Tasks**:

1. **Annual trend analysis**
   ```
   Q1 2026 Performance Trend
   ═════════════════════════════════════
   January:   73.1% improvement (12.1s)
   February:  73.2% improvement (12.1s)
   March:     73.3% improvement (12.0s)
   Average Q1: 73.2% ± 0.1% ✓

   Trend: STABLE (no regression)
   ```

2. **Mold ecosystem evaluation**
   - Check GitHub mold issues (any regressions reported?)
   - Review Rust community discussions
   - Assess mold adoption in ecosystem
   - Any known incompatibilities?

3. **Consider version upgrade**
   ```
   mold v2.4.x → v2.5.x?

   Benefits:
   - Latest features
   - Latest bug fixes

   Risks:
   - Potential new issues
   - Need to re-baseline

   Decision: Upgrade if stable for 2+ weeks in ecosystem
   ```

4. **Update ADR** (Architecture Decision Record)
   ```markdown
   # ADR-NNN: Mold Linker Optimization

   ## Status: ACCEPTED

   ## Context
   - Performance baseline: 73% improvement
   - Stable for 3 months (Q1 2026)
   - Zero regressions

   ## Decision
   - Continue using mold linker
   - Upgrade to v2.5.x when stable
   - Re-evaluate quarterly

   ## Consequences
   - Link times: ~12s (vs 45s without mold)
   - Annual savings: 11 hours
   ```

---

## Annual Review

### Cadence: December (or annually)

**Tasks**:

1. **Calculate actual annual impact**
   ```
   2026 Mold Linker Impact
   ═════════════════════════════════════
   Total CI builds (all repos): ~1200
   Link time savings per build: 33s

   Total time saved: 1200 × 33s / 60 / 60 = 11 hours

   At $50/hour dev cost: $550 value
   At $100/hour consulting rate: $1,100 value
   ```

2. **Compare alternatives**
   ```
   Alternative linkers evaluated:
   - GNU ld: 45-60s (baseline)
   - LLVM lld: 15-18s (slower than mold)
   - mold: 12-15s (BEST)
   - Platform-specific: ld64 (macOS), msvc (Windows)

   Decision: mold remains best choice for Linux
   ```

3. **Plan next year**
   ```
   2027 Mold Strategy:
   - Maintain current integration
   - Monitor for mold v3.x (if released)
   - Evaluate polyrepo split (if workspace grows >30 crates)
   - Consider sccache + mold combination
   ```

4. **Document in CHANGELOG**
   ```markdown
   ## 2026 Performance Improvements

   ### Mold Linker Integration
   - Integrated mold linker (March 2026)
   - 73% link time reduction (45s → 12s)
   - 11 hours saved annually
   - Zero regressions over 9 months
   - Stable, production-ready
   ```

---

## Alerting Rules

### Rule 1: Link Time Regression (Real-Time)

**Trigger**: Link time > 13.3s (10% above target)

**Action**:
- [ ] Fail GitHub Actions job
- [ ] Post warning in PR comment
- [ ] Tag @team-lead

**Investigation**:
```bash
# Check workspace changes
git log --oneline -10

# Verify mold is active
cargo build -vv 2>&1 | grep mold

# Check mold version
mold --version

# Check disk space
df -h
```

### Rule 2: Improvement Below Target (Real-Time)

**Trigger**: Improvement < 60%

**Action**:
- [ ] Fail GitHub Actions job
- [ ] Post error in PR comment
- [ ] Block merge until fixed

**Investigation**:
```bash
# Verify rustflags
grep rustflags .cargo/config.toml

# Test with verbose output
cargo build -vv 2>&1 | head -100 | grep -i "linker\|mold"

# Ensure mold is installed
which mold && mold --version
```

### Rule 3: Linker Errors (Real-Time)

**Trigger**: Build fails with linker error

**Action**:
- [ ] Fail GitHub Actions job
- [ ] Capture full error output
- [ ] Create issue if mold bug suspected

**Investigation**:
```bash
# Fallback to default linker
RUSTFLAGS="" cargo build --release

# If succeeds, mold is cause
# If fails, workspace issue
```

### Rule 4: Version Alert (Monthly)

**Trigger**: mold version > 1 minor behind latest

**Action**:
- [ ] Create issue: "Update mold to vX.Y.Z"
- [ ] Test on branch
- [ ] Schedule upgrade

---

## Maintenance Tasks

### Weekly
- [ ] Review GitHub Actions benchmark results
- [ ] Check for linker errors in CI logs
- [ ] Spot-check PR comments for results

### Monthly
- [ ] Aggregate weekly metrics into monthly report
- [ ] Review mold GitHub releases page
- [ ] Check for security advisories
- [ ] Share summary with team

### Quarterly
- [ ] Trend analysis (last 3 months)
- [ ] Ecosystem evaluation (community reports)
- [ ] Version upgrade decision
- [ ] Update ADR if decisions change

### Annually
- [ ] Impact calculation (time saved, value)
- [ ] Alternative linker re-evaluation
- [ ] Strategic planning for next year
- [ ] CHANGELOG update

---

## Dashboard & Reporting

### GitHub Actions Artifacts

**Location**: GitHub Actions → Benchmark job → Artifacts

**Artifact**: `benchmark-metrics.json` (90-day retention)

**Contents**:
```json
{
  "timestamp": "2026-03-31T12:00:00Z",
  "baseline_time_s": 45.2,
  "mold_times": [12.3, 12.1, 12.0],
  "mold_average_s": 12.1,
  "improvement_percent": 73.2,
  "variance_s": 0.15,
  "status": "success"
}
```

### Manual Reporting

**Monthly Report Template**:
```markdown
# Mold Linker - Monthly Report (YYYY-MM)

## Metrics Summary
- Total CI runs: XX
- Successful builds: XX (YY%)
- Average link time: XX.Xs ± X.Xs
- Average improvement: YY% ± Z%
- Regressions: 0

## mold Status
- Current version: vX.Y.Z
- Latest available: vX.Y.Z
- Update needed: Yes/No

## Team Summary
- Status: ✓ STABLE / ⚠️ NEEDS ATTENTION / ✗ FAILED
- Action items: [list]
- Next review: [date]
```

---

## Escalation Procedure

### If Monitoring Detects Issue

**Level 1: Variance Detection** (<5% above target)
- [ ] Investigate workspace changes
- [ ] Monitor next 2-3 builds
- [ ] Document findings

**Level 2: Regression Detection** (5-10% above target)
- [ ] Investigate immediately
- [ ] Consider temporary rollback
- [ ] File issue if mold-related

**Level 3: Failure** (Build fails with mold)
- [ ] Rollback immediately (Procedure 1: Comment out)
- [ ] Investigate root cause
- [ ] File bug with mold project if needed
- [ ] Plan fix before re-enabling

---

## WP2.5 Part 3 Status: COMPLETE ✓

**Deliverable**: Comprehensive monitoring strategy ✓

**Key Components**:
- Real-time CI alerts
- Weekly review cadence
- Monthly aggregation
- Quarterly evaluation
- Annual impact assessment
- Escalation procedures
- Alert rules
- Maintenance tasks

**Next**: CHANGELOG update (final WP2.5 task)
