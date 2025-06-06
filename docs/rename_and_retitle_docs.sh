#!/usr/bin/env bash
set -euo pipefail

FILES=(
  "000-introduction-to-causality.md|00-introduction-to-causality.md|00 Introduction to Causality"
  "001-core-principles.md|01-core-principles.md|01 Core Principles"
  "013-glossary-of-terms.md|02-glossary-of-terms.md|02 Glossary of Terms"
  "002-three-layer-architecture.md|03-three-layer-architecture.md|03 Three-Layer Architecture"
  "015-architectural-module-reorganization.md|04-architectural-module-reorganization.md|04 Architectural Module Reorganization"
  "003-layer-0-the-verifiable-execution-core.md|05-layer-0-the-verifiable-execution-core.md|05 Layer 0: The Verifiable Execution Core"
  "004-layer-1-structured-types-and-causality-lisp.md|06-layer-1-structured-types-and-causality-lisp.md|06 Layer 1: Structured Types and Causality Lisp"
  "005-layer-2-declarative-programming-effects-intents-orchestration.md|07-layer-2-declarative-programming-effects-intents-orchestration.md|07 Layer 2: Declarative Programming, Effects, Intents, Orchestration"
  "014-row-types-and-capability-system.md|08-row-types-and-capability-system.md|08 Row Types and Capability System"
  "015-hybrid-value-model.md|09-hybrid-value-model.md|09 Hybrid Value Model"
  "016-content-addressing-system.md|10-content-addressing-system.md|10 Content Addressing System"
  "007-causality-lisp-language-specification.md|11-causality-lisp-language-specification.md|11 Causality Lisp Language Specification"
  "007-tutorial-crafting-your-first-linear-application.md|12-tutorial-crafting-your-first-linear-application.md|12 Tutorial: Crafting Your First Linear Application"
  "008-advanced-examples-and-idiomatic-patterns.md|13-advanced-examples-and-idiomatic-patterns.md|13 Advanced Examples and Idiomatic Patterns"
  "010-causality-toolkit-and-rust-dsl.md|14-causality-toolkit-and-rust-dsl.md|14 Causality Toolkit and Rust DSL"
  "008-zero-knowledge-proof-integration.md|15-zero-knowledge-proof-integration.md|15 Zero-Knowledge Proof Integration"
  "009-simulation-engine-and-testing-strategies.md|16-simulation-engine-and-testing-strategies.md|16 Simulation Engine and Testing Strategies"
  "011-ocaml-integration-and-bindings.md|17-ocaml-integration-and-bindings.md|17 OCaml Integration and Bindings"
  "006-environment-setup-and-first-build.md|18-environment-setup-and-first-build.md|18 Environment Setup and First Build"
  "012-deployment-and-operational-considerations.md|19-deployment-and-operational-considerations.md|19 Deployment and Operational Considerations"
  "014-appendices.md|20-appendices.md|20 Appendices"
)

for entry in "${FILES[@]}"; do
  IFS="|" read -r OLD NEW TITLE <<< "$entry"
  if [[ -f "$OLD" ]]; then
    mv "$OLD" "$NEW"
  fi
  if [[ -f "$NEW" ]]; then
    # Get the first line
    firstline=$(head -n 1 "$NEW")
    if [[ $firstline =~ ^# ]]; then
      # Replace only the first line if it starts with '#'
      sed -i '' "1s/^# .*/# $TITLE/" "$NEW"
    else
      # Insert the title at the top if missing
      sed -i '' "1i\\
# $TITLE
" "$NEW"
    fi
  fi
done

echo "Docs renaming and title update complete."