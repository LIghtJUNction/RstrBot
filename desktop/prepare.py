#!/usr/bin/env python3
"""Stage native sidecars and the already-built web client for Tauri packaging.

Run on the native target after building the root workspace and WebUI. This does
not download a compiler, build an installer, or assert cross-platform support.
"""
import argparse
import hashlib
import json
from pathlib import Path
import shutil
import subprocess

ROOT = Path(__file__).resolve().parents[1]
DESKTOP = Path(__file__).resolve().parent
BINARIES = ('rstrbot', 'rstrbot-openai', 'rstrbot-diagnostic', 'rstrbot-webhook')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--profile', choices=('debug', 'release'), default='release')
    parser.add_argument('--target', help='Rust target triple; defaults to rustc host tuple')
    parser.add_argument('--binary-dir', type=Path, help='Directory containing the native root binaries')
    parser.add_argument('--check', action='store_true', help='Validate inputs without copying files')
    options = parser.parse_args()
    target = options.target or subprocess.check_output(['rustc', '--print', 'host-tuple'], text=True).strip()
    if not target or any(character not in 'abcdefghijklmnopqrstuvwxyz0123456789_-' for character in target):
        parser.error('Invalid Rust target triple')
    binary_dir = options.binary_dir or ROOT / 'target' / options.profile
    suffix = '.exe' if 'windows' in target else ''
    web = ROOT / 'web' / 'dist'
    if not (web / 'index.html').is_file():
        parser.error('Build web/dist first: cd web && npm ci && npm run build')
    sources = [(name, binary_dir / (name + suffix)) for name in BINARIES]
    missing = [str(path) for _, path in sources if not path.is_file()]
    if missing:
        parser.error('Build the native workspace binaries first: ' + ', '.join(missing))
    manifest = {'target': target, 'profile': options.profile, 'native_execution_verified': False, 'files': []}
    for name, source in sources:
        with source.open('rb') as stream:
            digest = hashlib.file_digest(stream, 'sha256').hexdigest()
        destination = DESKTOP / 'src-tauri' / 'binaries' / f'{name}-{target}{suffix}'
        manifest['files'].append({'name': name, 'sha256': digest, 'size': source.stat().st_size})
        if not options.check:
            destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(source, destination)
    if not options.check:
        target_web = DESKTOP / 'src-tauri' / 'resources' / 'web'
        # These are reproducible staging assets, never the user's data directory.
        if target_web.exists():
            shutil.rmtree(target_web)
        shutil.copytree(web, target_web)
        (DESKTOP / 'src-tauri' / 'resources' / 'sidecars.json').write_text(json.dumps(manifest, indent=2) + '\n')
    print(json.dumps(manifest, indent=2))


if __name__ == '__main__':
    main()
