#!/usr/bin/env python3
"""
Project Structure Analyzer

This script analyzes the project structure to determine the project type
and provides information about it.
"""

import os
import sys
import json
from pathlib import Path
from typing import Dict, List, Optional, Tuple


class ProjectAnalyzer:
    """Analyzes project structure to determine project type and characteristics."""
    
    def __init__(self, project_path: str = "."):
        """Initialize the analyzer with a project path."""
        self.project_path = Path(project_path)
        self.project_type = None
        self.detected_files = {}
        self.analysis_result = {}
    
    def detect_project_type(self) -> str:
        """
        Detect the project type based on configuration files.
        
        Returns:
            str: Project type ('rust', 'python', 'javascript', 'unknown')
        """
        # Check for Rust project files
        if (self.project_path / "Cargo.toml").exists():
            self.project_type = "rust"
            self.detected_files["Cargo.toml"] = True
            
            # Check for additional Rust files
            if (self.project_path / "Cargo.lock").exists():
                self.detected_files["Cargo.lock"] = True
            if (self.project_path / "src").exists():
                self.detected_files["src/"] = True
                
        # Check for Python project files
        elif (self.project_path / "pyproject.toml").exists():
            self.project_type = "python"
            self.detected_files["pyproject.toml"] = True
        elif (self.project_path / "setup.py").exists():
            self.project_type = "python"
            self.detected_files["setup.py"] = True
        elif (self.project_path / "requirements.txt").exists():
            self.project_type = "python"
            self.detected_files["requirements.txt"] = True
            
        # Check for JavaScript/Node.js project files
        elif (self.project_path / "package.json").exists():
            self.project_type = "javascript"
            self.detected_files["package.json"] = True
            
        else:
            self.project_type = "unknown"
            
        return self.project_type
    
    def analyze_rust_project(self) -> Dict:
        """Analyze a Rust project and extract information."""
        result = {
            "type": "rust",
            "cargo_toml_exists": False,
            "cargo_lock_exists": False,
            "src_directory_exists": False,
            "main_rs_exists": False,
            "lib_rs_exists": False,
            "dependencies": [],
            "package_info": {}
        }
        
        # Check Cargo.toml
        cargo_toml_path = self.project_path / "Cargo.toml"
        if cargo_toml_path.exists():
            result["cargo_toml_exists"] = True
            try:
                with open(cargo_toml_path, 'r') as f:
                    content = f.read()
                    
                    # Extract package name
                    for line in content.split('\n'):
                        if line.strip().startswith('name ='):
                            result["package_info"]["name"] = line.split('=')[1].strip().strip('"')
                        elif line.strip().startswith('version ='):
                            result["package_info"]["version"] = line.split('=')[1].strip().strip('"')
                        elif line.strip().startswith('edition ='):
                            result["package_info"]["edition"] = line.split('=')[1].strip().strip('"')
                        elif line.strip().startswith('[') and 'dependencies' in line:
                            # Start of dependencies section
                            pass
                        elif line.strip() and not line.strip().startswith('[') and '=' in line:
                            # This might be a dependency line
                            dep_line = line.strip()
                            if not dep_line.startswith('#'):  # Skip comments
                                result["dependencies"].append(dep_line)
            except Exception as e:
                result["package_info"]["error"] = f"Failed to parse Cargo.toml: {e}"
        
        # Check Cargo.lock
        if (self.project_path / "Cargo.lock").exists():
            result["cargo_lock_exists"] = True
        
        # Check src directory
        src_path = self.project_path / "src"
        if src_path.exists() and src_path.is_dir():
            result["src_directory_exists"] = True
            
            # Check for main.rs
            if (src_path / "main.rs").exists():
                result["main_rs_exists"] = True
            
            # Check for lib.rs
            if (src_path / "lib.rs").exists():
                result["lib_rs_exists"] = True
        
        return result
    
    def analyze_directory_structure(self, max_depth: int = 3) -> Dict:
        """Analyze the directory structure up to a certain depth."""
        structure = {}
        
        def scan_dir(path: Path, depth: int, current_dict: Dict):
            if depth > max_depth:
                return
            
            try:
                for item in path.iterdir():
                    if item.is_dir():
                        # Skip common large directories
                        if item.name in ['.git', 'node_modules', 'target', '__pycache__', '.venv']:
                            continue
                        
                        current_dict[item.name] = {}
                        scan_dir(item, depth + 1, current_dict[item.name])
                    else:
                        # Only include key files at deeper levels
                        if depth <= 2 or item.suffix in ['.rs', '.py', '.js', '.ts', '.toml', '.json', '.md']:
                            current_dict[item.name] = "file"
            except PermissionError:
                current_dict["[Permission Denied]"] = "error"
            except Exception as e:
                current_dict[f"[Error: {e}]"] = "error"
        
        scan_dir(self.project_path, 0, structure)
        return structure
    
    def run_analysis(self) -> Dict:
        """Run complete analysis of the project."""
        # Detect project type
        project_type = self.detect_project_type()
        
        # Initialize result
        self.analysis_result = {
            "project_type": project_type,
            "detected_files": self.detected_files,
            "project_path": str(self.project_path.absolute()),
            "analysis_timestamp": None
        }
        
        # Add type-specific analysis
        if project_type == "rust":
            self.analysis_result["rust_analysis"] = self.analyze_rust_project()
        
        # Add directory structure (limited)
        self.analysis_result["directory_structure"] = self.analyze_directory_structure(max_depth=2)
        
        return self.analysis_result
    
    def print_summary(self):
        """Print a human-readable summary of the analysis."""
        print("=" * 60)
        print("PROJECT ANALYSIS SUMMARY")
        print("=" * 60)
        print(f"Project Path: {self.analysis_result.get('project_path', 'Unknown')}")
        print(f"Project Type: {self.analysis_result.get('project_type', 'Unknown').upper()}")
        print()
        
        if self.analysis_result.get('project_type') == 'rust':
            rust_info = self.analysis_result.get('rust_analysis', {})
            print("RUST PROJECT DETAILS:")
            print("-" * 40)
            
            if rust_info.get('cargo_toml_exists'):
                pkg_info = rust_info.get('package_info', {})
                print(f"  Package: {pkg_info.get('name', 'Unknown')}")
                print(f"  Version: {pkg_info.get('version', 'Unknown')}")
                print(f"  Edition: {pkg_info.get('edition', 'Unknown')}")
            
            print(f"  Cargo.toml: {'✓' if rust_info.get('cargo_toml_exists') else '✗'}")
            print(f"  Cargo.lock: {'✓' if rust_info.get('cargo_lock_exists') else '✗'}")
            print(f"  src/ directory: {'✓' if rust_info.get('src_directory_exists') else '✗'}")
            print(f"  main.rs: {'✓' if rust_info.get('main_rs_exists') else '✗'}")
            print(f"  lib.rs: {'✓' if rust_info.get('lib_rs_exists') else '✗'}")
            
            deps = rust_info.get('dependencies', [])
            if deps:
                print(f"  Dependencies found: {len(deps)}")
                if len(deps) <= 5:
                    for dep in deps[:5]:
                        print(f"    - {dep}")
                else:
                    print(f"    (Showing first 5 of {len(deps)})")
                    for dep in deps[:5]:
                        print(f"    - {dep}")
        
        print()
        print("KEY FILES DETECTED:")
        print("-" * 40)
        for file, exists in self.analysis_result.get('detected_files', {}).items():
            if exists:
                print(f"  ✓ {file}")
        
        print()
        print("DIRECTORY STRUCTURE (simplified):")
        print("-" * 40)
        self._print_structure(self.analysis_result.get('directory_structure', {}), indent=2)
        
        print("=" * 60)
    
    def _print_structure(self, structure: Dict, indent: int = 0):
        """Recursively print directory structure."""
        for name, value in structure.items():
            if isinstance(value, dict):
                print(" " * indent + f"📁 {name}/")
                self._print_structure(value, indent + 2)
            else:
                print(" " * indent + f"📄 {name}")
    
    def save_report(self, output_path: str = "project_analysis_report.json"):
        """Save the analysis report to a JSON file."""
        try:
            with open(output_path, 'w') as f:
                json.dump(self.analysis_result, f, indent=2)
            print(f"Report saved to: {output_path}")
            return True
        except Exception as e:
            print(f"Failed to save report: {e}")
            return False


def main():
    """Main function to run the project analysis."""
    import argparse
    
    parser = argparse.ArgumentParser(description="Analyze project structure and determine project type")
    parser.add_argument("path", nargs="?", default=".", help="Path to the project directory (default: current directory)")
    parser.add_argument("--json", action="store_true", help="Output results in JSON format")
    parser.add_argument("--save", metavar="FILE", help="Save analysis report to JSON file")
    parser.add_argument("--check-rust", action="store_true", help="Specifically check if it's a Rust project")
    
    args = parser.parse_args()
    
    # Create analyzer and run analysis
    analyzer = ProjectAnalyzer(args.path)
    results = analyzer.run_analysis()
    
    # Output based on arguments
    if args.json:
        print(json.dumps(results, indent=2))
    elif args.check_rust:
        if results.get('project_type') == 'rust':
            print("✅ This is a Rust project.")
            rust_info = results.get('rust_analysis', {})
            if rust_info.get('cargo_toml_exists'):
                pkg_info = rust_info.get('package_info', {})
                print(f"   Package: {pkg_info.get('name', 'Unknown')} v{pkg_info.get('version', 'Unknown')}")
        else:
            print(f"❌ This is not a Rust project. Detected type: {results.get('project_type')}")
    else:
        analyzer.print_summary()
    
    # Save report if requested
    if args.save:
        analyzer.save_report(args.save)


if __name__ == "__main__":
    main()