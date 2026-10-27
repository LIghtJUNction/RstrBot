#!/usr/bin/env python3
"""Collect installed dependency license notices for an executable distribution.

Run cargo metadata with --locked --filter-platform for the distribution target
and pass its saved JSON here. Install the pinned web dependencies first. The
notice set deliberately includes build dependencies as a conservative superset.
It is a notice collector, not a substitute for the dependency-policy CI gate.
"""
import argparse
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def notices(directory):
    found = []
    for path in directory.iterdir():
        lower = path.name.lower()
        if path.is_file() and lower.startswith(("license", "licence", "copying", "notice")):
            found.append(path)
        elif path.is_dir() and lower in ("licenses", "licences"):
            found.extend(child for child in path.rglob("*") if child.is_file())
    return sorted(found)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("metadata", type=Path)
    args = parser.parse_args()
    metadata = json.loads(args.metadata.read_text())
    packages = []
    for package in metadata["packages"]:
        if package.get("source"):
            directory = Path(package["manifest_path"]).parent
            files = notices(directory)
            if package["name"] == "eventsource-stream" and not files:
                files = [ROOT / "docs/third-party/eventsource-stream-LICENSE-APACHE"]
            explicit = package.get("license_file")
            if explicit:
                path = Path(explicit)
                if not path.is_absolute():
                    path = directory / path
                if path.is_file() and path not in files:
                    files.append(path)
            packages.append(("Rust", package["name"], package["version"],
                             package.get("license") or "See bundled notices", files))
    modules = ROOT / "web/node_modules"
    for manifest in modules.glob("*/package.json"):
        if manifest.parent.name.startswith("."):
            continue
        package = json.loads(manifest.read_text())
        packages.append(("Web", package["name"], package["version"], str(package.get("license", "See bundled notices")), notices(manifest.parent)))
    for manifest in modules.glob("@*/*/package.json"):
        package = json.loads(manifest.read_text())
        packages.append(("Web", package["name"], package["version"], str(package.get("license", "See bundled notices")), notices(manifest.parent)))
    lines = ["# Third-party dependency notices", "", "Generated from the locked Linux build dependency metadata and installed WebUI packages. This includes a conservative superset of development dependencies; their inclusion does not imply that all are distributed in the runtime. Refer to Cargo.lock and web/package-lock.json for exact source identities. Each dependency retains its own license. RstrBot's MIT license does not replace these terms.", ""]
    missing = []
    for ecosystem, name, version, license_name, files in sorted(packages):
        lines.extend([f"## {ecosystem}: {name} {version}", "", f"Declared license: {license_name}", ""])
        if not files:
            missing.append(f"{ecosystem}:{name}@{version}")
            lines.extend(["The installed package did not contain a top-level license notice. Consult the corresponding registry source and declared license.", ""])
        for path in files:
            lines.extend([f"### {path.name}", "", "```text", path.read_text(errors="replace").rstrip(), "```", ""])
    output = ROOT / "THIRD_PARTY_NOTICES.md"
    output.write_text("\n".join(lines))
    print(json.dumps({"packages": len(packages), "packages_without_top_level_notice": missing, "output": str(output)}))


if __name__ == "__main__":
    main()
