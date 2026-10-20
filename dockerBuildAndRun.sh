#!/usr/bin/env bash
# ============================================================================
# Polaris (fork) - Docker-Image bauen und Container starten
# Entspricht buildAndRun.sh, nur für Docker.
#
#   ./dockerBuildAndRun.sh          # baut und startet (mit Standard-Werten)
#   MUSIC_DIR=/pfad/zur/musik ./dockerBuildAndRun.sh
#
# Benötigt: docker (mit BuildKit, Standard bei docker compose v2 / 23+)
# ============================================================================
set -euo pipefail
cd "$(dirname "$0")"

# ---- Konfigurierbar ----
IMAGE_TAG="${IMAGE_TAG:-polaris:latest}"
CONTAINER_NAME="${CONTAINER_NAME:-polaris}"
HOST_PORT="${HOST_PORT:-5050}"
VOLUME_NAME="${VOLUME_NAME:-polaris-data}"
MUSIC_DIR="${MUSIC_DIR:-./music}"

# ---- Bild bauen ----
echo "==> Docker-Image '${IMAGE_TAG}' bauen ..."
docker build -t "${IMAGE_TAG}" .

# ---- Laufenden Container zuerst stoppen/entfernen (falls vorhanden) ----
if docker ps -a --format '{{.Names}}' | grep -qx "${CONTAINER_NAME}"; then
  echo "==> Bereits laufender Container '${CONTAINER_NAME}' wird ersetzt ..."
  docker rm -f "${CONTAINER_NAME}" >/dev/null
fi

# ---- Music-Verzeichnis vorbereiten ----
if [ ! -d "${MUSIC_DIR}" ]; then
  echo "==> Music-Verzeichnis '${MUSIC_DIR}' existiert nicht - wird angelegt (leer)."
  mkdir -p "${MUSIC_DIR}"
fi
MUSIC_ABS="$(realpath "${MUSIC_DIR}")"

# ---- Container starten ----
echo "==> Container '${CONTAINER_NAME}' starten ..."
docker run -d --name "${CONTAINER_NAME}" \
  -p "${HOST_PORT}:5050" \
  -v "${VOLUME_NAME}:/var/lib/polaris" \
  -v "${MUSIC_ABS}:/music:ro" \
  --restart unless-stopped \
  "${IMAGE_TAG}"

echo "==> Fertig."
echo "    UI:   http://localhost:${HOST_PORT}"
echo "    Logs: docker logs -f ${CONTAINER_NAME}"