"""Spec Populator: Automated spec extraction from source code.

This module automatically:
1. Parses source code (Python, Go, Rust, TypeScript)
2. Extracts function/class/struct definitions with docstrings
3. Maps code entities to functional requirements
4. Generates AgilePlus YAML specs
5. Creates traceability links (FR → US → ADR → TC → CE)
"""

import os
import re
import json
import yaml
import ast
import subprocess
from pathlib import Path
from dataclasses import dataclass, asdict
from typing import List, Dict, Optional, Tuple
from collections import defaultdict


@dataclass
class CodeEntity:
    """Represents a code entity extracted from source."""
    name: str
    type: str  # 'function', 'class', 'method', 'struct', 'interface'
    file_path: str
    line_start: int
    line_end: int
    docstring: str
    signature: str
    language: str
    related_specs: List[str]
    test_file: Optional[str] = None


@dataclass
class FunctionalRequirement:
    """Represents a functional requirement."""
    id: str
    title: str
    description: str
    priority: str  # 'high', 'medium', 'low'
    status: str  # 'draft', 'approved', 'implemented', 'verified'
    acceptance_criteria: List[str]
    related_entities: List[str]
    user_story_id: Optional[str] = None
    adr_id: Optional[str] = None


@dataclass
class UserStory:
    """Represents a user story."""
    id: str
    title: str
    description: str
    acceptance_criteria: List[str]
    fr_id: Optional[str] = None


@dataclass
class ArchitectureDecision:
    """Represents an architecture decision."""
    id: str
    title: str
    context: str
    decision: str
    consequences: str
    status: str


class SourceCodeParser:
    """Parse source code files to extract entities and docstrings."""
    
    def __init__(self, repo_path: str):
        self.repo_path = Path(repo_path)
        self.entities: List[CodeEntity] = []
    
    def parse_all(self) -> List[CodeEntity]:
        """Parse all supported source files in the repository."""
        for ext, language in [
            ('*.py', 'python'),
            ('*.go', 'go'),
            ('*.rs', 'rust'),
            ('*.ts', 'typescript'),
            ('*.tsx', 'typescript'),
        ]:
            for file_path in self.repo_path.rglob(ext):
                if 'venv' in str(file_path) or 'node_modules' in str(file_path):
                    continue
                if language == 'python':
                    self._parse_python(file_path)
                elif language == 'go':
                    self._parse_go(file_path)
                elif language == 'rust':
                    self._parse_rust(file_path)
                elif language == 'typescript':
                    self._parse_typescript(file_path)
        
        return self.entities
    
    def _parse_python(self, file_path: Path):
        """Parse Python file using AST."""
        try:
            with open(file_path, 'r', encoding='utf-8') as f:
                content = f.read()
            
            tree = ast.parse(content)
            lines = content.split('\n')
            
            for node in ast.walk(tree):
                if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)):
                    docstring = ast.get_docstring(node) or ''
                    if docstring:  # Only process functions with docstrings
                        entity = CodeEntity(
                            name=node.name,
                            type='function' if isinstance(node, ast.FunctionDef) else 'async_function',
                            file_path=str(file_path),
                            line_start=node.lineno,
                            line_end=node.end_lineno if hasattr(node, 'end_lineno') else node.lineno,
                            docstring=docstring,
                            signature=self._extract_python_signature(node),
                            language='python',
                            related_specs=self._extract_trace_refs(docstring),
                            test_file=self._find_test_file(file_path, node.name)
                        )
                        self.entities.append(entity)
                
                elif isinstance(node, ast.ClassDef):
                    docstring = ast.get_docstring(node) or ''
                    if docstring:
                        entity = CodeEntity(
                            name=node.name,
                            type='class',
                            file_path=str(file_path),
                            line_start=node.lineno,
                            line_end=node.end_lineno if hasattr(node, 'end_lineno') else node.lineno,
                            docstring=docstring,
                            signature=f"class {node.name}",
                            language='python',
                            related_specs=self._extract_trace_refs(docstring),
                            test_file=self._find_test_file(file_path, node.name)
                        )
                        self.entities.append(entity)
        
        except Exception as e:
            print(f"  ⚠️  Could not parse {file_path}: {e}")
    
    def _extract_python_signature(self, node) -> str:
        """Extract function signature from AST node."""
        args = []
        for arg in node.args.args:
            arg_str = arg.arg
            if arg.annotation and hasattr(arg.annotation, 'id'):
                arg_str += f": {arg.annotation.id}"
            args.append(arg_str)
        
        if node.args.vararg:
            args.append(f"*{node.args.vararg.arg}")
        if node.args.kwarg:
            args.append(f"**{node.args.kwarg.arg}")
        
        return f"def {node.name}({', '.join(args)})"
    
    def _parse_go(self, file_path: Path):
        """Parse Go file using regex."""
        try:
            with open(file_path, 'r', encoding='utf-8') as f:
                content = f.read()
                lines = content.split('\n')
            
            # Match function declarations
            func_pattern = re.compile(r'^func\s+(\([^)]+\)\s+)?(\w+)\s*\([^)]*\)')
            
            for i, line in enumerate(lines, 1):
                match = func_pattern.match(line)
                if match:
                    func_name = match.group(2)
                    # Look for doc comment above function
                    docstring = ''
                    for j in range(max(0, i-5), i):
                        if lines[j].strip().startswith('//'):
                            docstring = lines[j].strip()[2:].strip() + '\n' + docstring
                    
                    if docstring:
                        entity = CodeEntity(
                            name=func_name,
                            type='function',
                            file_path=str(file_path),
                            line_start=i,
                            line_end=i,
                            docstring=docstring.strip(),
                            signature=line.strip(),
                            language='go',
                            related_specs=self._extract_trace_refs(docstring),
                            test_file=self._find_test_file(file_path, func_name)
                        )
                        self.entities.append(entity)
        
        except Exception as e:
            print(f"  ⚠️  Could not parse {file_path}: {e}")
    
    def _parse_rust(self, file_path: Path):
        """Parse Rust file using regex."""
        try:
            with open(file_path, 'r', encoding='utf-8') as f:
                content = f.read()
            
            # Match function declarations
            fn_pattern = re.compile(r'^\s*(pub\s+)?fn\s+(\w+)\s*<', re.MULTILINE)
            
            for match in fn_pattern.finditer(content):
                func_name = match.group(2)
                # Calculate line number
                line_num = content[:match.start()].count('\n') + 1
                
                # Look for doc comments
                doc_start = content.rfind('/**', 0, match.start())
                if doc_start == -1:
                    doc_start = content.rfind('///', 0, match.start())
                
                if doc_start != -1:
                    docstring = content[doc_start:match.start()].strip()
                    
                    entity = CodeEntity(
                        name=func_name,
                        type='function',
                        file_path=str(file_path),
                        line_start=line_num,
                        line_end=line_num,
                        docstring=docstring,
                        signature=f"fn {func_name}",
                        language='rust',
                        related_specs=self._extract_trace_refs(docstring),
                        test_file=self._find_test_file(file_path, func_name)
                    )
                    self.entities.append(entity)
        
        except Exception as e:
            print(f"  ⚠️  Could not parse {file_path}: {e}")
    
    def _parse_typescript(self, file_path: Path):
        """Parse TypeScript file using regex."""
        try:
            with open(file_path, 'r', encoding='utf-8') as f:
                content = f.read()
            
            # Match function declarations
            fn_pattern = re.compile(r'^\s*(export\s+)?(?:async\s+)?function\s+(\w+)\s*\(', re.MULTILINE)
            
            for match in fn_pattern.finditer(content):
                func_name = match.group(2)
                line_num = content[:match.start()].count('\n') + 1
                
                # Look for JSDoc comments
                doc_start = content.rfind('/**', 0, match.start())
                if doc_start != -1:
                    docstring = content[doc_start:match.start()].strip()
                    
                    entity = CodeEntity(
                        name=func_name,
                        type='function',
                        file_path=str(file_path),
                        line_start=line_num,
                        line_end=line_num,
                        docstring=docstring,
                        signature=f"function {func_name}",
                        language='typescript',
                        related_specs=self._extract_trace_refs(docstring),
                        test_file=self._find_test_file(file_path, func_name)
                    )
                    self.entities.append(entity)
        
        except Exception as e:
            print(f"  ⚠️  Could not parse {file_path}: {e}")
    
    def _extract_trace_refs(self, docstring: str) -> List[str]:
        """Extract traceability references from docstring."""
        refs = []
        patterns = [
            r'FR[-_](\d+)',  # FR-001, FR_001
            r'US[-_](\d+)',  # US-001
            r'ADR[-_](\d+)',  # ADR-001
            r'TC[-_](\d+)',  # TC-001
        ]
        
        for pattern in patterns:
            for match in re.finditer(pattern, docstring):
                refs.append(match.group(0))
        
        return refs
    
    def _find_test_file(self, source_file: Path, entity_name: str) -> Optional[str]:
        """Find the corresponding test file for a source file."""
        # Common test file patterns
        test_patterns = [
            source_file.parent / f"test_{source_file.name}",
            source_file.parent / f"{source_file.stem}_test{source_file.suffix}",
            source_file.parent / f"{source_file.stem}.test{source_file.suffix}",
            Path(str(source_file).replace('/src/', '/tests/')),
        ]
        
        for test_path in test_patterns:
            if test_path.exists():
                # Check if the test file contains tests for this entity
                try:
                    with open(test_path, 'r') as f:
                        content = f.read()
                    if entity_name in content:
                        return str(test_path)
                except:
                    pass
        
        return None


class SpecGenerator:
    """Generate AgilePlus YAML specs from code entities."""
    
    def __init__(self, repo_path: str, repo_name: str):
        self.repo_path = Path(repo_path)
        self.repo_name = repo_name
        self.specs_dir = self.repo_path / '.agileplus' / 'specs'
    
    def generate_specs(self, entities: List[CodeEntity]) -> Dict[str, any]:
        """Generate comprehensive specs from extracted entities."""
        specs = {
            'version': '1.0',
            'generated_from': f'{self.repo_name} source code',
            'functional_requirements': [],
            'user_stories': [],
            'architecture_decisions': [],
            'traceability_matrix': {}
        }
        
        # Group entities by functional area
        entity_groups = self._group_entities(entities)
        
        # Generate FRs from entity groups
        for i, (area, area_entities) in enumerate(entity_groups.items(), 1):
            fr_id = f"FR-{i:03d}"
            fr = self._create_fr(fr_id, area, area_entities)
            specs['functional_requirements'].append(asdict(fr))
            
            # Generate User Story for this FR
            us_id = f"US-{i:03d}"
            us = self._create_us(us_id, fr)
            specs['user_stories'].append(asdict(us))
            
            # Link FR to US
            fr.user_story_id = us_id
            
            # Generate Architecture Decision
            adr_id = f"ADR-{i:03d}"
            adr = self._create_adr(adr_id, area, area_entities)
            specs['architecture_decisions'].append(asdict(adr))
            
            # Link FR to ADR
            fr.adr_id = adr_id
            
            # Build traceability matrix
            specs['traceability_matrix'][fr_id] = {
                'user_story': us_id,
                'architecture_decision': adr_id,
                'test_cases': [f"TC-{i:03d}-{j:03d}" for j, _ in enumerate(area_entities[:5], 1)],
                'code_entities': [e.name for e in area_entities]
            }
        
        return specs
    
    def _group_entities(self, entities: List[CodeEntity]) -> Dict[str, List[CodeEntity]]:
        """Group entities by functional area based on file paths and names."""
        groups = defaultdict(list)
        
        for entity in entities:
            # Determine functional area from file path
            path_parts = Path(entity.file_path).parts
            
            if 'agent' in path_parts or 'agent' in entity.name.lower():
                groups['Agent Management'].append(entity)
            elif 'skill' in path_parts or 'skill' in entity.name.lower():
                groups['Skill System'].append(entity)
            elif 'task' in path_parts or 'task' in entity.name.lower():
                groups['Task Execution'].append(entity)
            elif 'llm' in path_parts or 'provider' in path_parts:
                groups['LLM Integration'].append(entity)
            elif 'tool' in path_parts or 'tool' in entity.name.lower():
                groups['Tool System'].append(entity)
            elif 'config' in path_parts:
                groups['Configuration'].append(entity)
            elif 'cli' in path_parts or 'cmd' in path_parts:
                groups['CLI Interface'].append(entity)
            else:
                groups['Core Infrastructure'].append(entity)
        
        return dict(groups)
    
    def _create_fr(self, fr_id: str, area: str, entities: List[CodeEntity]) -> FunctionalRequirement:
        """Create a functional requirement from a group of entities."""
        # Generate acceptance criteria from entity docstrings
        acceptance_criteria = []
        for e in entities[:5]:  # Top 5 entities
            # Extract first sentence of docstring as acceptance criterion
            first_sentence = e.docstring.split('.')[0] if e.docstring else f"{e.name} works correctly"
            acceptance_criteria.append(f"{e.name}: {first_sentence}")
        
        return FunctionalRequirement(
            id=fr_id,
            title=f"{area} Capability",
            description=f"The system shall provide {area.lower()} functionality through {len(entities)} code entities.",
            priority='high' if len(entities) > 5 else 'medium',
            status='implemented' if all(e.test_file for e in entities[:3]) else 'partial',
            acceptance_criteria=acceptance_criteria,
            related_entities=[e.name for e in entities],
            user_story_id=None,
            adr_id=None
        )
    
    def _create_us(self, us_id: str, fr: FunctionalRequirement) -> UserStory:
        """Create a user story from a functional requirement."""
        # Map FR titles to user story formats
        user_types = {
            'Agent': 'As a developer',
            'Skill': 'As a user',
            'Task': 'As an operator',
            'LLM': 'As an AI engineer',
            'Tool': 'As a power user',
            'Config': 'As a DevOps engineer',
            'CLI': 'As a CLI user',
        }
        
        user_type = 'As a user'
        for key, value in user_types.items():
            if key in fr.title:
                user_type = value
                break
        
        return UserStory(
            id=us_id,
            title=f"{fr.title} Story",
            description=f"{user_type}, I want {fr.description.lower()} so that I can be productive.",
            acceptance_criteria=fr.acceptance_criteria,
            fr_id=fr.id
        )
    
    def _create_adr(self, adr_id: str, area: str, entities: List[CodeEntity]) -> ArchitectureDecision:
        """Create an architecture decision from entity analysis."""
        # Analyze entity languages and patterns
        languages = set(e.language for e in entities)
        
        if len(languages) > 1:
            decision = f"Use polyglot architecture for {area} with {', '.join(languages)}"
            consequences = "Enables using best language for each task, but adds complexity"
        else:
            lang = list(languages)[0] if languages else 'unknown'
            decision = f"Use {lang} for {area} implementation"
            consequences = f"Consistent codebase in {lang}, easier maintenance"
        
        return ArchitectureDecision(
            id=adr_id,
            title=f"{area} Implementation Language",
            context=f"{area} requires {len(entities)} code entities with various responsibilities",
            decision=decision,
            consequences=consequences,
            status='accepted'
        )
    
    def save_specs(self, specs: Dict[str, any]):
        """Save specs to YAML files."""
        # Create specs directory
        frs_dir = self.specs_dir / 'functional-requirements'
        us_dir = self.specs_dir / 'user-stories'
        adr_dir = self.specs_dir / 'architecture-decisions'
        
        frs_dir.mkdir(parents=True, exist_ok=True)
        us_dir.mkdir(parents=True, exist_ok=True)
        adr_dir.mkdir(parents=True, exist_ok=True)
        
        # Save individual FR files
        for fr in specs['functional_requirements']:
            fr_file = frs_dir / f"{fr['id'].lower().replace('-', '_')}.yaml"
            with open(fr_file, 'w') as f:
                yaml.dump(fr, f, default_flow_style=False, sort_keys=False)
        
        # Save individual US files
        for us in specs['user_stories']:
            us_file = us_dir / f"{us['id'].lower().replace('-', '_')}.yaml"
            with open(us_file, 'w') as f:
                yaml.dump(us, f, default_flow_style=False, sort_keys=False)
        
        # Save individual ADR files
        for adr in specs['architecture_decisions']:
            adr_file = adr_dir / f"{adr['id'].lower().replace('-', '_')}.yaml"
            with open(adr_file, 'w') as f:
                yaml.dump(adr, f, default_flow_style=False, sort_keys=False)
        
        # Save traceability matrix
        matrix_file = self.specs_dir / 'traceability-matrix.yaml'
        with open(matrix_file, 'w') as f:
            yaml.dump(specs['traceability_matrix'], f, default_flow_style=False)
        
        # Save master index
        index_file = self.specs_dir / 'index.yaml'
        with open(index_file, 'w') as f:
            yaml.dump({
                'version': specs['version'],
                'generated_from': specs['generated_from'],
                'fr_count': len(specs['functional_requirements']),
                'us_count': len(specs['user_stories']),
                'adr_count': len(specs['architecture_decisions']),
            }, f, default_flow_style=False)
        
        print(f"  ✅ Saved specs to {self.specs_dir}")


def main():
    """CLI entry point."""
    import argparse
    
    parser = argparse.ArgumentParser(
        description='Populate AgilePlus specs from source code'
    )
    parser.add_argument(
        '--repo',
        default='.',
        help='Path to repository (default: current directory)'
    )
    parser.add_argument(
        '--name',
        help='Repository name (default: directory name)'
    )
    parser.add_argument(
        '--dry-run',
        action='store_true',
        help='Print what would be generated without saving'
    )
    
    args = parser.parse_args()
    
    repo_path = Path(args.repo).resolve()
    repo_name = args.name or repo_path.name
    
    print(f"🔍 Parsing source code in {repo_path}...")
    
    parser_obj = SourceCodeParser(repo_path)
    entities = parser_obj.parse_all()
    
    print(f"  📊 Found {len(entities)} code entities")
    
    if entities:
        print(f"📝 Generating specs...")
        generator = SpecGenerator(repo_path, repo_name)
        specs = generator.generate_specs(entities)
        
        print(f"  📋 Generated:")
        print(f"     • {len(specs['functional_requirements'])} Functional Requirements")
        print(f"     • {len(specs['user_stories'])} User Stories")
        print(f"     • {len(specs['architecture_decisions'])} Architecture Decisions")
        
        if not args.dry_run:
            generator.save_specs(specs)
        else:
            print(f"  ⚠️  Dry run - not saving files")
            print(f"  📄 Would save to: {generator.specs_dir}")
        
        return 0
    else:
        print(f"  ⚠️  No code entities found")
        return 1


if __name__ == '__main__':
    sys.exit(main())
