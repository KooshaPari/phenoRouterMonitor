# phenoSDK: Extract pheno-llm Package + PyO3 Rust Layer

## Problem
pheno/llm contains high-value, CPU-intensive components (ensemble routing 884 LOC, context folding 489 LOC) embedded in a monolith. These should be extracted as pheno-llm with optional Rust acceleration via PyO3.

## Acceptance Criteria
- [ ] pheno-llm: new package with routing/, optimization/, protocol/ modules
- [ ] pheno-llm depends on pheno-core only
- [ ] Python purity: fully functional Python implementation first
- [ ] PyO3 optional feature: pheno-llm-rs crate accelerates ensemble routing + context folding
- [ ] Benchmark comparison: Python vs PyO3 for routing (target: 10x speedup)
- [ ] Published to Phenotype GitHub Packages

## PyO3 target functions
- ensemble_router.route() — voting/scoring hot loop
- context_folder.fold() — tokenizer string manipulation
- Expose as: from pheno_llm_rs import route, fold (optional dep)
