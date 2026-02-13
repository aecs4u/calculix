# CalculiX Rust Solver - Documentation Index

**Last Updated**: 2026-02-11

This document provides a comprehensive index of all project documentation with recommendations for which documents to read based on your needs.

---

## 📋 Quick Start

| If you want to... | Read this |
|-------------------|-----------|
| **Understand the project** | [README.md](README.md) |
| **See current status** | [IMPLEMENTATION_STATUS.md](IMPLEMENTATION_STATUS.md) |
| **Run the solver** | [CLI Usage](#cli-usage) |
| **Run tests** | [TEST_SUITE_FIXES.md](TEST_SUITE_FIXES.md) |
| **See what's been done** | [Session Summaries](#session-summaries) |
| **Contribute** | [Development Plan](#development-plan) |

---

## 🗂️ Documentation Categories

### Core Documentation

#### 1. **[README.md](README.md)** - Project Overview
- **Purpose**: Main project documentation
- **Contains**: Architecture, features, build instructions
- **Status**: ✅ Current
- **Audience**: Everyone

#### 2. **[IMPLEMENTATION_STATUS.md](IMPLEMENTATION_STATUS.md)** - Current State
- **Purpose**: Comprehensive status report
- **Contains**:
  - Element types implemented (6/40)
  - Analysis types (4/16)
  - Test coverage (206+ tests, 75% pass rate)
  - Performance benchmarks
  - Production readiness checklist
- **Status**: ✅ Current (Updated 2026-02-11)
- **Audience**: Developers, project managers

### Test & Validation

#### 3. **[TEST_SUITE_FIXES.md](TEST_SUITE_FIXES.md)** - Test Infrastructure
- **Purpose**: Documents pytest test suite fixes
- **Contains**:
  - Fixed skipped tests (0 skipped now)
  - Rust solver test suite (4 new tests)
  - CGX viewer tests working
  - Legacy CCX tests marked as xfail
- **Status**: ✅ Current (2026-02-11)
- **Audience**: QA, developers

#### 4. **[RUST_SOLVER_FIXTURE_RESULTS.md](RUST_SOLVER_FIXTURE_RESULTS.md)** - Validation Results
- **Purpose**: Documents Rust solver validation on test fixtures
- **Contains**:
  - 5 fixtures tested (100% success)
  - Comparison with legacy CCX
  - Commands for running tests
  - Production readiness: 70%
- **Status**: ✅ Current (2026-02-11)
- **Audience**: QA, validation engineers

#### 5. **[TESTING.md](TESTING.md)** - Test Guide *(if exists)*
- **Purpose**: Testing procedures and guidelines
- **Status**: Check if current
- **Audience**: Developers

### Architecture & Planning

#### 6. **[docs/ccx_solver_modernization_roadmap.md](docs/ccx_solver_modernization_roadmap.md)** - Architecture
- **Purpose**: High-level architecture and modernization strategy
- **Contains**: Backend abstraction, solver design
- **Status**: ✅ Reference document
- **Audience**: Architects, senior developers

#### 7. **[DEVELOPMENT_PLAN.md](DEVELOPMENT_PLAN.md)** - Development Roadmap
- **Purpose**: Long-term development plan
- **Status**: Check if current
- **Audience**: Project managers, architects

#### 8. **[C3D8_IMPLEMENTATION_PLAN.md](C3D8_IMPLEMENTATION_PLAN.md)** - Element Implementation
- **Purpose**: Plan for implementing C3D8 solid elements
- **Status**: ✅ Complete (C3D8 implemented)
- **Audience**: Historical reference

### Session Summaries

#### 9. **[SESSION_SUMMARY_FINAL.md](SESSION_SUMMARY_FINAL.md)** - Most Recent Summary
- **Date**: 2026-02-09
- **Contains**: T3D3, B32 elements, Dynamic/Nonlinear solvers
- **Status**: ✅ Complete
- **Audience**: Tracking progress

#### 10. **[SESSION_SUMMARY_2026-02-09.md](SESSION_SUMMARY_2026-02-09.md)** - Previous Session
- **Status**: Historical
- **Audience**: Historical reference

#### 11. **[SESSION_SUMMARY_TODAY.md](SESSION_SUMMARY_TODAY.md)** - Current Session
- **Status**: ⚠️ May be outdated, check date
- **Audience**: Current work tracking

### Integration & Features

#### 12. **[CCXIO_INTEGRATION_COMPLETE.md](CCXIO_INTEGRATION_COMPLETE.md)** - I/O Integration
- **Purpose**: Documents ccx-io integration
- **Status**: ✅ Complete
- **Audience**: Developers working on I/O

#### 13. **[CLI_UPDATES_SUMMARY.md](CLI_UPDATES_SUMMARY.md)** - CLI Changes
- **Purpose**: Documents CLI tool updates
- **Status**: Check if current
- **Audience**: Users, developers

#### 14. **[EXAMPLES_INTEGRATION.md](EXAMPLES_INTEGRATION.md)** - Examples
- **Purpose**: Integration of 1,133 example INP files
- **Status**: ✅ Complete
- **Audience**: QA, validation

#### 15. **[PYNASTRAN_INTEGRATION.md](PYNASTRAN_INTEGRATION.md)** - PyNastran Integration
- **Purpose**: PyNastran mesh reader integration
- **Status**: Check status
- **Audience**: Developers

### Validation System

#### 16. **[VALIDATION_SYSTEM.md](VALIDATION_SYSTEM.md)** - Validation Infrastructure
- **Purpose**: Validation system design and implementation
- **Status**: Check if current
- **Audience**: QA engineers

#### 17. **[VALIDATION_SYSTEM_SUMMARY.md](VALIDATION_SYSTEM_SUMMARY.md)** - Summary
- **Purpose**: Summary of validation capabilities
- **Status**: Check if current
- **Audience**: QA, managers

#### 18. **[scratch/VALIDATION_SUMMARY.md](scratch/VALIDATION_SUMMARY.md)** - Test Results
- **Purpose**: Validation test run summaries
- **Status**: Historical results
- **Audience**: QA

### Examples & Tutorials

#### 19. **[examples/RUST_SOLVER_EXAMPLES.md](examples/RUST_SOLVER_EXAMPLES.md)** - Solver Examples
- **Purpose**: Example usage of Rust solver
- **Status**: Check if current
- **Audience**: Users, developers

#### 20. **[examples/CATALOG.md](examples/CATALOG.md)** - Example Catalog
- **Purpose**: Catalog of 1,133 example files
- **Status**: ✅ Complete
- **Audience**: Users finding examples

#### 21. **[examples/README.md](examples/README.md)** - Examples Overview
- **Purpose**: Overview of example files
- **Status**: Check if current
- **Audience**: Users

### Build & CLI

#### 22. **[docs/ccx_2.23_build_scripts.md](docs/ccx_2.23_build_scripts.md)** - Build Scripts
- **Purpose**: Building legacy CCX binaries
- **Status**: ✅ Reference
- **Audience**: Build engineers

#### 23. **[docs/calculix_cli.md](docs/calculix_cli.md)** - CLI Documentation
- **Purpose**: CLI tool documentation
- **Status**: Check if current
- **Audience**: Users

#### 24. **[docs/FEATURE_COMPARISON.md](docs/FEATURE_COMPARISON.md)** - Feature Matrix
- **Purpose**: Comparison of features vs legacy
- **Status**: Check if current
- **Audience**: Managers, users

---

## 🔍 Recommended Reading Paths

### For New Contributors
1. [README.md](README.md) - Understand the project
2. [IMPLEMENTATION_STATUS.md](IMPLEMENTATION_STATUS.md) - See current state
3. [docs/ccx_solver_modernization_roadmap.md](docs/ccx_solver_modernization_roadmap.md) - Architecture
4. [DEVELOPMENT_PLAN.md](DEVELOPMENT_PLAN.md) - Future work

### For QA Engineers
1. [TEST_SUITE_FIXES.md](TEST_SUITE_FIXES.md) - Test infrastructure
2. [RUST_SOLVER_FIXTURE_RESULTS.md](RUST_SOLVER_FIXTURE_RESULTS.md) - Validation results
3. [VALIDATION_SYSTEM.md](VALIDATION_SYSTEM.md) - Validation system
4. [examples/CATALOG.md](examples/CATALOG.md) - Test fixtures

### For Users
1. [README.md](README.md) - Getting started
2. [docs/calculix_cli.md](docs/calculix_cli.md) - Using the CLI
3. [examples/RUST_SOLVER_EXAMPLES.md](examples/RUST_SOLVER_EXAMPLES.md) - Examples
4. [RUST_SOLVER_FIXTURE_RESULTS.md](RUST_SOLVER_FIXTURE_RESULTS.md) - Capabilities

### For Project Managers
1. [IMPLEMENTATION_STATUS.md](IMPLEMENTATION_STATUS.md) - Current status
2. [SESSION_SUMMARY_FINAL.md](SESSION_SUMMARY_FINAL.md) - Recent work
3. [DEVELOPMENT_PLAN.md](DEVELOPMENT_PLAN.md) - Future roadmap
4. [docs/FEATURE_COMPARISON.md](docs/FEATURE_COMPARISON.md) - Feature comparison

---

## 📊 Documentation Health

### Current & Maintained
- ✅ README.md
- ✅ IMPLEMENTATION_STATUS.md
- ✅ TEST_SUITE_FIXES.md
- ✅ RUST_SOLVER_FIXTURE_RESULTS.md
- ✅ SESSION_SUMMARY_FINAL.md

### Needs Review
- ⚠️ TESTING.md
- ⚠️ DEVELOPMENT_PLAN.md
- ⚠️ CLI_UPDATES_SUMMARY.md
- ⚠️ VALIDATION_SYSTEM.md
- ⚠️ docs/FEATURE_COMPARISON.md

### Historical (Archive Recommended)
- 📦 SESSION_SUMMARY_2026-02-09.md
- 📦 C3D8_IMPLEMENTATION_PLAN.md (implemented)
- 📦 SESSION_SUMMARY_TODAY.md (check date)

---

## 🚀 CLI Usage

### Quick Commands

```bash
# Check version
target/release/ccx-cli --version

# Analyze an INP file
target/release/ccx-cli analyze tests/fixtures/solver/achtel2.inp

# Run validation
target/release/ccx-cli validate --fixtures-dir tests/fixtures/solver

# Run tests
uv run pytest -v

# Run Rust solver tests only
uv run pytest -v -m rust_solver
```

---

## 📁 Directory Structure

```
calculix/
├── crates/
│   ├── ccx-solver/       # Rust FEA solver core
│   ├── ccx-cli/          # Command-line interface
│   ├── ccx-inp/          # Input file parser
│   ├── ccx-model/        # Domain model
│   ├── ccx-io/           # I/O routines
│   └── webapp/   # FastAPI validation dashboard
├── docs/                 # Architecture docs
├── examples/             # 1,133 example INP files
├── tests/                # Pytest test suite
│   └── fixtures/solver/  # 638 test .inp files
├── validation/solver/    # 629 reference .dat.ref files
└── scripts/              # Build and test scripts
```

---

## 🔄 Documentation Update Policy

When creating new documentation:
1. **Update this index** with the new document
2. **Add to appropriate category**
3. **Mark status** (Current/Needs Review/Historical)
4. **Update last modified date**

When a document becomes obsolete:
1. **Mark as Historical** in this index
2. **Move to archives/** directory if needed
3. **Update cross-references**

---

## 📝 Contributing

See [DEVELOPMENT_PLAN.md](DEVELOPMENT_PLAN.md) for:
- Coding standards
- PR process
- Testing requirements
- Documentation requirements

---

## 📧 Contact

**Repository**: https://github.com/aecs4u/calculix
**Branch**: feature/ccx223-build-scripts

For questions or issues, open an issue or PR on GitHub.
