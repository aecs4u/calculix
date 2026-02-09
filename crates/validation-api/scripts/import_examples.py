#!/usr/bin/env python3
"""Import all example INP files from examples/ directory into the validation database."""

import subprocess
import sys
from pathlib import Path
from datetime import datetime

# Add app to path
sys.path.insert(0, str(Path(__file__).parent.parent))

from sqlalchemy import func

from app.database import (
    Example,
    SessionLocal,
    TestCase,
    TestModule,
    ValidationResult,
    init_db,
)


def categorize_example(path: Path) -> str:
    """Categorize an example based on its path."""
    path_str = str(path).lower()

    if "contact" in path_str or "cont/" in path_str:
        return "Contact"
    elif "dynamic" in path_str:
        return "Dynamics"
    elif "linear" in path_str:
        return "Linear"
    elif "nonlinear" in path_str:
        return "NonLinear"
    elif "thermal" in path_str or "heat" in path_str:
        return "Thermal"
    elif "frequency" in path_str or "modal" in path_str:
        return "Modal"
    elif "buckle" in path_str or "buckling" in path_str:
        return "Buckling"
    elif "beam" in path_str:
        return "Beam"
    elif "shell" in path_str:
        return "Shell"
    elif "solid" in path_str or "3d" in path_str:
        return "Solid"
    elif "plate" in path_str:
        return "Plate"
    elif "disk" in path_str or "axisym" in path_str:
        return "Axisymmetric"
    elif "truss" in path_str:
        return "Truss"
    else:
        return "Other"


def find_inp_files(directory: Path) -> list[Path]:
    """Recursively find all .inp files in a directory."""
    return list(directory.rglob("*.inp"))


def parse_inp_file(file_path: Path) -> dict | None:
    """
    Parse an INP file and extract basic information.
    Returns None if parsing fails.
    """
    try:
        with open(file_path, "r", encoding="utf-8", errors="ignore") as f:
            content = f.read()

        # Count basic elements
        node_count = content.count("*NODE")
        element_count = content.count("*ELEMENT")
        material_count = content.count("*MATERIAL")
        step_count = content.count("*STEP")

        # Detect analysis type
        has_static = "*STATIC" in content
        has_frequency = "*FREQUENCY" in content
        has_buckle = "*BUCKLE" in content
        has_dynamic = "*DYNAMIC" in content
        has_heat_transfer = "*HEAT TRANSFER" in content

        return {
            "node_count": node_count,
            "element_count": element_count,
            "material_count": material_count,
            "step_count": step_count,
            "has_static": has_static,
            "has_frequency": has_frequency,
            "has_buckle": has_buckle,
            "has_dynamic": has_dynamic,
            "has_heat_transfer": has_heat_transfer,
        }
    except Exception as e:
        print(f"  ⚠️  Failed to parse {file_path.name}: {e}")
        return None


def import_examples(examples_dir: Path, db):
    """Import all examples from the examples directory."""
    print(f"📂 Scanning examples directory: {examples_dir}")

    inp_files = find_inp_files(examples_dir)
    print(f"   Found {len(inp_files)} INP files")

    # Group by category
    categories = {}
    for file_path in inp_files:
        category = categorize_example(file_path)
        if category not in categories:
            categories[category] = []
        categories[category].append(file_path)

    print(f"\n📊 Breakdown by category:")
    for category, files in sorted(categories.items()):
        print(f"   {category:15} {len(files):4} files")

    # Create or get test module for examples
    examples_module = (
        db.query(TestModule).filter(TestModule.name == "examples").first()
    )
    if not examples_module:
        examples_module = TestModule(
            name="examples",
            description="Validation of example INP files from examples/ directory",
        )
        db.add(examples_module)
        db.commit()
        db.refresh(examples_module)

    print(f"\n📝 Importing examples into database...")

    imported = 0
    skipped = 0
    failed = 0

    for file_path in inp_files:
        rel_path = file_path.relative_to(examples_dir)
        name = str(rel_path)

        # Check if already exists
        existing = db.query(Example).filter(Example.name == name).first()
        if existing:
            skipped += 1
            continue

        # Parse file
        info = parse_inp_file(file_path)
        if info is None:
            failed += 1
            continue

        # Determine description
        category = categorize_example(file_path)
        description = f"{category} analysis example from {rel_path.parts[0] if len(rel_path.parts) > 1 else 'root'}"

        # Create example
        example = Example(
            name=name,
            description=description,
            input_file_path=str(file_path),
            element_type="Mixed",  # Most examples have multiple element types
        )
        db.add(example)
        imported += 1

        # Create a test case for this example
        test_name = f"parse_{name.replace('/', '_').replace('.inp', '')}"
        test_case = TestCase(
            module_id=examples_module.id,
            name=test_name,
            description=f"Parse validation for {name}",
            test_type="parse",
        )
        db.add(test_case)

        if imported % 100 == 0:
            print(f"   Imported {imported} examples...")
            db.commit()

    db.commit()

    print(f"\n✅ Import complete:")
    print(f"   Imported: {imported}")
    print(f"   Skipped (already exists): {skipped}")
    print(f"   Failed: {failed}")

    return imported


def update_statistics(db):
    """Update database statistics after import."""
    total_examples = db.query(Example).count()
    total_tests = db.query(TestCase).count()
    total_modules = db.query(TestModule).count()

    print(f"\n📈 Database statistics:")
    print(f"   Total examples: {total_examples}")
    print(f"   Total test cases: {total_tests}")
    print(f"   Total modules: {total_modules}")


def main():
    print("🦀 CalculiX Example Files Import")
    print("=" * 50)

    # Find examples directory
    script_dir = Path(__file__).parent
    project_root = script_dir.parent.parent.parent
    examples_dir = project_root / "examples"

    if not examples_dir.exists():
        print(f"❌ Examples directory not found: {examples_dir}")
        sys.exit(1)

    # Initialize database
    print("🗄️  Connecting to database...")
    init_db()
    db = SessionLocal()

    try:
        # Import examples
        imported = import_examples(examples_dir, db)

        # Update statistics
        update_statistics(db)

        print(f"\n🎉 Successfully imported {imported} examples!")

    except Exception as e:
        print(f"\n❌ Error: {e}")
        import traceback

        traceback.print_exc()
        sys.exit(1)
    finally:
        db.close()


if __name__ == "__main__":
    main()
