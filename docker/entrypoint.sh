#!/bin/sh
# ============================================================================
# Polaris entrypoint — startet den Server im Vordergrund (~foreground), damit
# PID 1 (tini) der Prozess bleibt und Docker SIGTERM/signals sauber durchreicht.
#
# Alle Pfade werden explizit per CLI-Flag gesetzt (nicht per option_env!-ENV,
# die nur zur Kompilierzeit greift). So landen Konfig, Index, Caches in den
# persistente Volumes, auch wenn der Build mit anderen Defaults erzeugt wurde.
# ============================================================================
set -eu

: "${POLARIS_PORT:=5050}"
: "${POLARIS_WEB_DIR:=/usr/share/polaris/web}"
: "${POLARIS_DATA_DIR:=/var/lib/polaris}"

exec /usr/local/bin/polaris \
  --foreground \
  --port "${POLARIS_PORT}" \
  --web "${POLARIS_WEB_DIR}" \
  --data "${POLARIS_DATA_DIR}" \
  --config "${POLARIS_DATA_DIR}/polaris.toml" \
  --database "${POLARIS_DATA_DIR}/db.sqlite" \
  --cache "${POLARIS_DATA_DIR}/cache"