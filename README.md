# LLM-Templated Router: Spec-Driven Development Framework

A Rust workspace exploring **spec-driven development** using LLM code generation, featuring a claude-code-router implementation.

## 🎯 Core Concept: Specification-First Development

This project implements a development workflow where **Markdown specifications drive code generation**:

```
Specification (.md) → LLM Generator → Implementation (.rs) → Verification
```

### How It Works

1. **Write specifications** in `instruct/*.md` files describing requirements
2. **Generate code** using LLM tools: `llm-groq-5 spec.md → code.rs`
3. **Verify implementation** matches specification through regeneration testing
4. **Maintain consistency** between documentation and code

## 🏗️ Project Structure

- **`llm/`** - LLM templating framework and code generation tools
- **`claude-code-router/`** - Rust implementation of TypeScript claude-code-router
- **`origin/`** - Original TypeScript implementations for comparison

## 🚀 Claude Code Router (Primary Example)

A Rust reimplementation achieving 85-90% feature parity with the original TypeScript version:

```bash
# Start the intelligent LLM router
cargo run --bin ccr -- start

# Route Claude Code CLI through different LLM providers
cargo run --bin ccr -- code "list the files in current directory"
```

**Features:**
- Intelligent routing based on token count, model type, tools
- Multiple LLM providers (OpenRouter, Groq, Gemini, etc.)
- Format transformation between Claude and OpenAI APIs
- Compatible with TypeScript version configuration

## 💭 Claude's Assessment of Spec-Driven Development

### What Worked Well:

**Documentation Quality**
- Specifications serve as living documentation
- Forces clear thinking about requirements upfront
- Creates explicit contracts between modules

**Verification Through Regeneration**
- Can regenerate codebase from specs to verify accuracy
- Helps catch when documentation diverges from implementation
- Provides a form of regression testing

**Modular Architecture**
- Specification-first approach encourages well-defined boundaries
- Naturally creates focused, testable components
- Self-documenting interfaces

**Iteration Speed**
- Can experiment with different implementations quickly
- Easy to refactor when specifications evolve

### Challenges Observed:

**Specification Writing**
- Requires skill to balance requirements vs implementation details
- Too vague generates poor code, too detailed becomes prescriptive
- Learning curve for effective specification patterns

**Tool Dependencies**
- Workflow depends on LLM code generation quality and availability
- Need backup plans when generation fails or produces poor results

**Complexity**
- More moving parts than traditional development
- Initial setup requires understanding multiple tools
- Can be overkill for simple projects

### Assessment

This approach shows promise for projects with:
- Well-understood requirements and interfaces
- Need for tight documentation-code alignment
- Complex systems benefiting from modular design
- Team environments where specifications aid coordination

The claude-code-router demonstrates the methodology can produce functional software while maintaining specification alignment. However, it requires investment in tooling and specification-writing skills.

**Trade-offs**: More upfront complexity in exchange for better documentation consistency and modular architecture.

## 🛠️ Getting Started

```bash
# Clone and setup
git clone <repository>
cd llm-templated-router

# Build the workspace
cargo build

# Try the claude-code-router
cd claude-code-router
cargo run --bin ccr -- start

# Generate code from specifications (requires GROQ_API_KEY)
source .env
cd llm
cargo run --bin llm-groq-5 -- ../claude-code-router/instruct/MODULE.md ../claude-code-router/src/MODULE.rs
```

## 📄 License

MIT License - See [LICENSE.md](LICENSE.md)

---

*An exploration of specification-driven development using large language models for code generation.*