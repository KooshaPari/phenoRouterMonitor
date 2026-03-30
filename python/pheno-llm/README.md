# pheno-llm

LLM integration and model selection for the Phenotype SDK.

## Overview

`pheno-llm` provides abstractions for working with various language models, model selection strategies, and LLM integrations. It supports multiple providers (OpenAI, Anthropic, Hugging Face, local models) with a unified interface.

## Features

- **Model Providers**: Support for OpenAI, Anthropic, Hugging Face, and local models
- **Model Selection**: Intelligent model selection by capability, provider, or speed
- **Configuration**: Type-safe model configuration with Pydantic
- **Client Factory**: Factory pattern for creating provider-specific clients
- **Extensible**: Easy to add new providers and clients

## Installation

```bash
pip install pheno-llm
```

## Quick Start

```python
from pheno_llm import ModelConfig, ModelProvider, ModelSelector, ClientFactory

# Create model config
config = ModelConfig(
    name="gpt-4",
    provider=ModelProvider.OPENAI,
    api_key="sk-...",
    max_tokens=4096,
    temperature=0.7,
)

# Create client
client = ClientFactory.create_client(config)

# Use model selector
selector = ModelSelector()
selector.register_model(config)
fastest = selector.select_fastest()
capable = selector.select_by_capability(8192)
```

## Testing

```bash
pip install -e ".[dev]"
pytest tests/ -v
```

## License

MIT
