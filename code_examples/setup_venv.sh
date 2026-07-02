#!/usr/bin/env bash
# Create code_examples/.venv and install deps + kappa_apk (editable via maturin).
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
VENV="${SCRIPT_DIR}/.venv"

create_venv() {
  if command -v uv >/dev/null 2>&1; then
    echo "[1/4] Creating venv with uv at ${VENV}"
    uv venv "${VENV}" --python python3.12
    return 0
  fi
  echo "[1/4] Creating venv with python3 -m venv at ${VENV}"
  python3 -m venv "${VENV}" || {
    echo "Failed: install python3-venv (e.g. apt install python3.12-venv) or install uv." >&2
    exit 1
  }
}

install_requirements() {
  if command -v uv >/dev/null 2>&1; then
    echo "[2/4] Installing Python deps with uv pip"
    uv pip install -r "${SCRIPT_DIR}/requirements.txt" --python "${VENV}/bin/python"
    return
  fi
  echo "[2/4] Upgrading pip and installing requirements"
  "${VENV}/bin/pip" install --upgrade pip
  "${VENV}/bin/pip" install -r "${SCRIPT_DIR}/requirements.txt"
}

create_venv
install_requirements

echo "[3/4] Building and installing kappa_apk (maturin develop)"
cd "${ROOT_DIR}"
export VIRTUAL_ENV="${VENV}"
"${VENV}/bin/maturin" develop --release

echo ""
echo "[4/4] Verify install"
"${VENV}/bin/python" -c "
import kappa_apk
import torch
print('kappa_apk OK — version:', kappa_apk.version())
print('torch', torch.__version__)
for cls in ('KappaDataset', 'KappaDataLoader', 'DataLoaderHelper'):
    assert hasattr(kappa_apk, cls), f'Missing export: {cls}'
print('loader exports OK')
"

echo ""
echo "Done. Activate and run the MNIST training example:"
echo "  source ${VENV}/bin/activate"
echo "  python mnist_kappa_training_example.py --base-url URL --login-id USER --password '***'"
