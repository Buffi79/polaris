# syntax=docker/dockerfile:1
# ============================================================================
# Polaris (fork) - Music streaming server + Web UI
#
# REIN RUNTIME-IMAGE. Es wird NICHTS in Docker gebaut.
# Backend (target/release/polaris) und Frontend (polaris-web/dist) werden auf
# dem Host vorgebaut und hier nur noch kopiert.
#
# Vor `docker build` also zwingend ausfuehren:
#   ./build_docker.sh   (ruft build_backend.sh + build_web.sh + docker build)
# oder manuell:
#   ./build_backend.sh && ./build_web.sh && docker build -t polaris:latest .
#
# Gestartet wird der Container via docker compose (siehe docker-compose.yml).
# ============================================================================
FROM debian:bookworm-slim

# Laufzeit-Bedarf: CA-Zertifikate (HTTPS) und tini als PID 1, damit Signale
# sauber durchgereicht werden.
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates tini \
    && rm -rf /var/lib/apt/lists/*

# Vorgebaute Artefakte vom Host (Build-Context).
COPY target/release/polaris /usr/local/bin/polaris
COPY polaris-web/dist /usr/share/polaris/web

# Persistente Daten (konfig, Index, Playlists) und die Musikbibliothek.
VOLUME ["/var/lib/polaris", "/music"]

ENV POLARIS_WEB_DIR=/usr/share/polaris/web \
    POLARIS_DATA_DIR=/var/lib/polaris \
    POLARIS_PORT=5050

EXPOSE 5050

COPY docker/entrypoint.sh /usr/local/bin/entrypoint.sh
RUN chmod +x /usr/local/bin/entrypoint.sh

ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]