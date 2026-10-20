#!/usr/bin/env bash
# ============================================================================
# Polaris (fork) - Frontend (polaris-web / Vue + Vite) bauen.
# Erzeugt polaris-web/dist, das spaeter ins Docker-Image kopiert wird.
#   ./build_web.sh
# ============================================================================
set -euo pipefail
cd "$(dirname "$0")/polaris-web"

echo "==> Frontend (Vue) bauen ..."

# Abhaengigkeiten nur installieren, wenn noch nicht vorhanden (idempotent).
# `npm install` statt `npm ci`, weil das Lockfile nach dem Pin (vue3-apexcharts)
# momentan von package.json abweichen kann.
if [ ! -d node_modules ]; then
  echo "==> node_modules fehlt: npm install --no-audit --no-fund ..."
  npm install --no-audit --no-fund
else
  echo "==> node_modules vorhanden - npm install uebersprungen."
fi

if [ ! -d node_modules ]; then
  echo "FEHLER: npm install hat node_modules nicht erzeugt." >&2
  exit 1
fi

npm run build

if [ ! -d dist ]; then
  echo "FEHLER: polaris-web/dist wurde nicht erzeugt." >&2
  exit 1
fi

echo "==> OK: Frontend-Build vorhanden unter polaris-web/dist/."