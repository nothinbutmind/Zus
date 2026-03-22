#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage:
  generate_shared_verifier.sh --circuit-dir <path> [--output-dir <path>] [--scheme <scheme>] [--oracle-hash <hash>] [--no-optimized]

Options:
  --circuit-dir   Noir circuit directory containing Nargo.toml
  --output-dir    Output directory for verifier artifacts
  --scheme        Barretenberg scheme (default: ultra_honk)
  --oracle-hash   Oracle hash for VK generation (default: keccak)
  --no-optimized  Disable optimized Solidity verifier generation
  -h, --help      Show this help
EOF
}

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
VERIFIER_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"

CIRCUIT_DIR=""
OUTPUT_DIR=""
SCHEME="ultra_honk"
ORACLE_HASH="keccak"
OPTIMIZED="1"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --circuit-dir)
      CIRCUIT_DIR="${2:-}"
      shift 2
      ;;
    --output-dir)
      OUTPUT_DIR="${2:-}"
      shift 2
      ;;
    --scheme)
      SCHEME="${2:-}"
      shift 2
      ;;
    --oracle-hash)
      ORACLE_HASH="${2:-}"
      shift 2
      ;;
    --no-optimized)
      OPTIMIZED="0"
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown argument: $1" >&2
      usage >&2
      exit 1
      ;;
  esac
done

if [[ -z "${CIRCUIT_DIR}" ]]; then
  echo "--circuit-dir is required" >&2
  usage >&2
  exit 1
fi

CIRCUIT_DIR="$(cd "${CIRCUIT_DIR}" && pwd)"
NARGO_TOML="${CIRCUIT_DIR}/Nargo.toml"

if [[ ! -f "${NARGO_TOML}" ]]; then
  echo "Could not find Nargo.toml at ${NARGO_TOML}" >&2
  exit 1
fi

PACKAGE_NAME="$(awk -F '=' '/^name[[:space:]]*=/{gsub(/[ "]/,"",$2); print $2; exit}' "${NARGO_TOML}")"
if [[ -z "${PACKAGE_NAME}" ]]; then
  echo "Could not derive package name from ${NARGO_TOML}" >&2
  exit 1
fi

if [[ -z "${OUTPUT_DIR}" ]]; then
  OUTPUT_DIR="${VERIFIER_DIR}/generated/${PACKAGE_NAME}"
fi
mkdir -p "${OUTPUT_DIR}"

echo "==> Compiling Noir circuit: ${PACKAGE_NAME}"
(
  cd "${CIRCUIT_DIR}"
  nargo compile --skip-brillig-constraints-check
)

BYTECODE_PATH="${CIRCUIT_DIR}/target/${PACKAGE_NAME}.json"
VK_PATH="${OUTPUT_DIR}/vk"
VERIFIER_SOL="${OUTPUT_DIR}/Verifier.sol"

if [[ ! -f "${BYTECODE_PATH}" ]]; then
  echo "Expected bytecode not found at ${BYTECODE_PATH}" >&2
  exit 1
fi

echo "==> Writing verification key"
bb write_vk \
  -s "${SCHEME}" \
  -b "${BYTECODE_PATH}" \
  -o "${VK_PATH}" \
  --oracle_hash "${ORACLE_HASH}"

echo "==> Writing Solidity verifier"
BB_VERIFIER_ARGS=(
  write_solidity_verifier
  -s "${SCHEME}"
  -k "${VK_PATH}"
  -o "${VERIFIER_SOL}"
)

if [[ "${OPTIMIZED}" == "1" ]]; then
  BB_VERIFIER_ARGS+=(--optimized)
fi

bb "${BB_VERIFIER_ARGS[@]}"

cat <<EOF

Shared verifier artifacts generated.

Package:        ${PACKAGE_NAME}
Circuit dir:    ${CIRCUIT_DIR}
Bytecode path:  ${BYTECODE_PATH}
VK path:        ${VK_PATH}
Verifier sol:   ${VERIFIER_SOL}

Next step:
  ./verifier/scripts/deploy_shared_verifier.sh \\
    --verifier-sol "${VERIFIER_SOL}" \\
    --contract-name ZusVerifier \\
    --rpc-url "\$RPC_URL" \\
    --private-key "\$PRIVATE_KEY"
EOF
