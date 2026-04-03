# Validation Integration Template

This template shows how to integrate Zod for validation in TypeScript with hexagonal architecture.

## Files

- `package.json` - Dependencies (zod, jest)
- `src/config.ts` - Zod schemas
- `tests/validation.test.ts` - Validation tests

## Quick Start

1. Copy these files to your project
2. Install dependencies:
   ```bash
   npm install
   ```
3. Define your configuration schemas in `src/config.ts`
4. Add validators using Zod's validation system
5. Validate at load time and runtime

## Features

- Type-safe configuration using Zod schemas
- Custom validators for complex validation rules
- Environment variable support
- JSON serialization
- Nested schema validation
- Automatic TypeScript type inference

## Usage

### Basic Configuration

```typescript
import { AppConfig, loadConfig } from './src/config';

// Parse and validate
const config = AppConfig.parse({
  name: 'my-app',
  version: '1.0.0',
  server: {
    host: 'localhost',
    port: 8080
  },
  database: {
    url: 'postgres://localhost/mydb',
    poolSize: 10,
    timeout: 30
  }
});
```

### Safe Parsing

```typescript
const result = AppConfig.safeParse(data);

if (result.success) {
  console.log('Config loaded:', result.data);
} else {
  console.error('Validation errors:', result.error.errors);
}
```

### Validation

Validation happens automatically on schema parse:

```typescript
// This will throw ZodError
const invalid = AppConfig.parse({
  name: '',  // Empty name not allowed
  version: '1.0.0',
  server: {
    host: 'localhost',
    port: 70000  // Invalid port
  },
  database: {
    url: 'invalid://localhost',  // Invalid URL scheme
    poolSize: 10,
    timeout: 30
  }
});
```

## Hexagonal Architecture

Validation fits naturally into hexagonal architecture:

```
┌─────────────────────────────────────────┐
│           Application Layer              │
│  ┌─────────────────────────────────┐   │
│  │    Configuration (validated)    │   │
│  │      AppConfig, ServerConfig    │   │
│  └─────────────────────────────────┘   │
└─────────────────────────────────────────┘
                     │
                     ▼
┌─────────────────────────────────────────┐
│            Validation Port               │
│      ┌──────────────────────┐          │
│      │   Zod Schema Pattern │          │
│      │   (Zod schemas)    │          │
│      └──────────────────────┘          │
└─────────────────────────────────────────┘
```

## Integration with Application Services

Use configuration in your application services:

```typescript
import { AppConfig } from './config';

class ApplicationService {
  constructor(private readonly config: AppConfig) {
    // Config is already validated at this point
  }

  async startServer(): Promise<void> {
    // Use validated server config
    const host = this.config.server.host;
    const port = this.config.server.port;
    // ...
  }
}
```

## Testing

Test your configuration validation:

```typescript
import { AppConfig, ServerConfig, DatabaseConfig } from './config';

test('valid config', () => {
  const config = AppConfig.parse({
    name: 'my-app',
    version: '1.0.0',
    server: { host: 'localhost', port: 8080 },
    database: { url: 'postgres://localhost/mydb' }
  });
  expect(config.name).toBe('my-app');
});

test('invalid port', () => {
  expect(() => {
    ServerConfig.parse({ host: 'localhost', port: 70000 });
  }).toThrow();
});
```

## Environment Variables

Load from environment variables:

```typescript
import { z } from 'zod';

const EnvSchema = z.object({
  APP_NAME: z.string().default('my-app'),
  APP_DEBUG: z.coerce.boolean().default(false),
  DATABASE_URL: z.string().url(),
  PORT: z.coerce.number().min(1).max(65535).default(8080)
});

const env = EnvSchema.parse(process.env);
```

## Additional Resources

- [Zod documentation](https://zod.dev/)
- [Zod error handling](https://zod.dev/ERROR_HANDLING)
- [Hexagonal architecture](https://alistair.cockburn.us/hexagonal-architecture/)
