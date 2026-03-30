# pheno-atoms

Core atom types and validation for the Phenotype SDK.

## Overview

`pheno-atoms` provides fundamental atom types, validation rules, and registries for building resilient Phenotype infrastructure. Atoms represent immutable, typed, and validatable units that form the foundation of infrastructure descriptions.

## Features

- **Atom Types**: Simple, compound, abstract, and template atoms
- **Validation**: Pydantic-based validation with semantic versioning support
- **Registry**: Central registry for managing atom instances
- **Type Safety**: Full type hints for IDE support

## Installation

```bash
pip install pheno-atoms
```

## Quick Start

```python
from pheno_atoms import Atom, AtomType, AtomValidator, AtomRegistry

# Create an atom using the validator
validator = AtomValidator(
    name="my-infrastructure",
    description="Production infrastructure unit",
    version="1.0.0",
)

# Convert to atom
atom = validator.to_atom()

# Register atoms
registry = AtomRegistry()
registry.register(atom)

# Query atoms
all_atoms = registry.list_all()
simple_atoms = registry.filter_by_type(AtomType.SIMPLE)
```

## Testing

```bash
pip install -e ".[dev]"
pytest tests/ -v
```

## License

MIT
