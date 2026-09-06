#!/usr/bin/env bash
# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 html-generator contributors. All rights reserved.
set -euo pipefail

# Assert every version-bearing file in this repository agrees with the
# crate version, before a tag exists.
#
# Adopted from noyalib in the v0.0.7 cycle, after this repository had
# shipped 0.0.6 with a CHANGELOG that stopped at 0.0.5 and, twice, had
# `Cargo.toml` move to a new version without a tag or a release. Every
# file that carries the version is checked here so that drift fails
# locally in a second instead of surfacing after a tag exists.
#
# Usage:
#   scripts/verify-release-versions.sh            # against Cargo.toml
#   scripts/verify-release-versions.sh 0.0.29     # against an intended version
#   scripts/verify-release-versions.sh v0.0.29    # a leading v is accepted

cd "$(cd "$(dirname -- "$0")/.." && pwd)"
exec python3 - "${1:-}" <<'PY'
import json, pathlib, re, sys

root = pathlib.Path.cwd()
wanted = sys.argv[1].lstrip("v") if len(sys.argv) > 1 and sys.argv[1] else ""

GREEN, RED, DIM, OFF = "\033[32m", "\033[31m", "\033[2m", "\033[0m"
failed = []


def ok(what, detail):
    print(f"  {GREEN}ok{OFF}    {what:<34} {detail}")


def bad(what, detail):
    print(f"  {RED}FAIL{OFF}  {what:<34} {detail}")
    failed.append(what)


def skip(what, detail):
    print(f"  {DIM}·     {what:<34} {detail}{OFF}")


def package_field(key):
    """Read a key from the [package] table of the crate's manifest."""
    for candidate in (root / "crates" / root.name / "Cargo.toml", root / "Cargo.toml"):
        if not candidate.is_file():
            continue
        text = candidate.read_text(encoding="utf-8")
        block = re.search(r"^\[package\](.*?)(?=^\[|\Z)", text, re.S | re.M)
        if not block:
            continue
        m = re.search(rf'^{key}\s*=\s*"([^"]+)"', block.group(1), re.M)
        if m:
            return m.group(1)
    return ""


version, name = package_field("version"), package_field("name")
if not version:
    sys.exit("Could not read the crate version from Cargo.toml")

if wanted and wanted != version:
    sys.exit(
        f"Cargo.toml says {version}, you asked for {wanted}.\n"
        "Bump Cargo.toml first — it is the source of truth."
    )

print(f"Verifying every version-bearing file against {name} {version}\n")


def locked_version(pkg):
    lock = root / "Cargo.lock"
    if not lock.is_file():
        return None
    m = re.search(rf'^name = "{re.escape(pkg)}"\nversion = "([^"]+)"',
                  lock.read_text(encoding="utf-8"), re.M)
    return m.group(1) if m else None


# Cargo.lock — this crate's own entry.
own = locked_version(name)
if own is None:
    skip("Cargo.lock", f"no entry for {name}")
elif own == version:
    ok("Cargo.lock", own)
else:
    bad("Cargo.lock", f"{own} — run cargo check to refresh")

# noyalib and dtt are pinned exactly (`=0.0.X`): same-author 0.0.x lines where
# every release may break. The pin and the lock entry must agree, or CI,
# which builds --locked, fails on a stale lock.
manifest = (root / "Cargo.toml").read_text(encoding="utf-8") if (root / "Cargo.toml").is_file() else ""
for dep in ("noyalib", "dtt"):
    pin = re.search(rf'^{dep}\s*=\s*(?:"=([^"]+)"|\{{\s*version\s*=\s*"=([^"]+)")', manifest, re.M)
    if not pin:
        continue
    pinned = pin.group(1) or pin.group(2)
    locked = locked_version(dep)
    if locked is None:
        skip(f"Cargo.lock {dep}", "absent")
    elif locked == pinned:
        ok(f"Cargo.toml {dep} pin", f"={pinned}, lock agrees")
    else:
        bad(f"Cargo.lock {dep}", f"{locked} — pin says ={pinned}; CI builds --locked")

# JSON manifests that carry a version of their own.
#
# Only the repository root is checked; none exist here today, and the
# loop costs nothing until one does.
for fname in ("server.json", "glama.json", "package.json"):
    f = root / fname
    if not f.is_file():
        continue
    try:
        got = json.loads(f.read_text(encoding="utf-8")).get("version", "")
    except json.JSONDecodeError as e:
        bad(fname, f"invalid JSON: {e}")
        continue
    if not got:
        skip(fname, "no version field")
    elif got == version:
        ok(fname, got)
    else:
        bad(fname, got)

# Container image tags embedded in those manifests move with the
# version, or the registry entry points at the previous image.
for fname in ("server.json", "glama.json"):
    f = root / fname
    if not f.is_file():
        continue
    for ref in sorted(set(re.findall(r"ghcr\.io/[\w./-]+:\d+\.\d+\.\d+", f.read_text(encoding="utf-8")))):
        (ok if ref.rsplit(":", 1)[1] == version else bad)(f"{fname} image tag", ref)

# The changelog must have promoted this version out of [Unreleased].
# CITATION.cff carries its own version field (added in the v0.0.31
# cycle); a release prep that skips it ships a stale citation.
citation = root / "CITATION.cff"
if citation.is_file():
    text = citation.read_text(encoding="utf-8")
    m = re.search(r"^version: (\S+)$", text, re.M)
    if m and m.group(1) == version:
        ok("CITATION.cff", f"version {m.group(1)}")
    else:
        bad("CITATION.cff", f"says {m.group(1) if m else 'nothing'} — update the version field")

changelog = root / "CHANGELOG.md"
if changelog.is_file():
    if re.search(rf"^## \[v?{re.escape(version)}\]", changelog.read_text(encoding="utf-8"), re.M):
        ok("CHANGELOG.md", f"has a [v{version}] section")
    else:
        bad("CHANGELOG.md", f"no [v{version}] heading — still under [Unreleased]?")

# Install snippets naming this crate: the README and every page under
# docs/.
snippet_files = [root / "README.md", root / "crates" / name / "README.md"]
snippet_files += sorted((root / "docs").rglob("*.md")) if (root / "docs").is_dir() else []
for f in snippet_files:
    rel = str(f.relative_to(root))
    # Release notes and decision records quote the versions of their
    # own day; they are history, not install instructions.
    if not f.is_file() or "release-notes" in rel or "/adr/" in rel:
        continue
    stale = set()
    for line in f.read_text(encoding="utf-8").splitlines():
        if re.search(rf"\b{re.escape(name)}\s*=\s*[\"{{]", line):
            stale.update(v for v in re.findall(r"\d+\.\d+\.\d+", line) if v != version)
    if stale:
        bad(rel, "mentions " + ", ".join(sorted(stale)))
    else:
        ok(rel, "install snippets current")

print()
if failed:
    print("Version mismatch. Fix these before tagging — a tag that fails the")
    print("release workflow's Validate job must be deleted and recreated.")
    sys.exit(1)
print(f"All version-bearing files agree on {version}.")
PY
