#!/usr/bin/env bash
# Build a release package in dist/: the program, its readme, the list of game
# files, empty asset folders and the third-party licences. The game data stays
# out: the games' licences forbid distributing it, so players bring their own.
#
# license.txt must ship with the program. By its terms, publish packages only
# on the community forums.
#
# Usage: tools/package-release.sh linux|windows
# The Windows build runs the Windows toolchain, so run it from WSL or Windows.
set -euo pipefail

REPO="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO"
VERSION="$(grep -m1 '^version' crates/game/Cargo.toml | cut -d'"' -f2)"

case "${1:-}" in
	linux)
		cargo build --profile dist -p wonderland-game
		exe=target/dist/wonderland
		triple=x86_64-unknown-linux-gnu
		;;
	windows)
		cargo.exe build --profile dist -p wonderland-game --target-dir target/windows
		exe=target/windows/dist/wonderland.exe
		triple=x86_64-pc-windows-msvc
		;;
	*)
		echo "usage: $0 linux|windows" >&2
		exit 1
		;;
esac

name="wonderland-$VERSION-$1-x86_64"
stage="dist/$name"
rm -rf "$stage"
mkdir -p "$stage"
cp "$exe" "$stage/"
cp tools/dist-readme.txt "$stage/README.txt"
cp tools/dist-assets.txt "$stage/ASSETS.txt"
for game in RTWP TOW; do
	mkdir -p "$stage/assets/$game"
	echo "Put your game files here. ASSETS.txt lists them." >"$stage/assets/$game/PUT-GAME-FILES-HERE.txt"
done
python3 tools/third-party.py "$triple" "$stage/THIRD-PARTY.txt"
cp license.txt "$stage/"

python3 -c "import shutil, sys; shutil.make_archive(sys.argv[1], sys.argv[2], 'dist', sys.argv[3])" \
	"dist/$name" "$([ "$1" = windows ] && echo zip || echo gztar)" "$name"
ls -la dist
