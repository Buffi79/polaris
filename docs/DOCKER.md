# Polaris im Docker

Das Image ist bewusst ein **schlankes Runtime-Image**: Backend und Frontend
werden NICHT in Docker gebaut, sondern zuerst **auf dem Host** vorgebaut und
anschließend ins Image kopiert. Das beschleunigt den Docker-Build erheblich und
nutzt die bereits vorhandenen Build-Artefakte (`target/release/polaris` und
`polaris-web/dist/`).

## Voraussetzungen

- Docker (mit BuildKit, standard ab Docker 23+)
- Für das Vor-Bauen: Rust-Toolchain (rustup stable) und Node.js/npm (node 20+)
- Netzwerkzugriff beim Vor-Bauen, da Cargo/npm Dependencies nachladen

## Bauen

Alles in einem Schritt (baut Backend → Frontend → Docker-Image):

```bash
./build_docker.sh
```

Einzeln:

```bash
./build_backend.sh    # cargo build --release  -> target/release/polaris
./build_web.sh        # (npm install;) npm run build -> polaris-web/dist/
docker build -t polaris:latest .
```

Eigenes Image-Tag:

```bash
IMAGE_TAG=polaris:0.1.0 ./build_docker.sh
```

> Wichtig: `target` und `polaris-web/dist` müssen vor `docker build` existieren —
> das Dockerfile kopiert diese Artefakte, es baut nichts selbst (siehe
> `.dockerignore`, das diese Pfade bewusst NICHT ausschließt).

## Starten (docker compose)

```bash
./build_docker.sh
docker compose up -d
```

(Nur das Image neu bauen, ohne Container zu starten: `docker compose build`.)

## Starten (docker run)

```bash
docker run -d --name polaris \
  -p 5050:5050 \
  -v polaris-data:/var/lib/polaris \
  -v /pfad/zu/deiner/musik:/music:ro \
  polaris:latest
```

Danach im Browser: **http://localhost:5050**

Beim ersten Start erscheint der **Initial-Setup**-Flow (Admin-Benutzer anlegen
und Musikverzeichnis als Mount hinzufügen — Ziel z. B. `/music`).

## Konfiguration

| Zweck | Standard | Überschreiben |
|---|---|---|
| Port | `5050` | env `POLARIS_PORT` oder CLI `--port` |
| Web-Client | `/usr/share/polaris/web` | env `POLARIS_WEB_DIR` |
| Daten (polearis.toml, Index, Playlists) | `/var/lib/polaris` | env `POLARIS_DATA_DIR` |
| Musik | Mount unter `/music` | beliebig, im UI als Mount registrieren |

Alle Verzeichnisse lassen sich auch per CLI-Flags (`--data`, `--web`, `--cache`,
`--db`, `--config`, `--port`) setzen — z. B. wenn Docker-Env-Vars nicht
ausreichen: `docker run polaris:latest --port 8080`.

> Hinweis: Der Container läuft mit `--foreground` (images so konzipiert, dass
> der Prozess PID 1 bleibt) — Polaris würde sonst daemonisieren und der
> Container sofort beenden.

## Sonos (Ausblick)

Die Sonos-Konfiguration (sonos-node-http-api URL + MP3/SMB-Server) ist bewusst
**nicht** im Image hartkodiert — sie wird nach dem ersten Start über
**Settings → Sonos** in der Weboberfläche (bzw. in `polaris.toml`) gesetzt.
Sobald die Sonos-Integration finalisiert ist, gehören die CIFS/SMB-Details zu
diesem Abschnitt.