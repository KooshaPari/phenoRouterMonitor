#!/usr/bin/env python3
"""Validate Coverage: Check traceability coverage across specs, tests, and code."""

import os
import sys
import json
import yaml
import re
from pathlib import Path
from typing import Dict, List, Optional, Tuple
from dataclasses import dataclass, field
from collections import defaultdict


@dataclass
class CoverageItem:
    """Represents a traceability item."""
    id: str
    type: str  # 'FR', 'US', 'ADR', 'TC', 'CE'
    title: str
    status: str
    linked_to: List[str] = field(default_factory=list)
    links_from: List[str] = field(default_factory=list)
    coverage_percent: float = 0.0


class TraceabilityValidator:
    """Validate 5-level traceability (FR → US → ADR → TC → CE)."""
    
    def __init__(self, repo_path: str, config: Optional[Dict] = None):
        self.repo_path = Path(repo_path)
        self.config = config or self._load_config()
        self.items: Dict[str, CoverageItem] = {}
        self.missing_links: List[Tuple[str, str, str]] = []  # (from_id, to_type, to_id)
        self.coverage_gaps: List[Dict] = []
    
    def _load_config(self) -> Dict:
        """Load traceability configuration."""
        config_path = self.repo_path / '.agileplus' / 'traceability-config.json'
        if config_path.exists():
            with open(config_path) as f:
                return json.load(f)
        return {
            'coverage_requirements': {
                'FR': {'min_coverage': 100, 'required_links': ['US', 'ADR']},
                'US': {'min_coverage': 100, 'required_links': ['FR', 'TC']},
                'ADR': {'min_coverage': 100, 'required_links': ['FR', 'US']},
                'TC': {'min_coverage': 100, 'required_links': ['US', 'CE']},
                'CE': {'min_coverage': 100, 'required_links': ['TC']},
            },
            'badge_colors': {
                'green': 100,
                'yellow': 80,
                'red': 0
            }
        }
    
    def validate(self) -> Dict:
        """Run full validation and return report."""
        print(f"🔍 Validating traceability in {self.repo_path}...")
        
        # Load all items
        self._load_frs()
        self._load_uss()
        self._load_adrs()
        self._load_tcs()
        self._load_ces()
        
        # Build link graph
        self._build_link_graph()
        
        # Validate coverage
        coverage = self._validate_coverage()
        
        # Find gaps
        self._find_gaps()
        
        # Generate report
        report = {
            'repository': str(self.repo_path),
            'summary': {
                'total_items': len(self.items),
                'fr_count': len([i for i in self.items.values() if i.type == 'FR']),
                'us_count': len([i for i in self.items.values() if i.type == 'US']),
                'adr_count': len([i for i in self.items.values() if i.type == 'ADR']),
                'tc_count': len([i for i in self.items.values() if i.type == 'TC']),
                'ce_count': len([i for i in self.items.values() if i.type == 'CE']),
                'coverage_percent': coverage,
                'missing_links': len(self.missing_links),
                'gaps': len(self.coverage_gaps),
            },
            'coverage_by_type': self._coverage_by_type(),
            'missing_links': [
                {'from': ml[0], 'to_type': ml[1], 'to_id': ml[2]}
                for ml in self.missing_links
            ],
            'gaps': self.coverage_gaps,
            'badge_color': self._badge_color(coverage),
        }
        
        return report
    
    def _load_frs(self):
        """Load functional requirements."""
        fr_dir = self.repo_path / '.agileplus' / 'specs' / 'functional-requirements'
        if fr_dir.exists():
            for fr_file in fr_dir.glob('*.yaml'):
                with open(fr_file) as f:
                    data = yaml.safe_load(f)
                    if data:
                        item = CoverageItem(
                            id=data.get('id', fr_file.stem.upper().replace('_', '-')),
                            type='FR',
                            title=data.get('title', 'Unknown'),
                            status=data.get('status', 'draft'),
                            linked_to=[data.get('user_story_id'), data.get('adr_id')],
                        )
                        self.items[item.id] = item
    
    def _load_uss(self):
        """Load user stories."""
        us_dir = self.repo_path / '.agileplus' / 'specs' / 'user-stories'
        if us_dir.exists():
            for us_file in us_dir.glob('*.yaml'):
                with open(us_file) as f:
                    data = yaml.safe_load(f)
                    if data:
                        item = CoverageItem(
                            id=data.get('id', us_file.stem.upper().replace('_', '-')),
                            type='US',
                            title=data.get('title', 'Unknown'),
                            status=data.get('status', 'draft'),
                            linked_to=[data.get('fr_id')],
                        )
                        self.items[item.id] = item
    
    def _load_adrs(self):
        """Load architecture decisions."""
        adr_dir = self.repo_path / '.agileplus' / 'specs' / 'architecture-decisions'
        if adr_dir.exists():
            for adr_file in adr_dir.glob('*.yaml'):
                with open(adr_file) as f:
                    data = yaml.safe_load(f)
                    if data:
                        item = CoverageItem(
                            id=data.get('id', adr_file.stem.upper().replace('_', '-')),
                            type='ADR',
                            title=data.get('title', 'Unknown'),
                            status=data.get('status', 'draft'),
                        )
                        self.items[item.id] = item
    
    def _load_tcs(self):
        """Load test cases from test files."""
        # Look for test files
        for pattern in ['test_*.py', '*_test.py', '*.test.ts', '*_test.rs']:
            for test_file in self.repo_path.rglob(pattern):
                if 'venv' in str(test_file) or 'node_modules' in str(test_file):
                    continue
                
                try:
                    with open(test_file, 'r') as f:
                        content = f.read()
                    
                    # Extract test function names and trace refs
                    for match in re.finditer(r'def\s+(test_\w+)|it\([\'"]([\w\s]+)[\'"]', content):
                        test_name = match.group(1) or match.group(2)
                        tc_id = f"TC-{test_file.stem}-{test_name}"
                        
                        # Look for trace refs in docstring/comments
                        trace_refs = re.findall(r'FR[-_]\d+|US[-_]\d+', content[:match.start()])
                        
                        item = CoverageItem(
                            id=tc_id,
                            type='TC',
                            title=test_name,
                            status='defined',
                            linked_to=trace_refs,
                        )
                        self.items[tc_id] = item
                        
                except Exception:
                    pass
    
    def _load_ces(self):
        """Load code entities from source files."""
        for ext, language in [
            ('*.py', 'python'),
            ('*.go', 'go'),
            ('*.rs', 'rust'),
            ('*.ts', 'typescript'),
        ]:
            for src_file in self.repo_path.rglob(ext):
                if 'venv' in str(src_file) or 'node_modules' in str(src_file):
                    continue
                
                try:
                    with open(src_file, 'r') as f:
                        content = f.read()
                    
                    # Extract function/class definitions with trace refs
                    if language == 'python':
                        for match in re.finditer(r'def\s+(\w+)\s*\(', content):
                            func_name = match.group(1)
                            ce_id = f"CE-{src_file.stem}-{func_name}"
                            
                            # Look for trace refs in docstring above function
                            before_func = content[:match.start()]
                            trace_refs = re.findall(r'FR[-_]\d+|TC[-_]\d+', before_func[-500:])
                            
                            item = CoverageItem(
                                id=ce_id,
                                type='CE',
                                title=func_name,
                                status='implemented',
                                linked_to=trace_refs,
                            )
                            self.items[ce_id] = item
                            
                except Exception:
                    pass
    
    def _build_link_graph(self):
        """Build bidirectional link graph."""
        for item in self.items.values():
            for linked_id in item.linked_to:
                if linked_id and linked_id in self.items:
                    self.items[linked_id].links_from.append(item.id)
    
    def _validate_coverage(self) -> float:
        """Calculate overall coverage percentage."""
        if not self.items:
            return 0.0
        
        total_links = 0
        valid_links = 0
        
        for item in self.items.values():
            reqs = self.config.get('coverage_requirements', {}).get(item.type, {})
            required = reqs.get('required_links', [])
            
            for req_type in required:
                total_links += 1
                # Check if there's a link of the required type
                for linked_id in item.linked_to:
                    if linked_id and linked_id.startswith(req_type):
                        valid_links += 1
                        break
        
        return (valid_links / total_links * 100) if total_links > 0 else 100.0
    
    def _find_gaps(self):
        """Find coverage gaps."""
        for item in self.items.values():
            reqs = self.config.get('coverage_requirements', {}).get(item.type, {})
            required = reqs.get('required_links', [])
            
            for req_type in required:
                has_link = any(
                    linked_id and linked_id.startswith(req_type)
                    for linked_id in item.linked_to
                )
                
                if not has_link:
                    self.coverage_gaps.append({
                        'item_id': item.id,
                        'item_type': item.type,
                        'item_title': item.title,
                        'missing_link_type': req_type,
                        'severity': 'high' if item.type in ['FR', 'TC'] else 'medium',
                    })
                    self.missing_links.append((item.id, req_type, f"{req_type}-XXX"))
    
    def _coverage_by_type(self) -> Dict:
        """Get coverage breakdown by type."""
        by_type = defaultdict(lambda: {'total': 0, 'valid': 0, 'percent': 0})
        
        for item in self.items.values():
            reqs = self.config.get('coverage_requirements', {}).get(item.type, {})
            required = reqs.get('required_links', [])
            
            for req_type in required:
                by_type[item.type]['total'] += 1
                if any(linked_id and linked_id.startswith(req_type) for linked_id in item.linked_to):
                    by_type[item.type]['valid'] += 1
        
        for type_data in by_type.values():
            type_data['percent'] = (
                type_data['valid'] / type_data['total'] * 100
                if type_data['total'] > 0 else 100
            )
        
        return dict(by_type)
    
    def _badge_color(self, coverage: float) -> str:
        """Determine badge color based on coverage."""
        thresholds = self.config.get('badge_colors', {'green': 100, 'yellow': 80, 'red': 0})
        
        if coverage >= thresholds.get('green', 100):
            return 'green'
        elif coverage >= thresholds.get('yellow', 80):
            return 'yellow'
        else:
            return 'red'
    
    def print_report(self, report: Dict, format: str = 'terminal'):
        """Print validation report."""
        if format == 'terminal':
            self._print_terminal_report(report)
        elif format == 'json':
            print(json.dumps(report, indent=2))
        elif format == 'markdown':
            self._print_markdown_report(report)
    
    def _print_terminal_report(self, report: Dict):
        """Print formatted terminal report."""
        print(f"\n{'='*60}")
        print(f"Traceability Validation Report")
        print(f"{'='*60}")
        print(f"Repository: {report['repository']}")
        print(f"\n📊 Summary:")
        print(f"  Total Items: {report['summary']['total_items']}")
        print(f"  FRs: {report['summary']['fr_count']}")
        print(f"  USs: {report['summary']['us_count']}")
        print(f"  ADRs: {report['summary']['adr_count']}")
        print(f"  TCs: {report['summary']['tc_count']}")
        print(f"  CEs: {report['summary']['ce_count']}")
        
        color = report['badge_color']
        color_code = {'green': '\033[92m', 'yellow': '\033[93m', 'red': '\033[91m'}.get(color, '')
        reset = '\033[0m'
        
        print(f"\n🎯 Overall Coverage: {color_code}{report['summary']['coverage_percent']:.1f}%{reset}")
        print(f"   Badge: {color.upper()}")
        
        print(f"\n📋 Coverage by Type:")
        for type_name, data in report['coverage_by_type'].items():
            percent_color = '\033[92m' if data['percent'] >= 100 else '\033[93m' if data['percent'] >= 80 else '\033[91m'
            print(f"  {type_name}: {percent_color}{data['percent']:.1f}%{reset} ({data['valid']}/{data['total']})")
        
        if report['missing_links']:
            print(f"\n⚠️  Missing Links ({report['summary']['missing_links']}):")
            for ml in report['missing_links'][:10]:  # Show first 10
                print(f"  {ml['from']} → {ml['to_type']} ({ml['to_id']})")
            if len(report['missing_links']) > 10:
                print(f"  ... and {len(report['missing_links']) - 10} more")
        
        if report['gaps']:
            print(f"\n🚨 Coverage Gaps ({report['summary']['gaps']}):")
            for gap in report['gaps'][:10]:
                severity_color = '\033[91m' if gap['severity'] == 'high' else '\033[93m'
                print(f"  {severity_color}[{gap['severity'].upper()}]{reset} {gap['item_id']}: {gap['item_title']}")
                print(f"    Missing: {gap['missing_link_type']}")
        
        print(f"\n{'='*60}")
        
        # Return exit code based on coverage
        if report['summary']['coverage_percent'] < 100:
            return 1
        return 0
    
    def _print_markdown_report(self, report: Dict):
        """Print markdown report for CI/CD."""
        print(f"# Traceability Validation Report\n")
        print(f"**Repository:** `{report['repository']}`\n")
        
        print(f"## Summary\n")
        print(f"| Metric | Value |")
        print(f"|--------|-------|")
        print(f"| Total Items | {report['summary']['total_items']} |")
        print(f"| Coverage | {report['summary']['coverage_percent']:.1f}% |")
        print(f"| Missing Links | {report['summary']['missing_links']} |")
        print(f"| Gaps | {report['summary']['gaps']} |")
        
        print(f"\n## Coverage by Type\n")
        print(f"| Type | Coverage |")
        print(f"|------|----------|")
        for type_name, data in report['coverage_by_type'].items():
            print(f"| {type_name} | {data['percent']:.1f}% |")
        
        if report['gaps']:
            print(f"\n## Coverage Gaps\n")
            print(f"| Item | Missing Link | Severity |")
            print(f"|------|--------------|----------|")
            for gap in report['gaps']:
                print(f"| {gap['item_id']} | {gap['missing_link_type']} | {gap['severity']} |")
        
        print(f"\n---\n")
        print(f"**Status:** {'✅ PASS' if report['summary']['coverage_percent'] >= 100 else '❌ FAIL'}\n")


def main():
    """CLI entry point."""
    import argparse
    
    parser = argparse.ArgumentParser(
        description='Validate traceability coverage'
    )
    parser.add_argument(
        '--repo',
        default='.',
        help='Path to repository (default: current directory)'
    )
    parser.add_argument(
        '--format',
        choices=['terminal', 'json', 'markdown'],
        default='terminal',
        help='Output format'
    )
    parser.add_argument(
        '--output',
        help='Output file (default: stdout)'
    )
    
    args = parser.parse_args()
    
    validator = TraceabilityValidator(args.repo)
    report = validator.validate()
    
    if args.output:
        with open(args.output, 'w') as f:
            if args.format == 'json':
                json.dump(report, f, indent=2)
            else:
                f.write(str(report))
        print(f"Report saved to {args.output}")
    else:
        exit_code = validator.print_report(report, args.format)
        sys.exit(exit_code)


if __name__ == '__main__':
    main()
