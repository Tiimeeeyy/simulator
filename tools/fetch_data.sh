#!/usr/bin/env bash
# Fetch / refresh all external data used by ygopro-data.
# Network etiquette: YGOPRODeck is pulled ONCE (full DB) and cached.
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DATA="$ROOT/data"
CACHE="$ROOT/.cache"
mkdir -p "$DATA" "$CACHE"

echo ">> cards.cdb + strings.conf (ProjectIgnis)"
curl -fsSL -o "$DATA/cards.cdb" https://raw.githubusercontent.com/ProjectIgnis/BabelCDB/master/cards.cdb
curl -fsSL -o "$DATA/strings.conf" https://raw.githubusercontent.com/ProjectIgnis/Distribution/master/config/strings.conf

echo ">> historical F/L lists (ProjectIgnis/LFLists git history)"
if [ -d "$CACHE/LFLists/.git" ]; then git -C "$CACHE/LFLists" pull -q; else git clone -q https://github.com/ProjectIgnis/LFLists.git "$CACHE/LFLists"; fi
python3 "$ROOT/tools/extract_lflist_history.py" "$CACHE/LFLists" "$DATA/lflists"

echo ">> release dates (YGOPRODeck, single full pull, cached)"
if [ ! -f "$CACHE/cardinfo.json" ] || [ "${REFRESH_YGOPRODECK:-0}" = "1" ]; then
  curl -fsS --compressed -o "$CACHE/cardinfo.json" "https://db.ygoprodeck.com/api/v7/cardinfo.php?misc=yes"
fi
python3 "$ROOT/tools/build_release_dates.py" "$CACHE/cardinfo.json" "$DATA/release_dates.csv"

echo ">> CardScripts (ProjectIgnis)"
if [ -d "$CACHE/CardScripts/.git" ]; then git -C "$CACHE/CardScripts" pull -q; else git clone --depth 1 https://github.com/ProjectIgnis/CardScripts.git "$CACHE/CardScripts"; fi
ln -sfn ../.cache/CardScripts "$DATA/scripts"

