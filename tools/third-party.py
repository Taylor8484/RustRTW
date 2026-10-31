#!/usr/bin/env python3
"""Writes THIRD-PARTY.txt for a target: each crate in the game, and the
licence files those crates ship.

Usage: third-party.py <target triple> <output file>
"""
import hashlib
import json
import pathlib
import subprocess
import sys

target, out = sys.argv[1:3]
tree = subprocess.run(
    ["cargo", "tree", "-p", "wonderland-game", "-e", "normal", "--target", target,
     "--prefix", "none", "-f", "{p}"],
    check=True, capture_output=True, text=True).stdout
used = set()
for line in tree.splitlines():
    name, version = line.split()[:2]
    used.add((name, version.lstrip("v")))

meta = json.loads(subprocess.run(
    ["cargo", "metadata", "--format-version", "1", "--filter-platform", target],
    check=True, capture_output=True, text=True).stdout)

listing = []
texts = {}
for p in sorted(meta["packages"], key=lambda p: (p["name"], p["version"])):
    if (p["name"], p["version"]) not in used or p["name"].startswith("wonderland-"):
        continue
    listing.append(f'{p["name"]} {p["version"]}: {p.get("license") or "see its licence file"}')
    folder = pathlib.Path(p["manifest_path"]).parent
    for f in sorted(folder.iterdir()):
        if f.is_file() and f.name.upper().startswith(("LICENSE", "LICENCE", "COPYING", "NOTICE")):
            body = f.read_text(errors="replace").strip()
            key = hashlib.sha256(body.encode()).hexdigest()
            texts.setdefault(key, (body, []))[1].append(p["name"])

with open(out, "w", newline="\n") as o:
    o.write("Third-party libraries\n=====================\n\n")
    o.write("\n".join(listing) + "\n")
    for body, names in texts.values():
        o.write("\n" + "=" * 72 + "\n" + ", ".join(sorted(set(names))) + "\n\n" + body + "\n")
print(f"{len(listing)} crates, {len(texts)} licence texts")
