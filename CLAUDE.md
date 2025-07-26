# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Architecture

This is a Rust workspace that implements an LLM-templated router system with a unique approach to code generation:

- **Template-to-Code Generation**: The core concept is using Markdown files in `instruct/` directories as templates that generate corresponding Rust files in `src/` directories
- **Workspace Structure**: Root workspace with `llm/` member containing the main functionality
- **Binary Architecture**: Multiple specialized binaries in `llm/src/bin/` for different LLM providers and use cases:
  - `llm-groq-*.rs`: Groq API integration binaries (versioned)
  - `rev-llm-groq.rs`: Reverse engineering tool for Groq
  - `next-llm.rs`: Utility for creating next version of LLM binaries
  - `wcr.rs`: Word count and text processing utility

## Key Development Commands

### Building and Generation
```bash
# Build all targets (generates .rs files from .md templates)
make all

# Build specific workspace member
make llm

# Show discovered targets and workspace members
make debug

# Clean generated files
make clean
```

### Cargo Commands
```bash
# Build all workspace members
cargo build

# Run specific binary (example)
cd llm && cargo run --bin llm-groq-5 -- instruct/bin/example.md src/bin/output.rs

# Build release version
cargo build --release
```

## Template System

The Makefile implements an automatic code generation system:
- `.md` files in `instruct/bin/` become `.rs` files in `src/bin/`
- Default generator binary is `llm-groq-4`
- Override generator with `BINARY` variable for specific files
- Common template includes are in `llm/include/common.md`

## File Safety and Git Integration

The `llm-groq-*.rs` binaries include git safety checks:
- Verify output files have no uncommitted changes before generation
- Automatic git operations in `next-llm.rs` for version management
- Draft and rejection file handling (`.draft`, `.rej` extensions)

## Version Management

Use `next-llm.rs` to create new versions of LLM binaries:
```bash
cd llm && cargo run --bin next-llm -- groq
```
This copies the highest numbered `llm-groq-*.md` and `.rs` files to the next version and commits them.