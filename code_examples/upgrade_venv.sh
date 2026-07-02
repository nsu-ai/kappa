#!/usr/bin/env bash
# Refresh code_examples/.venv: Python deps + latest kappa_apk from this repo.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
VENV="${SCRIPT_DIR}/.venv"
PY="${VENV}/bin/python"

if [[ ! -x "${PY}" ]]; then
  echo "No venv at ${VENV}. Run ./code_examples/setup_venv.sh first." >&2
  exit 1
fi

echo "[1/3] Upgrade Python dependencies"
if command -v uv >/dev/null 2>&1; then
  uv pip install --upgrade -r "${SCRIPT_DIR}/requirements.txt" --python "${PY}"
else
  "${PY}" -m ensurepip --upgrade 2>/dev/null || true
  "${PY}" -m pip install --upgrade pip
  "${PY}" -m pip install --upgrade -r "${SCRIPT_DIR}/requirements.txt"
fi

echo "[2/3] Rebuild and install kappa_apk (maturin develop --release)"
cd "${ROOT_DIR}"
export VIRTUAL_ENV="${VENV}"
"${VENV}/bin/maturin" develop --release

echo "[3/3] Verify"
"${PY}" -c "
import kappa_apk
print('kappa_apk version:', kappa_apk.version())
try:
    import torch
    print('torch', torch.__version__)
except ImportError:
    pass
try:
    import matplotlib
    print('matplotlib', matplotlib.__version__)
except ImportError:
    pass
for cls in ('KappaDataset', 'KappaDataLoader', 'DataLoaderHelper'):
    assert hasattr(kappa_apk, cls), f'Missing export: {cls}'
print('loader exports OK')
"

echo ""
echo "Done. Activate: source ${VENV}/bin/activate"
echo "Run: python mnist_kappa_training_example.py --base-url URL --login-id USER --password '***'"
