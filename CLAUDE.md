# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Architecture

This is a Rust workspace that implements an LLM-templated router system with a unique approach to code generation:

- **Template-to-Code Generation**: The core concept is using Markdown files in `instruct/` directories as templates that generate corresponding Rust files in `src/` directories
- **Workspace Structure**: Root workspace with two main members:
  - `llm/`: Original LLM templating framework and code generation tools
  - `claude-code-router/`: New Rust implementation of the TypeScript claude-code-router
- **Binary Architecture**: Multiple specialized binaries in `llm/src/bin/` for different LLM providers and use cases:
  - `llm-groq-*.rs`: Groq API integration binaries (versioned)
  - `rev-llm-groq.rs`: Reverse engineering tool for Groq
  - `next-llm.rs`: Utility for creating next version of LLM binaries
  - `wcr.rs`: Word count and text processing utility

## Claude Code Router Project

### Current Status
The `claude-code-router/` workspace member is a Rust reimplementation of the TypeScript claude-code-router project found in `origin/claude-code-router/`. 

**Completed Components:**
- CLI interface (`ccr` binary) with start/stop/status/help commands using clap
- Configuration module with TypeScript-compatible data structures
- Project structure matching the spec-to-code framework pattern

**Key Files:**
- `claude-code-router/src/bin/ccr.rs`: Main CLI entry point
- `claude-code-router/src/config.rs`: Configuration management with serde JSON support
- `claude-code-router/instruct/bin/ccr.md`: CLI specification
- `claude-code-router/instruct/config.md`: Config module specification
- `claude-code-router/instruct/server.md`: Server module specification (ready for generation)

**Configuration Compatibility:**
The Rust config structures exactly match the TypeScript config format:
- `Config` struct with `Providers`, `Router`, `APIKEY`, `HOST`, `LOG` fields
- `Provider` struct with `name`, `api_base_url`, `api_key`, `models`, `transformer`
- `RouterConfig` with `default`, `background`, `think`, `longContext` routing rules
- Serde field renaming to handle TypeScript naming conventions

**Next Steps:**
1. Generate server module from spec: `source ../.env && cd llm && cargo run --bin llm-groq-5 -- ../claude-code-router/instruct/server.md ../claude-code-router/src/server.rs`
2. Create HTTP routing logic specs
3. Implement LLM provider integration
4. Add authentication middleware
5. Create process management utilities

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

# Build specific workspace member
cargo build -p llm
cargo build -p claude-code-router

# Run specific binary (example)
cd llm && cargo run --bin llm-groq-5 -- instruct/bin/example.md src/bin/output.rs

# Run claude-code-router CLI
cargo run -p claude-code-router --bin ccr -- start

# Build release version
cargo build --release
```

### Code Generation Workflow
```bash
# Generate code from specification (requires GROQ_API_KEY in .env)
source .env && cd llm && cargo run --bin llm-groq-5 -- ../claude-code-router/instruct/MODULE.md ../claude-code-router/src/MODULE.rs

# Example: Generate server module
source .env && cd llm && cargo run --bin llm-groq-5 -- ../claude-code-router/instruct/server.md ../claude-code-router/src/server.rs
```

## Template System

The Makefile implements an automatic code generation system:
- `.md` files in `instruct/bin/` become `.rs` files in `src/bin/`
- Default generator binary is `llm-groq-4` but `llm-groq-5` is currently used
- Override generator with `BINARY` variable for specific files
- Common template includes are in `llm/include/common.md`

### Spec-to-Code Framework Guidelines
For optimal code generation results:
1. **Detailed Specifications**: Include exact struct definitions, field names, and data types
2. **TypeScript Compatibility**: When porting from TypeScript, analyze the original config.example.json and TypeScript interfaces
3. **Small, Self-Contained Pieces**: Break large functionality into focused, single-purpose modules
4. **Placeholder Pattern**: Create minimal placeholder files first (e.g., `// Placeholder module` or `fn main() {}`)
5. **Frequent Commits**: Commit before and after each generation to enable git safety checks
6. **Error Handling**: Specify exact error handling patterns and Result types

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