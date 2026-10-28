#!/usr/bin/env python3
"""Collect installed dependency license notices for an executable distribution.

Run cargo metadata with --locked --filter-platform for the distribution target
and pass its saved JSON here. Install the pinned web dependencies first. The
notice set deliberately includes build dependencies as a conservative superset.
It is a notice collector, not a substitute for the dependency-policy CI gate.
"""
import argparse
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

# These published workspace subcrates omit their repository-root license files.
# Pin both version and .cargo_vcs_info.json revision: never apply an old license
# fallback silently to a newly upgraded dependency. License wording is retained;
# trailing line whitespace is normalized in vendored/generated notice text.
ASN1_COMMIT = "a20e5f7319c896737ad0f2557037817b91ad854f"
WEBAUTHN_COMMIT = "d2c10d53ca5ef033d37ee6462e936e9eb72ad98c"
RUST_NOTICE_FALLBACKS = {
    ("asn1-rs-impl", "0.2.0"): (
        ASN1_COMMIT,
        [
            ("docs/third-party/asn1-rs-impl-0.2.0-LICENSE-MIT",
             f"https://raw.githubusercontent.com/rusticata/asn1-rs/{ASN1_COMMIT}/LICENSE-MIT",
             "a5c61b93b6ee1d104af9920cf020ff3c7efe818e31fe562c72261847a728f513"),
            ("docs/third-party/asn1-rs-impl-0.2.0-LICENSE-APACHE",
             f"https://raw.githubusercontent.com/rusticata/asn1-rs/{ASN1_COMMIT}/LICENSE-APACHE",
             "a60eea817514531668d7e00765731449fe14d059d3249e0bc93b36de45f759f2"),
        ],
    ),
    **{
        (name, "0.5.5"): (
            WEBAUTHN_COMMIT,
            [("docs/third-party/webauthn-rs-0.5.5-LICENSE.md",
              f"https://raw.githubusercontent.com/kanidm/webauthn-rs/{WEBAUTHN_COMMIT}/LICENSE.md",
              "393eaf2b92b3d49bea139e39b6fd205b6e99c499999986aec5554586a830dfdd")],
        )
        for name in (
            "base64urlsafedata", "webauthn-attestation-ca", "webauthn-rs",
            "webauthn-rs-core", "webauthn-rs-proto",
        )
    },
}

# The upstream MPL file has one trailing space on line 38. Record its original
# checksum separately from the whitespace-normalized copy retained in git.
NORMALIZED_UPSTREAM_HASHES = {
    "docs/third-party/webauthn-rs-0.5.5-LICENSE.md":
        "32ee9dbf6196874fc9d406c54a888a6c4cbb9aa4a7f35b46befeaff43a78fe85",
}

# The native binding manifests declare this exact parent package's repository,
# license and version, and the parent declares them as exact-version optional
# dependencies. These bytes were compared with the official npm tarball after
# verifying its SHA-512 integrity against web/package-lock.json.
ROLLDOWN_BINDINGS = {
    "@rolldown/binding-linux-x64-gnu", "@rolldown/binding-linux-x64-musl",
}
ROLLDOWN_VERSION = "1.2.15"
ROLLDOWN_TARBALL = "https://registry.npmjs.org/rolldown/-/rolldown-1.2.15.tgz"
ROLLDOWN_NOTICES = [
    ("web/node_modules/rolldown/LICENSE", f"{ROLLDOWN_TARBALL}#package/LICENSE",
     "23ecfff35a5a2e80d92142f75228912c3b1abc4b5a8337a821ff4397e2f9f734"),
    ("web/node_modules/rolldown/THIRD-PARTY-LICENSE", f"{ROLLDOWN_TARBALL}#package/THIRD-PARTY-LICENSE",
     "a877291d800ed43692f3f9ae09d8e01cc6f7293ad39d43896059c188ffbb8b7c"),
]


def verified_fallback(entries, basis):
    files, sources = [], []
    for relative, source, expected in entries:
        path = ROOT / relative
        actual = hashlib.sha256(path.read_bytes()).hexdigest()
        if actual != expected:
            raise ValueError(f"Pinned dependency notice checksum mismatch: {relative}")
        files.append(path)
        record = {"file": relative, "source": source, "sha256": actual, "basis": basis}
        if relative in NORMALIZED_UPSTREAM_HASHES:
            record["upstream_sha256"] = NORMALIZED_UPSTREAM_HASHES[relative]
            record["normalization"] = "One trailing space removed; license wording unchanged"
        sources.append(record)
    return files, sources


def notices(directory):
    found = []
    for path in directory.iterdir():
        lower = path.name.lower()
        if path.is_file() and lower.startswith(("license", "licence", "copying", "notice", "third-party-license", "third_party_license")):
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
            sources = []
            explicit = package.get("license_file")
            if explicit:
                path = Path(explicit)
                if not path.is_absolute():
                    path = directory / path
                if path.is_file() and path not in files:
                    files.append(path)
            if not files and (package["name"], package["version"]) == ("eventsource-stream", "0.2.3"):
                files = [ROOT / "docs/third-party/eventsource-stream-LICENSE-APACHE"]
            fallback = RUST_NOTICE_FALLBACKS.get((package["name"], package["version"]))
            if not files and fallback:
                commit, entries = fallback
                vcs = json.loads((directory / ".cargo_vcs_info.json").read_text())
                if vcs.get("git", {}).get("sha1") != commit:
                    raise ValueError(f"Unexpected published source revision for {package['name']}")
                files, sources = verified_fallback(entries, f"Repository-root license at the published crate's recorded commit {commit}")
            packages.append(("Rust", package["name"], package["version"],
                             package.get("license") or "See bundled notices", files, sources))
    modules = ROOT / "web/node_modules"
    for manifest in [*modules.glob("*/package.json"), *modules.glob("@*/*/package.json")]:
        if manifest.parent.name.startswith("."):
            continue
        package = json.loads(manifest.read_text())
        files, sources = notices(manifest.parent), []
        if not files and package["name"] in ROLLDOWN_BINDINGS and package["version"] == ROLLDOWN_VERSION:
            parent = json.loads((modules / "rolldown/package.json").read_text())
            if (parent["version"] != package["version"]
                    or parent.get("repository") != package.get("repository")
                    or parent.get("license") != package.get("license")
                    or parent.get("optionalDependencies", {}).get(package["name"]) != package["version"]):
                raise ValueError(f"Rolldown binding does not match its notice-bearing parent: {package['name']}")
            files, sources = verified_fallback(ROLLDOWN_NOTICES, "Matching rolldown 1.2.15 parent package; same repository/license and exact-version native optional dependency")
        packages.append(("Web", package["name"], package["version"], str(package.get("license", "See bundled notices")), files, sources))
    lines = ["# Third-party dependency notices", "", "Generated from the locked Linux build dependency metadata and installed WebUI packages. This includes a conservative superset of development dependencies; their inclusion does not imply that all are distributed in the runtime. Refer to Cargo.lock and web/package-lock.json for exact source identities. Each dependency retains its own license. RstrBot's MIT license does not replace these terms.", ""]
    missing = []
    fallback_sources = []
    for ecosystem, name, version, license_name, files, sources in sorted(packages, key=lambda package: package[:3]):
        lines.extend([f"## {ecosystem}: {name} {version}", "", f"Declared license: {license_name}", ""])
        if sources:
            fallback_sources.append({"package": f"{ecosystem}:{name}@{version}", "notices": sources})
            for source in sources:
                lines.extend([f"Notice source: [{source['file']}]({source['source']}). {source['basis']}.",
                              f"Retained notice SHA-256: `{source['sha256']}`.", ""])
                if "upstream_sha256" in source:
                    lines.extend([f"Upstream file SHA-256: `{source['upstream_sha256']}`. {source['normalization']}.", ""])
        if not files:
            missing.append(f"{ecosystem}:{name}@{version}")
            lines.extend(["The installed package did not contain a top-level license notice. Consult the corresponding registry source and declared license.", ""])
        for path in files:
            body = "\n".join(line.rstrip() for line in path.read_text(errors="replace").splitlines()).rstrip()
            lines.extend([f"### {path.name}", "", "```text", body, "```", ""])
    output = ROOT / "THIRD_PARTY_NOTICES.md"
    output.write_text("\n".join(lines))
    print(json.dumps({"packages": len(packages), "packages_without_top_level_notice": missing,
                      "fallback_sources": fallback_sources, "output": str(output)}))


if __name__ == "__main__":
    main()
