# Python Configuration Standards

This document defines the standard configuration management approach for all Python projects using **pydantic-settings v2.x**.

## Overview

Pydantic Settings provides:
- Type-safe configuration management with automatic validation
- Environment variable loading with optional `.env` file support
- Composite settings with inheritance and field validation
- Field descriptions and defaults
- Integration with environment-specific configurations

## Configuration Organization

### Directory Structure

\`\`\`
src/
├── config/
│   ├── __init__.py           # Export settings classes
│   ├── base.py               # BaseSettings with common config
│   ├── core.py               # Core application settings
│   ├── database.py           # Database-specific settings
│   └── services.py           # External service settings
├── settings.py               # Main settings import
└── .env.example              # Example environment file
\`\`\`

### Environment Variables Convention

Use consistent prefixes per project:
- \`THGENT_*\` for thegent project
- \`AGILEPLUS_*\` for AgilePlus project
- \`PHENCH_*\` for phench project
- \`PROJECT_ABBREV_*\` for new projects

## Core Patterns

### 1. Base Settings Configuration

\`\`\`python
# src/config/base.py
from pydantic_settings import BaseSettings, SettingsConfigDict
from pydantic import Field, field_validator
from typing import Optional
from pathlib import Path

class Settings(BaseSettings):
    """Base settings with common configuration."""

    model_config = SettingsConfigDict(
        env_file='.env',
        env_file_encoding='utf-8',
        case_sensitive=False,
        frozen=True,  # Immutable in production
        validate_default=True,
        extra='ignore',  # Ignore unknown fields
    )

    # Application info
    app_name: str = Field(default='MyApp', description='Application name')
    app_version: str = Field(default='0.1.0', description='Application version')
    environment: str = Field(default='development', description='Environment: development, staging, production')
    debug: bool = Field(default=False, description='Enable debug mode')

    # Paths
    base_dir: Path = Field(default=Path(__file__).parent.parent.parent, description='Base directory')
    log_dir: Path = Field(default_factory=lambda: Path.cwd() / 'logs', description='Logs directory')

    @field_validator('environment')
    @classmethod
    def validate_environment(cls, v: str) -> str:
        """Validate environment is one of allowed values."""
        valid_envs = {'development', 'staging', 'production'}
        if v.lower() not in valid_envs:
            raise ValueError(f'environment must be one of {valid_envs}')
        return v.lower()
\`\`\`

### 2. Composite Settings Pattern

\`\`\`python
# src/config/core.py
from pydantic_settings import BaseSettings, SettingsConfigDict
from pydantic import Field, field_validator, ConfigDict
from typing import Optional
from datetime import timedelta

class DatabaseConfig(BaseSettings):
    """Database configuration."""

    model_config = SettingsConfigDict(env_prefix='THGENT_DB_')

    url: str = Field(..., description='Database URL')
    pool_size: int = Field(default=10, ge=1, le=100, description='Connection pool size')
    max_overflow: int = Field(default=20, ge=0, description='Maximum overflow connections')
    timeout: int = Field(default=30, gt=0, description='Connection timeout in seconds')
    echo: bool = Field(default=False, description='Log SQL statements')

    @field_validator('url')
    @classmethod
    def validate_db_url(cls, v: str) -> str:
        """Validate database URL format."""
        if not (v.startswith('postgresql://') or v.startswith('sqlite://')):
            raise ValueError('Database URL must start with postgresql:// or sqlite://')
        return v

class RedisConfig(BaseSettings):
    """Redis configuration."""

    model_config = SettingsConfigDict(env_prefix='THGENT_REDIS_')

    host: str = Field(default='localhost', description='Redis host')
    port: int = Field(default=6379, ge=1, le=65535, description='Redis port')
    db: int = Field(default=0, ge=0, le=15, description='Redis database number')
    password: Optional[str] = Field(default=None, description='Redis password')
    ttl: int = Field(default=3600, gt=0, description='Default TTL in seconds')

    @property
    def url(self) -> str:
        """Generate Redis URL."""
        auth = f':{self.password}@' if self.password else ''
        return f'redis://{auth}{self.host}:{self.port}/{self.db}'

class LoggingConfig(BaseSettings):
    """Logging configuration."""

    model_config = SettingsConfigDict(env_prefix='THGENT_LOG_')

    level: str = Field(default='INFO', description='Log level')
    format: str = Field(
        default='%(asctime)s - %(name)s - %(levelname)s - %(message)s',
        description='Log format'
    )
    file: Optional[str] = Field(default=None, description='Log file path')

    @field_validator('level')
    @classmethod
    def validate_log_level(cls, v: str) -> str:
        """Validate log level."""
        valid_levels = {'DEBUG', 'INFO', 'WARNING', 'ERROR', 'CRITICAL'}
        if v.upper() not in valid_levels:
            raise ValueError(f'log level must be one of {valid_levels}')
        return v.upper()

class AppConfig(BaseSettings):
    """Main application configuration."""

    model_config = SettingsConfigDict(
        env_prefix='THGENT_',
        env_file='.env',
        env_file_encoding='utf-8',
        case_sensitive=False,
        frozen=True,
    )

    # Basic info
    app_name: str = Field(default='thegent', description='Application name')
    version: str = Field(default='0.1.0', description='Application version')
    environment: str = Field(default='development', description='Deployment environment')
    debug: bool = Field(default=False, description='Enable debug mode')

    # Nested configurations
    database: DatabaseConfig = Field(default_factory=DatabaseConfig)
    redis: RedisConfig = Field(default_factory=RedisConfig)
    logging: LoggingConfig = Field(default_factory=LoggingConfig)

    # Security
    secret_key: str = Field(..., description='Secret key for signing')
    allowed_hosts: list[str] = Field(default=['localhost', '127.0.0.1'], description='Allowed hosts')
    cors_origins: list[str] = Field(default=['http://localhost:3000'], description='CORS origins')

    # Feature flags
    enable_metrics: bool = Field(default=True, description='Enable Prometheus metrics')
    enable_tracing: bool = Field(default=False, description='Enable distributed tracing')

    @field_validator('secret_key')
    @classmethod
    def validate_secret_key(cls, v: str) -> str:
        """Validate secret key is not default."""
        if v == 'change-me-in-production':
            raise ValueError('secret_key must be changed for production')
        return v
\`\`\`

## Best Practices

### 1. Never Hardcode Secrets

\`\`\`python
# Good: Use environment variables
class Config(BaseSettings):
    api_key: str = Field(..., description='API key from environment')

    model_config = SettingsConfigDict(env_prefix='MYAPP_')

# Bad: Hardcoded secrets
class Config(BaseSettings):
    api_key: str = 'sk-1234567890'  # Don't do this!
\`\`\`

### 2. Validate Configuration on Startup

\`\`\`python
# Good: Validate during initialization
def startup_config():
    """Validate configuration at startup."""
    settings = get_settings()

    # Validate critical settings
    if not settings.secret_key or len(settings.secret_key) < 32:
        raise ValueError('secret_key must be at least 32 characters')

    if settings.environment == 'production' and settings.debug:
        raise ValueError('debug must be False in production')

    return settings
\`\`\`

## Dependencies

\`\`\`toml
[tool.poetry.dependencies]
python = "^3.11"
pydantic = "^2.6"
pydantic-settings = "^2.1"
python-dotenv = "^1.0.0"
\`\`\`

## See Also

- [Pydantic Documentation](https://docs.pydantic.dev/)
- [Pydantic Settings](https://docs.pydantic.dev/latest/concepts/pydantic_settings/)
