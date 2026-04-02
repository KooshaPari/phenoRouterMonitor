---
audience: [developers, agents]
---

# Testing Strategy

No tests or builds were run because direct recovery was intentionally deferred until the dirty
state is preserved.

## Follow-on Verification

After preservation and restore:

```bash
npm install
npm start
```

Then verify the static asset tree and rendered site behavior.
