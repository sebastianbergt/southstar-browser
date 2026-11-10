#!/usr/bin/env python3
# cargo-sources.py — writes flatpak-builder sources for every crates.io package in Cargo.lock, verified by the lock's checksums.
# Copyright 2026 Andreas Røsdal
# SPDX-License-Identifier: NSL-1.0 OR GPL-3.0-or-later
import json
import sys
import tomllib

def main(lock_path, out_path):
    with open(lock_path, "rb") as lock_file:
        lock = tomllib.load(lock_file)
    sources = []
    for package in lock.get("package", []):
        source = package.get("source")
        if source is None:
            continue
        if source != "registry+https://github.com/rust-lang/crates.io-index":
            sys.exit(f"unsupported source for {package['name']}: {source}")
        name, version, checksum = package["name"], package["version"], package["checksum"]
        dest = f"cargo/vendor/{name}-{version}"
        sources.append({
            "type": "archive",
            "archive-type": "tar-gzip",
            "url": f"https://static.crates.io/crates/{name}/{name}-{version}.crate",
            "sha256": checksum,
            "dest": dest,
        })
        sources.append({
            "type": "inline",
            "contents": json.dumps({"package": checksum, "files": {}}),
            "dest": dest,
            "dest-filename": ".cargo-checksum.json",
        })
    sources.append({
        "type": "inline",
        "contents": "[source.crates-io]\nreplace-with = \"vendored-sources\"\n\n"
        "[source.vendored-sources]\ndirectory = \"cargo/vendor\"\n\n[net]\noffline = true\n",
        "dest": "cargo",
        "dest-filename": "config.toml",
    })
    with open(out_path, "w") as out:
        json.dump(sources, out, indent=1)
        out.write("\n")

if __name__ == "__main__":
    main(sys.argv[1], sys.argv[2])
