/**
 * Configuration with validation.
 *
 * Demonstrates using Zod for configuration validation.
 */

import { z } from 'zod';

// TLS Configuration Schema
export const TlsConfigSchema = z.object({
  cert: z.string().min(1, 'Certificate path cannot be empty'),
  key: z.string().min(1, 'Key path cannot be empty'),
});

export type TlsConfig = z.infer<typeof TlsConfigSchema>;

// Server Configuration Schema
export const ServerConfigSchema = z.object({
  host: z
    .string()
    .min(1, 'Host cannot be empty')
    .regex(/^[\w.-]+$/, 'Host contains invalid characters')
    .default('localhost'),
  port: z.number().int().min(1).max(65535).default(8080),
  tls: TlsConfigSchema.optional(),
});

export type ServerConfig = z.infer<typeof ServerConfigSchema>;

// Database Configuration Schema
export const DatabaseConfigSchema = z.object({
  url: z.string().refine(
    (val) =>
      val.startsWith('postgres://') ||
      val.startsWith('mysql://') ||
      val.startsWith('sqlite://') ||
      val.startsWith('postgresql://'),
    {
      message:
        'Invalid database URL. Must start with one of: postgres://, mysql://, sqlite://, postgresql://',
    }
  ),
  poolSize: z.number().int().min(1).max(100).default(10),
  timeout: z.number().int().min(1).max(300).default(30),
});

export type DatabaseConfig = z.infer<typeof DatabaseConfigSchema>;

// Logging Configuration Schema
export const LoggingConfigSchema = z.object({
  level: z
    .enum(['debug', 'info', 'warning', 'error', 'critical'])
    .default('info'),
  format: z
    .string()
    .default(
      '%(asctime)s - %(name)s - %(levelname)s - %(message)s'
    ),
  file: z.string().optional(),
});

export type LoggingConfig = z.infer<typeof LoggingConfigSchema>;

// Feature Flags Schema
export const FeatureFlagsSchema = z.object({
  enableCache: z.boolean().default(true),
  enableMetrics: z.boolean().default(false),
  enableProfiling: z.boolean().default(false),
  experimentalFeatures: z.array(z.string()).default([]),
});

export type FeatureFlags = z.infer<typeof FeatureFlagsSchema>;

// Environment Enum
const EnvironmentEnum = z.enum(['development', 'staging', 'production']);

// Application Configuration Schema
export const AppConfigSchema = z.object({
  name: z
    .string()
    .min(1, 'Application name cannot be empty')
    .max(100)
    .regex(
      /^[\w-]+$/,
      'Name must contain only letters, numbers, underscores, and hyphens'
    ),
  version: z
    .string()
    .regex(
      /^\d+\.\d+\.\d+(-.+)?$/,
      'Version must follow semver (e.g., 1.0.0 or 1.0.0-beta.1)'
    ),
  environment: EnvironmentEnum.default('development'),
  server: ServerConfigSchema.default({ host: 'localhost', port: 8080 }),
  database: DatabaseConfigSchema,
  logging: LoggingConfigSchema.default({ level: 'info' }),
  features: FeatureFlagsSchema.default({
    enableCache: true,
    enableMetrics: false,
    enableProfiling: false,
    experimentalFeatures: [],
  }),
});

// Export the inferred type
export type AppConfig = z.infer<typeof AppConfigSchema>;

// Extended config type with helper methods
export interface AppConfigWithMethods extends AppConfig {
  isProduction(): boolean;
  isDevelopment(): boolean;
  getDatabaseUrl(): string;
}

// Factory function to create config with methods
export function createAppConfig(data: unknown): AppConfigWithMethods {
  const config = AppConfigSchema.parse(data);

  return {
    ...config,
    isProduction(): boolean {
      return this.environment === 'production';
    },
    isDevelopment(): boolean {
      return this.environment === 'development';
    },
    getDatabaseUrl(): string {
      return this.database.url;
    },
  };
}

// Safe parsing function
export function safeParseConfig(
  data: unknown
): { success: true; data: AppConfigWithMethods } | { success: false; error: z.ZodError } {
  const result = AppConfigSchema.safeParse(data);

  if (!result.success) {
    return { success: false, error: result.error };
  }

  return { success: true, data: createAppConfig(result.data) };
}

// Load from object
export function loadConfigFromObject(obj: Record<string, unknown>): AppConfigWithMethods {
  return createAppConfig(obj);
}

// Validation utilities
export function validateServerConfig(
  data: unknown
): { success: true; data: ServerConfig } | { success: false; error: z.ZodError } {
  return ServerConfigSchema.safeParse(data);
}

export function validateDatabaseConfig(
  data: unknown
): { success: true; data: DatabaseConfig } | { success: false; error: z.ZodError } {
  return DatabaseConfigSchema.safeParse(data);
}

// Environment schema for loading from env vars
export const EnvironmentSchema = z.object({
  APP_NAME: z.string().min(1).default('my-app'),
  APP_VERSION: z.string().regex(/^\d+\.\d+\.\d+/).default('1.0.0'),
  APP_ENVIRONMENT: EnvironmentEnum.default('development'),
  SERVER_HOST: z.string().default('localhost'),
  SERVER_PORT: z.coerce.number().int().min(1).max(65535).default(8080),
  DATABASE_URL: z.string(),
  DATABASE_POOL_SIZE: z.coerce.number().int().min(1).max(100).default(10),
  DATABASE_TIMEOUT: z.coerce.number().int().min(1).max(300).default(30),
  LOG_LEVEL: z.enum(['debug', 'info', 'warning', 'error', 'critical']).default('info'),
  ENABLE_CACHE: z.coerce.boolean().default(true),
  ENABLE_METRICS: z.coerce.boolean().default(false),
});

export type EnvironmentConfig = z.infer<typeof EnvironmentSchema>;

// Load from environment variables
export function loadConfigFromEnv(): AppConfigWithMethods {
  const env = EnvironmentSchema.parse(process.env);

  return createAppConfig({
    name: env.APP_NAME,
    version: env.APP_VERSION,
    environment: env.APP_ENVIRONMENT,
    server: {
      host: env.SERVER_HOST,
      port: env.SERVER_PORT,
    },
    database: {
      url: env.DATABASE_URL,
      poolSize: env.DATABASE_POOL_SIZE,
      timeout: env.DATABASE_TIMEOUT,
    },
    logging: {
      level: env.LOG_LEVEL,
    },
    features: {
      enableCache: env.ENABLE_CACHE,
      enableMetrics: env.ENABLE_METRICS,
      enableProfiling: false,
      experimentalFeatures: [],
    },
  });
}

// Re-export Zod for convenience
export { z };

// Default export
export default {
  AppConfigSchema,
  ServerConfigSchema,
  DatabaseConfigSchema,
  LoggingConfigSchema,
  FeatureFlagsSchema,
  TlsConfigSchema,
  createAppConfig,
  safeParseConfig,
  loadConfigFromObject,
  loadConfigFromEnv,
  validateServerConfig,
  validateDatabaseConfig,
};
