# pheno-cli Implementation Plan

## Overview

`pheno-cli` is the unified CLI for the Phenotype platform.

## Phases

### Phase 1: Core CLI (Weeks 1-2)
- [ ] Initialize Cobra CLI
- [ ] Add configuration management
- [ ] Implement output formatters (JSON, YAML, table)
- [ ] Add shell completion

### Phase 2: Agent Commands (Weeks 3-4)
- [ ] Implement agent create/list/get/delete
- [ ] Add agent logs command
- [ ] Implement agent events streaming

### Phase 3: Task Commands (Weeks 5-6)
- [ ] Implement task submit/list/get
- [ ] Add task cancel/retry
- [ ] Implement task logs

### Phase 4: Skill Commands (Weeks 7-8)
- [ ] Implement skill list/get
- [ ] Add skill invoke
- [ ] Implement skill reload

### Phase 5: Environment Commands (Weeks 9-10)
- [ ] Implement env create/list/delete
- [ ] Add env info/logs

### Phase 6: Interactive Mode (Weeks 11-12)
- [ ] Implement REPL
- [ ] Add auto-completion
- [ ] Implement history

## Resource Estimate

1 engineer, 3 months
