# Phase 4: HTTP Client Implementation

## Status: PLANNED

## Goals
1. Create phenotype-http-client-core crate
2. Unify auth patterns across clients
3. Add missing retry logic

## Key Findings
- reqwest dominant in Rust, httpx in Python
- Three different auth patterns found
- RetryPolicy exists only in codex-client
- ~145 LOC savings opportunity

## Implementation Plan
1. Create phenotype-http-client-core crate
2. Extract HttpTransport, RetryPolicy, TransportError
3. Standardize auth middleware across clients
