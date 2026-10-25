#!/usr/bin/env bash
# Copy the game data files the source code loads from local Wonderland installs
# into assets/<GAME> (gitignored).
#
# Kept:    res*.dat (pack index/data + direct-loaded textures, music, levels),
#          story.dat (WSW), *.lev (TOW bonus levels), extra.cfg (TOW),
#          options.cfg and players/ (settings and save profiles).
# Skipped: executables, shortcuts, uninstallers, leftover pack *.tmp extracts,
#          ldr*.dat (not referenced by the source), readme/license, screenshots.
#
# Usage: tools/import-assets.sh [SOURCE_ROOT]   (default: /mnt/c/Games/Wonderland)
set -euo pipefail

SRC_ROOT="${1:-/mnt/c/Games/Wonderland}"
REPO="$(cd "$(dirname "$0")/.." && pwd)"
DEST_ROOT="$REPO/assets"

# Map an install folder name to the game code folder in this repo.
game_for() {
	local n
	n="$(echo "$1" | tr '[:upper:]' '[:lower:]')"
	case "$n" in
		*secret*|*wsw*)             echo WSW ;;
		*return*|*rtw*|*platinum*)  echo RTWP ;;
		*original*|*tow*|wonderland|"wonderland v"*) echo TOW ;;
		*)                          echo "" ;;
	esac
}

[ -d "$SRC_ROOT" ] || { echo "Source not found: $SRC_ROOT" >&2; exit 1; }

for src in "$SRC_ROOT"/*/; do
	name="$(basename "$src")"
	game="$(game_for "$name")"
	if [ -z "$game" ]; then
		echo "?? Skipping '$name': can't tell which game it is"
		continue
	fi
	dest="$DEST_ROOT/$game"
	echo "=== $name -> assets/$game"
	mkdir -p "$dest"
	find "$src" -maxdepth 1 -type f \( -iname 'res*.dat' -o -iname 'story.dat' \
		-o -iname '*.lev' -o -iname 'extra.cfg' -o -iname 'options.cfg' \) \
		-exec cp -p {} "$dest/" \;
	[ -d "$src/players" ] && cp -rp "$src/players" "$dest/"
	echo "   $(find "$dest" -type f | wc -l) files ($(du -sh "$dest" | cut -f1))"
done

# Sanity check: pack files every game loads at startup.
for game in RTWP WSW TOW; do
	for f in res1.dat res2.dat; do
		[ -f "$DEST_ROOT/$game/$f" ] || echo "   MISSING: assets/$game/$f"
	done
done
echo "Done."
