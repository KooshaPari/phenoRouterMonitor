/**
 * Tests for configuration validation.
 *
 * Demonstrates how to test Zod configuration schemas.
 */

import {
  AppConfigSchema,
  ServerConfigSchema,
  DatabaseConfigSchema,
  TlsConfigSchema,
  LoggingConfigSchema,
  FeatureFlagsSchema,
  createAppConfig,
  safeParseConfig,
  loadConfigFromObject,
  loadConfigFromEnv,
  EnvironmentSchema,
  z,
} from '../src/config';

describe('ServerConfig', () => {
  test('valid server config', () => {
    const config = ServerConfigSchema.parse({ host: 'localhost', port: 8080 });
    expect(config.host).toBe('localhost');
    expect(config.port).toBe(8080);
    expect(config.tls).toBeUndefined();
  });

  test('default values', () => {
    const config = ServerConfigSchema.parse({});
    expect(config.host).toBe('localhost');
    expect(config.port).toBe(8080);
  });

  test('invalid port too high', () => {
    expect(() => {
      ServerConfigSchema.parse({ host: 'localhost', port: 70000 });
    }).toThrow(z.ZodError);
  });

  test('invalid port too low', () => {
    expect(() => {
      ServerConfigSchema.parse({ host: 'localhost', port: 0 });
    }).toThrow(z.ZodError);
  });

  test('invalid host empty', () => {
    expect(() => {
      ServerConfigSchema.parse({ host: '', port: 8080 });
    }).toThrow(z.ZodError);
  });

  test('invalid host whitespace', () => {
    expect(() => {
      ServerConfigSchema.parse({ host: '   ', port: 8080 });
    }).toThrow(z.ZodError);
  });

  test('host with invalid characters', () => {
    expect(() => {
      ServerConfigSchema.parse({ host: 'host@invalid', port: 8080 });
    }).toThrow(z.ZodError);
  });

  test('with tls', () => {
    const config = ServerConfigSchema.parse({
      host: 'example.com',
      port: 443,
      tls: { cert: '/path/to/cert.pem', key: '/path/to/key.pem' },
    });
    expect(config.tls).toBeDefined();
    expect(config.tls?.cert).toBe('/path/to/cert.pem');
  });
});

describe('DatabaseConfig', () => {
  test('valid postgres url', () => {
    const config = DatabaseConfigSchema.parse({
      url: 'postgres://localhost/mydb',
      poolSize: 10,
      timeout: 30,
    });
    expect(config.url).toBe('postgres://localhost/mydb');
    expect(config.poolSize).toBe(10);
    expect(config.timeout).toBe(30);
  });

  test('valid mysql url', () => {
    const config = DatabaseConfigSchema.parse({
      url: 'mysql://localhost/mydb',
      poolSize: 5,
    });
    expect(config.url).toBe('mysql://localhost/mydb');
  });

  test('valid sqlite url', () => {
    const config = DatabaseConfigSchema.parse({
      url: 'sqlite:///path/to/db.sqlite',
    });
    expect(config.url).toBe('sqlite:///path/to/db.sqlite');
  });

  test('valid postgresql url', () => {
    const config = DatabaseConfigSchema.parse({
      url: 'postgresql://localhost/mydb',
    });
    expect(config.url).toBe('postgresql://localhost/mydb');
  });

  test('invalid url scheme', () => {
    expect(() => {
      DatabaseConfigSchema.parse({
        url: 'invalid://localhost/mydb',
        poolSize: 10,
      });
    }).toThrow(z.ZodError);
  });

  test('invalid pool size too high', () => {
    expect(() => {
      DatabaseConfigSchema.parse({
        url: 'postgres://localhost/mydb',
        poolSize: 200,
      });
    }).toThrow(z.ZodError);
  });

  test('invalid pool size too low', () => {
    expect(() => {
      DatabaseConfigSchema.parse({
        url: 'postgres://localhost/mydb',
        poolSize: 0,
      });
    }).toThrow(z.ZodError);
  });

  test('invalid timeout too high', () => {
    expect(() => {
      DatabaseConfigSchema.parse({
        url: 'postgres://localhost/mydb',
        timeout: 400,
      });
    }).toThrow(z.ZodError);
  });

  test('default values', () => {
    const config = DatabaseConfigSchema.parse({
      url: 'postgres://localhost/mydb',
    });
    expect(config.poolSize).toBe(10);
    expect(config.timeout).toBe(30);
  });
});

describe('TlsConfig', () => {
  test('valid tls config', () => {
    const config = TlsConfigSchema.parse({
      cert: '/path/to/cert.pem',
      key: '/path/to/key.pem',
    });
    expect(config.cert).toBe('/path/to/cert.pem');
    expect(config.key).toBe('/path/to/key.pem');
  });

  test('empty cert path', () => {
    expect(() => {
      TlsConfigSchema.parse({ cert: '', key: '/path/to/key.pem' });
    }).toThrow(z.ZodError);
  });

  test('empty key path', () => {
    expect(() => {
      TlsConfigSchema.parse({ cert: '/path/to/cert.pem', key: '' });
    }).toThrow(z.ZodError);
  });
});

describe('LoggingConfig', () => {
  test('valid log levels', () => {
    const levels = ['debug', 'info', 'warning', 'error', 'critical'] as const;
    levels.forEach((level) => {
      const config = LoggingConfigSchema.parse({ level });
      expect(config.level).toBe(level);
    });
  });

  test('invalid log level', () => {
    expect(() => {
      LoggingConfigSchema.parse({ level: 'invalid' });
    }).toThrow(z.ZodError);
  });

  test('default values', () => {
    const config = LoggingConfigSchema.parse({});
    expect(config.level).toBe('info');
    expect(config.format).toContain('%(asctime)s');
    expect(config.file).toBeUndefined();
  });

  test('with log file', () => {
    const config = LoggingConfigSchema.parse({ file: '/var/log/app.log' });
    expect(config.file).toBe('/var/log/app.log');
  });
});

describe('FeatureFlags', () => {
  test('default flags', () => {
    const config = FeatureFlagsSchema.parse({});
    expect(config.enableCache).toBe(true);
    expect(config.enableMetrics).toBe(false);
    expect(config.enableProfiling).toBe(false);
    expect(config.experimentalFeatures).toEqual([]);
  });

  test('custom flags', () => {
    const config = FeatureFlagsSchema.parse({
      enableCache: false,
      enableMetrics: true,
      enableProfiling: true,
      experimentalFeatures: ['beta-feature', 'alpha-api'],
    });
    expect(config.enableCache).toBe(false);
    expect(config.enableMetrics).toBe(true);
    expect(config.enableProfiling).toBe(true);
    expect(config.experimentalFeatures).toHaveLength(2);
  });
});

describe('AppConfig', () => {
  test('valid config', () => {
    const config = AppConfigSchema.parse({
      name: 'my-app',
      version: '1.0.0',
      server: { host: 'localhost', port: 8080 },
      database: {
        url: 'postgres://localhost/mydb',
        poolSize: 10,
        timeout: 30,
      },
    });
    expect(config.name).toBe('my-app');
    expect(config.version).toBe('1.0.0');
    expect(config.environment).toBe('development');
  });

  test('invalid name empty', () => {
    expect(() => {
      AppConfigSchema.parse({
        name: '',
        version: '1.0.0',
        server: { host: 'localhost', port: 8080 },
        database: { url: 'postgres://localhost/mydb' },
      });
    }).toThrow(z.ZodError);
  });

  test('invalid name invalid chars', () => {
    expect(() => {
      AppConfigSchema.parse({
        name: 'my app!',  // Invalid: space and !
        version: '1.0.0',
        server: { host: 'localhost', port: 8080 },
        database: { url: 'postgres://localhost/mydb' },
      });
    }).toThrow(z.ZodError);
  });

  test('invalid version format', () => {
    expect(() => {
      AppConfigSchema.parse({
        name: 'my-app',
        version: 'not-a-version',
        server: { host: 'localhost', port: 8080 },
        database: { url: 'postgres://localhost/mydb' },
      });
    }).toThrow(z.ZodError);
  });

  test('version with prerelease', () => {
    const config = AppConfigSchema.parse({
      name: 'my-app',
      version: '1.0.0-beta.1',
      server: { host: 'localhost', port: 8080 },
      database: { url: 'postgres://localhost/mydb' },
    });
    expect(config.version).toBe('1.0.0-beta.1');
  });

  test('invalid environment', () => {
    expect(() => {
      AppConfigSchema.parse({
        name: 'my-app',
        version: '1.0.0',
        environment: 'invalid-env' as any,
        server: { host: 'localhost', port: 8080 },
        database: { url: 'postgres://localhost/mydb' },
      });
    }).toThrow(z.ZodError);
  });

  test('nested validation error', () => {
    expect(() => {
      AppConfigSchema.parse({
        name: 'my-app',
        version: '1.0.0',
        server: { host: 'localhost', port: 70000 },  // Invalid port
        database: { url: 'postgres://localhost/mydb' },
      });
    }).toThrow(z.ZodError);
  });
});

describe('createAppConfig with methods', () => {
  test('environment detection', () => {
    const devConfig = createAppConfig({
      name: 'my-app',
      version: '1.0.0',
      environment: 'development',
      server: { host: 'localhost', port: 8080 },
      database: { url: 'postgres://localhost/mydb' },
    });
    expect(devConfig.isDevelopment()).toBe(true);
    expect(devConfig.isProduction()).toBe(false);

    const prodConfig = createAppConfig({
      name: 'my-app',
      version: '1.0.0',
      environment: 'production',
      server: { host: 'localhost', port: 8080 },
      database: { url: 'postgres://localhost/mydb' },
    });
    expect(prodConfig.isProduction()).toBe(true);
    expect(prodConfig.isDevelopment()).toBe(false);
  });

  test('getDatabaseUrl', () => {
    const config = createAppConfig({
      name: 'my-app',
      version: '1.0.0',
      server: { host: 'localhost', port: 8080 },
      database: { url: 'postgres://localhost/mydb' },
    });
    expect(config.getDatabaseUrl()).toBe('postgres://localhost/mydb');
  });
});

describe('safeParseConfig', () => {
  test('successful parse', () => {
    const result = safeParseConfig({
      name: 'my-app',
      version: '1.0.0',
      server: { host: 'localhost', port: 8080 },
      database: { url: 'postgres://localhost/mydb' },
    });

    expect(result.success).toBe(true);
    if (result.success) {
      expect(result.data.name).toBe('my-app');
      expect(result.data.isDevelopment()).toBe(true);
    }
  });

  test('failed parse', () => {
    const result = safeParseConfig({
      name: '',  // Invalid
      version: '1.0.0',
      database: { url: 'invalid://localhost/mydb' },  // Invalid URL
    });

    expect(result.success).toBe(false);
    if (!result.success) {
      expect(result.error.errors.length).toBeGreaterThan(0);
    }
  });
});

describe('loadConfigFromObject', () => {
  test('load valid config', () => {
    const obj = {
      name: 'my-app',
      version: '1.0.0',
      server: { host: 'localhost', port: 8080 },
      database: { url: 'postgres://localhost/mydb', poolSize: 10 },
    };

    const config = loadConfigFromObject(obj);
    expect(config.name).toBe('my-app');
    expect(config.server.port).toBe(8080);
    expect(config.database.poolSize).toBe(10);
  });

  test('load invalid config throws', () => {
    const obj = {
      name: '',  // Invalid
      version: '1.0.0',
      database: { url: 'postgres://localhost/mydb' },
    };

    expect(() => {
      loadConfigFromObject(obj);
    }).toThrow(z.ZodError);
  });
});

describe('EnvironmentSchema', () => {
  test('valid environment variables', () => {
    const env = {
      APP_NAME: 'test-app',
      APP_VERSION: '1.0.0',
      APP_ENVIRONMENT: 'production',
      SERVER_HOST: '0.0.0.0',
      SERVER_PORT: '3000',
      DATABASE_URL: 'postgres://dbhost/mydb',
      DATABASE_POOL_SIZE: '20',
      DATABASE_TIMEOUT: '60',
      LOG_LEVEL: 'debug',
      ENABLE_CACHE: 'false',
      ENABLE_METRICS: 'true',
    };

    const config = EnvironmentSchema.parse(env);
    expect(config.APP_NAME).toBe('test-app');
    expect(config.SERVER_PORT).toBe(3000);
    expect(config.ENABLE_CACHE).toBe(false);
    expect(config.ENABLE_METRICS).toBe(true);
  });

  test('default values', () => {
    const env = {
      DATABASE_URL: 'postgres://localhost/mydb',
    };

    const config = EnvironmentSchema.parse(env);
    expect(config.APP_NAME).toBe('my-app');
    expect(config.APP_VERSION).toBe('1.0.0');
    expect(config.SERVER_HOST).toBe('localhost');
    expect(config.SERVER_PORT).toBe(8080);
    expect(config.ENABLE_CACHE).toBe(true);
  });
});

describe('loadConfigFromEnv', () => {
  const originalEnv = process.env;

  beforeEach(() => {
    // Reset process.env before each test
    process.env = { ...originalEnv };
  });

  afterAll(() => {
    process.env = originalEnv;
  });

  test('load from environment variables', () => {
    process.env.DATABASE_URL = 'postgres://envhost/mydb';
    process.env.APP_NAME = 'env-app';
    process.env.SERVER_PORT = '9090';

    const config = loadConfigFromEnv();
    expect(config.name).toBe('env-app');
    expect(config.server.port).toBe(9090);
    expect(config.database.url).toBe('postgres://envhost/mydb');
  });
});

describe('Config serialization', () => {
  test('config to JSON and back', () => {
    const original = createAppConfig({
      name: 'my-app',
      version: '1.0.0',
      server: { host: 'localhost', port: 8080 },
      database: { url: 'postgres://localhost/mydb' },
    });

    const json = JSON.stringify(original);
    const parsed = JSON.parse(json);

    // Re-validate after parsing
    const restored = AppConfigSchema.parse(parsed);
    expect(restored.name).toBe(original.name);
    expect(restored.version).toBe(original.version);
    expect(restored.server.host).toBe(original.server.host);
    expect(restored.database.url).toBe(original.database.url);
  });

  test('roundtrip through parse', () => {
    const original = {
      name: 'my-app',
      version: '1.0.0',
      server: { host: 'localhost', port: 8080 },
      database: { url: 'postgres://localhost/mydb' },
    };

    const parsed = AppConfigSchema.parse(original);
    const serialized = JSON.parse(JSON.stringify(parsed));
    const reparsed = AppConfigSchema.parse(serialized);

    expect(reparsed).toEqual(parsed);
  });
});
