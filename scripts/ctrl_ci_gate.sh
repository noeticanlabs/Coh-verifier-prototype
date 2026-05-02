#!/bin/bash
# CTRL-v1.0 CI Gate Script
# Usage: bash ctrl_ci_gate.sh <repo_root>

ROOT=$1
if [ -z "$ROOT" ]; then
    ROOT="."
fi

echo "--- Starting CTRL Static Audit ---"
SORRY_COUNT=$(grep -r "sorry" $ROOT/coh-t-stack/Coh | wc -l)
ADMIT_COUNT=$(grep -r "admit" $ROOT/coh-t-stack/Coh | wc -l)
AXIOM_COUNT=$(grep -r "axiom" $ROOT/coh-t-stack/Coh | wc -l)

echo "Sorry count: $SORRY_COUNT"
echo "Admit count: $ADMIT_COUNT"
echo "Axiom count: $AXIOM_COUNT"

if [ $SORRY_COUNT -gt 0 ] || [ $ADMIT_COUNT -gt 0 ]; then
    echo "FAILED: Formal debt (sorry/admit) detected in kernel."
    exit 1
fi

if [ $AXIOM_COUNT -gt 1 ]; then
    echo "WARNING: Unexpected axioms detected. Expected only spinor_current_conservation."
fi

echo "--- Checking Lean Build ---"
if command -v lake &> /dev/null; then
    cd $ROOT/coh-t-stack
    lake build Coh
    if [ $? -eq 0 ]; then
        echo "SUCCESS: Lean build verified."
    else
        echo "FAILED: Lean build failed."
        exit 1
    fi
    cd ..
else
    echo "SKIP: lake not found, skipping dynamic build check."
fi

echo "--- Checking Rust Build ---"
if command -v cargo &> /dev/null; then
    cd $ROOT/coh-node
    cargo check -p coh-genesis -p coh-ctrl
    if [ $? -eq 0 ]; then
        echo "SUCCESS: Rust crates verified."
    else
        echo "FAILED: Rust build failed."
        exit 1
    fi
    cd ..
else
    echo "SKIP: cargo not found, skipping rust check."
fi

echo "--- CI GATE PASSED ---"
exit 0
