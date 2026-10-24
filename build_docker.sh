#!/usr/bin/env bash
# ============================================================================
# Polaris (fork) - Docker-Image bauen.
#
# Backend und Frontend werden NICHT in Docker gebaut, sondern zuerst auf dem
# Host vorgebaut (build_backend.sh + build_web.sh) und anschliessend von dem
# schlanken Runtime-Image einfach kopiert.
#
#   ./build_docker.sh                       # Image 'polaris:latest'
#   IMAGE_TAG=polaris:2.0 ./build_docker.sh # eigenes Tag
#
# Starten des Containers erfolgt ueber docker compose:
#   docker compose up -d
# (Erstinstellungen/Neubau:  docker compose up -d --build)
# ============================================================================
set -euo pipefail
cd "$(dirname "$0")"

IMAGE_TAG="${IMAGE_TAG:-polaris:latest}"

echo "==> Stufe 1: Backend bauen ..."
#./build_backend.sh

echo "==> Stufe 2: Frontend bauen ..."
#./build_web.sh

echo "==> Stufe 3: Docker-Image '${IMAGE_TAG}' bauen ..."
docker build -t "${IMAGE_TAG}" .

echo "==> Fertig."
echo "    Image:  ${IMAGE_TAG}"
echo "    Start:  docker compose up -d"
echo "    UI:     http://localhost:5050"
