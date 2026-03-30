# phenoSDK: Fix Critical NotImplementedError Stubs

## Problem
60+ NotImplementedError instances across phenoSDK make it unusable as a real SDK. Critical: auth playwright adapter (BROKEN), vector search client (PARTIALLY IMPLEMENTED), DB adapters (in-memory only, no real persistence).

## Acceptance Criteria
- [ ] auth/playwright_adapter.py: implement or properly abstract with clear contract
- [ ] vector/client.py: implement real vector search (qdrant or chromadb adapter)
- [ ] adapters/persistence: add SQLAlchemy adapter alongside in-memory
- [ ] All remaining NotImplementedError have TODO tracking issues or are removed
- [ ] Health report shows <10 NotImplementedError remaining
- [ ] All new adapters have integration tests

## Approach
Port-based: define real adapters implementing existing port interfaces. Do not break existing in-memory implementations (keep as test adapters).
