#!/usr/bin/env bash
set -euo pipefail

if [[ $# -lt 1 ]]; then
  echo "Usage: $0 <ENV_VAR_NAME> [input_file]"
  echo "Example: make build-example && make deploy-cross-contract-call | ./update_contract_env.sh CONTRACT_ADDRESS_CROSS_CALL"
  exit 1
fi

ENV_VAR="$1"
INPUT="${2:-/dev/stdin}"

# Extract address from the input
ADDRESS=$(grep 'deployed code at address' "$INPUT" | sed 's/\x1b\[[0-9;]*m//g' | awk '{print $5}')

if [[ -z "${ADDRESS}" ]]; then
  echo "No deployed address found in input." >&2
  exit 1
fi

echo "Found deployed contract address: $ADDRESS"

# Update or insert the variable in .env
if grep -q "^${ENV_VAR}=" .env 2>/dev/null; then
  sed -i.bak "s|^${ENV_VAR}=.*|${ENV_VAR}=${ADDRESS}|" .env
else
  echo "${ENV_VAR}=${ADDRESS}" >> .env
fi

echo "Updated .env with ${ENV_VAR}=${ADDRESS}"

