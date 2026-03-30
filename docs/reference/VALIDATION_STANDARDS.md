# TypeScript Validation Standards

This document defines the standard validation approach for all TypeScript projects using **Zod** (^3.24.2+).

## Overview

Zod provides:
- Runtime validation with type-safe inference
- Composable, chainable schema builders
- Integration with form libraries (react-hook-form, formik)
- JSON Schema generation for API documentation
- Zero-dependency runtime validation

## Schema Organization

### Directory Structure

```
src/
├── schemas/
│   ├── index.ts              # Export all schemas
│   ├── user.ts               # User-related schemas
│   ├── config.ts             # Configuration schemas
│   ├── api/
│   │   ├── requests.ts       # API request payloads
│   │   └── responses.ts      # API response structures
│   └── forms/
│       ├── login.ts          # Login form schemas
│       └── profile.ts        # Profile form schemas
```

### Schema Naming Convention

- **Request schemas**: `Create<Entity>RequestSchema`, `Update<Entity>RequestSchema`
- **Response schemas**: `<Entity>ResponseSchema`
- **Form schemas**: `<FormName>FormSchema`
- **Type exports**: Use `z.infer<typeof schema>` for TypeScript types

## Core Patterns

### 1. Basic Schema Definition

```typescript
// src/schemas/user.ts
import { z } from 'zod';

export const UserSchema = z.object({
  id: z.string().uuid(),
  email: z.string().email('Invalid email format'),
  name: z.string().min(1, 'Name is required').max(100),
  age: z.number().int().min(0).max(150).optional(),
  createdAt: z.date(),
  status: z.enum(['active', 'inactive', 'suspended']),
});

// Type inference
export type User = z.infer<typeof UserSchema>;
```

### 2. Composition and Reuse

```typescript
// Extend existing schemas
export const UserWithPasswordSchema = UserSchema.extend({
  passwordHash: z.string().min(60),
  passwordChangedAt: z.date(),
});

// Merge multiple schemas
export const UserProfileSchema = UserSchema.merge(
  z.object({
    bio: z.string().optional(),
    avatar: z.string().url().optional(),
  })
);

// Omit fields
export const UserPublicSchema = UserSchema.omit({
  passwordHash: true,
});
```

### 3. Custom Validators

```typescript
// Inline custom validation
export const PasswordSchema = z
  .string()
  .min(8, 'Password must be at least 8 characters')
  .regex(/[A-Z]/, 'Must contain uppercase letter')
  .regex(/[0-9]/, 'Must contain number')
  .regex(/[^a-zA-Z0-9]/, 'Must contain special character');

// Refine for complex logic
export const CreateUserSchema = z.object({
  email: z.string().email(),
  password: PasswordSchema,
  confirmPassword: z.string(),
}).refine(
  (data) => data.password === data.confirmPassword,
  {
    message: 'Passwords do not match',
    path: ['confirmPassword'], // Sets which field has the error
  }
);
```

### 4. API Request/Response Validation

```typescript
// src/schemas/api/requests.ts
export const CreateUserRequestSchema = z.object({
  email: z.string().email(),
  name: z.string().min(1).max(100),
});

// src/schemas/api/responses.ts
export const UserResponseSchema = z.object({
  id: z.string().uuid(),
  email: z.string().email(),
  name: z.string(),
  createdAt: z.string().datetime(),
});

// API Handler Type
type CreateUserRequest = z.infer<typeof CreateUserRequestSchema>;
type UserResponse = z.infer<typeof UserResponseSchema>;

export async function createUser(req: CreateUserRequest): Promise<UserResponse> {
  // Handler implementation
}
```

### 5. React Hook Form Integration

```typescript
// src/schemas/forms/login.ts
import { z } from 'zod';

export const LoginFormSchema = z.object({
  email: z.string().email('Invalid email'),
  password: z.string().min(6, 'Password too short'),
  rememberMe: z.boolean().default(false),
});

export type LoginFormData = z.infer<typeof LoginFormSchema>;

// In React component
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';

function LoginForm() {
  const form = useForm<LoginFormData>({
    resolver: zodResolver(LoginFormSchema),
    defaultValues: {
      email: '',
      password: '',
      rememberMe: false,
    },
  });

  return (
    <form onSubmit={form.handleSubmit(async (data) => {
      // Type-safe data here
      const result = LoginFormSchema.safeParse(data);
      if (!result.success) {
        // Handle validation error
      }
      // Submit to API
    })}>
      {/* Form fields */}
    </form>
  );
}
```

### 6. Runtime Validation

```typescript
// Always use .safeParse() for untrusted input
function processUserData(input: unknown): User | null {
  const result = UserSchema.safeParse(input);

  if (!result.success) {
    console.error('Validation failed:', result.error.issues);
    // Log structured errors
    result.error.issues.forEach(issue => {
      console.error(`Field "${issue.path.join('.')}": ${issue.message}`);
    });
    return null;
  }

  return result.data; // Type-safe User object
}

// .parse() throws on validation failure (use for trusted sources)
const user = UserSchema.parse(someData); // Throws ZodError
```

### 7. JSON Schema Generation

```typescript
// src/schemas/index.ts
import { z } from 'zod';
import { zodToJsonSchema } from 'zod-to-json-schema';

export const UserSchema = z.object({
  id: z.string().uuid().describe('Unique user identifier'),
  email: z.string().email().describe('User email address'),
  name: z.string().min(1).describe('User display name'),
});

// Generate JSON Schema for OpenAPI/Swagger
const userJsonSchema = zodToJsonSchema(UserSchema, {
  target: 'openapi3',
  refStrategy: 'none',
});

// Use in API documentation
export const apiDocs = {
  paths: {
    '/users': {
      get: {
        responses: {
          '200': {
            content: {
              'application/json': {
                schema: userJsonSchema,
              },
            },
          },
        },
      },
    },
  },
};
```

### 8. Discriminated Unions (for variants)

```typescript
// Define variant types
export const SuccessResponseSchema = z.object({
  status: z.literal('success'),
  data: z.any(),
});

export const ErrorResponseSchema = z.object({
  status: z.literal('error'),
  code: z.string(),
  message: z.string(),
});

// Discriminated union by status field
export const ApiResponseSchema = z.discriminatedUnion('status', [
  SuccessResponseSchema,
  ErrorResponseSchema,
]);

// Type-safe pattern matching
type ApiResponse = z.infer<typeof ApiResponseSchema>;

function handleResponse(response: ApiResponse) {
  if (response.status === 'success') {
    // Type narrowed to SuccessResponse
    console.log(response.data);
  } else {
    // Type narrowed to ErrorResponse
    console.error(response.message);
  }
}
```

## Best Practices

### 1. Centralize Schema Definitions

```typescript
// ✅ Good: Centralized definitions
export const schemas = {
  user: UserSchema,
  config: ConfigSchema,
  api: {
    request: CreateUserRequestSchema,
    response: UserResponseSchema,
  },
};

// Use across project
import { schemas } from '@/schemas';
const user = schemas.user.parse(data);
```

### 2. Use Branded Types for Strong Typing

```typescript
const UserId = z.string().uuid().brand<'UserId'>();
export type UserId = z.infer<typeof UserId>;

// Now UserId is distinct from string
function getUserById(id: UserId) {
  // id is guaranteed to be a valid UUID
}
```

### 3. Version API Schemas

```typescript
export const UserV1Schema = z.object({
  id: z.string(),
  email: z.string().email(),
});

export const UserV2Schema = z.object({
  id: z.string(),
  email: z.string().email(),
  createdAt: z.date(), // New field in V2
});

export const ApiRequestV1 = z.object({ version: z.literal('v1'), user: UserV1Schema });
export const ApiRequestV2 = z.object({ version: z.literal('v2'), user: UserV2Schema });
```

### 4. Handle Edge Cases

```typescript
// Handle empty strings as undefined
const OptionalStringSchema = z.string()
  .pipe(
    z.string().transform((s) => (s === '' ? undefined : s))
  );

// Coerce types for form inputs
const AgeSchema = z.coerce.number().int().min(0).max(150);

// Lazy evaluation for circular references
type User = {
  id: string;
  friends: User[];
};

const UserSchema: z.ZodType<User> = z.lazy(() =>
  z.object({
    id: z.string(),
    friends: z.array(UserSchema),
  })
);
```

### 5. Consistent Error Messages

```typescript
// Use clear, actionable error messages
export const EmailSchema = z.string()
  .email('Please provide a valid email address')
  .toLowerCase();

export const PasswordSchema = z.string()
  .min(8, 'Password must be at least 8 characters long')
  .regex(/[A-Z]/, 'Password must contain at least one uppercase letter');
```

## Testing Validation

```typescript
// tests/schemas.test.ts
import { describe, it, expect } from 'vitest';
import { UserSchema, LoginFormSchema } from '@/schemas';

describe('UserSchema', () => {
  it('accepts valid user data', () => {
    const result = UserSchema.safeParse({
      id: '550e8400-e29b-41d4-a716-446655440000',
      email: 'user@example.com',
      name: 'John Doe',
      createdAt: new Date(),
      status: 'active',
    });
    expect(result.success).toBe(true);
  });

  it('rejects invalid email', () => {
    const result = UserSchema.safeParse({
      id: '550e8400-e29b-41d4-a716-446655440000',
      email: 'not-an-email',
      name: 'John Doe',
      createdAt: new Date(),
      status: 'active',
    });
    expect(result.success).toBe(false);
    if (!result.success) {
      expect(result.error.issues[0].path).toContain('email');
    }
  });
});

describe('LoginFormSchema', () => {
  it('requires both email and password', () => {
    const result = LoginFormSchema.safeParse({
      email: '',
      password: '',
    });
    expect(result.success).toBe(false);
  });
});
```

## Dependencies

```json
{
  "dependencies": {
    "zod": "^3.24.2"
  },
  "devDependencies": {
    "@hookform/resolvers": "^3.3.4",
    "react-hook-form": "^7.50.0",
    "zod-to-json-schema": "^3.22.4"
  }
}
```

## See Also

- [Zod Documentation](https://zod.dev/)
- [react-hook-form Documentation](https://react-hook-form.com/)
- [zod-to-json-schema](https://github.com/StefanTerdell/zod-to-json-schema)
