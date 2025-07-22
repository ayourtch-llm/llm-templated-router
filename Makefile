# Default binary
DEFAULT_BINARY = llm-groq-4

# Discover all workspace members that have both instruct/ and src/ directories
WORKSPACE_MEMBERS := $(shell find . -maxdepth 2 -type d -name instruct -exec dirname {} \; | grep -v "^\./target" | sort -u)

# For each workspace member, find all .md files and create corresponding targets
define create_member_targets
$(1)_INSTRUCT_FILES := $$(shell find $(1)/instruct/ -name "*.md" 2>/dev/null)
$(1)_TARGETS := $$(patsubst $(1)/instruct/%.md,$(1)/src/%.rs,$$($(1)_INSTRUCT_FILES))
ALL_TARGETS += $$($(1)_TARGETS)
endef

# Generate targets for each workspace member
$(foreach member,$(WORKSPACE_MEMBERS),$(eval $(call create_member_targets,$(member))))

# Default target - build all discovered targets across all workspace members
all: $(ALL_TARGETS)

# Per-member targets for convenience
define create_member_phony
.PHONY: $(notdir $(1))
$(notdir $(1)): $$($(1)_TARGETS)
endef

$(foreach member,$(WORKSPACE_MEMBERS),$(eval $(call create_member_phony,$(member))))

# Override binary for specific files (specify full path from workspace root)
./crate1/src/bin/llm-groq.rs: BINARY = llm-claude
./crate2/src/bin/llm-ollama-qwen.rs: BINARY = llm-claude

# Generic pattern rule: any .md in any member's instruct/ creates corresponding .rs
%/src/%.rs: %/instruct/%.md
	@echo "Processing $< -> $@"
	@# Extract the workspace member directory
	$(eval MEMBER_DIR := $(dir $<))
	@# Change to member directory and run cargo
	cd $(MEMBER_DIR) && cargo run --bin $(or $(BINARY),$(DEFAULT_BINARY)) -- instruct/$*.md src/$*.rs

# Alternative approach: run from workspace root with -p flag
# Uncomment this rule and comment the one above if you prefer workspace-level execution
# %/src/%.rs: %/instruct/%.md
# 	@echo "Processing $< -> $@"
# 	@# Extract workspace member name (assuming it's the directory name)
# 	$(eval MEMBER_NAME := $(notdir $(patsubst %/,%,$(dir $<))))
# 	cargo run -p $(MEMBER_NAME) --bin $(or $(BINARY),$(DEFAULT_BINARY)) -- instruct/$*.md src/$*.rs

# Debug target to show discovered members and targets
debug:
	@echo "Workspace members found:"
	@$(foreach member,$(WORKSPACE_MEMBERS),echo "  $(member)";)
	@echo "All targets:"
	@$(foreach target,$(ALL_TARGETS),echo "  $(target)";)

# Clean generated files across all workspace members
clean:
	@$(foreach member,$(WORKSPACE_MEMBERS),find $(member)/src/ -name "*.rs" -newer $(member)/instruct/ -delete 2>/dev/null || true;)

.PHONY: all debug clean
