#!/usr/bin/env python3
"""Bundle the dependencies of an already verified macOS app without recompiling it."""

import argparse
import hashlib
import json
import plistlib
import shutil
import struct
import subprocess
from pathlib import Path


def run(*args):
    return subprocess.check_output([str(arg) for arg in args], text=True).strip()


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def text_digest(path):
    """Hash executable instructions independently of load commands and signing."""
    data = path.read_bytes()
    if struct.unpack_from('<I', data)[0] != 0xFEEDFACF:
        raise ValueError(f'Expected a thin 64-bit Mach-O executable: {path}')
    offset = 32
    for _ in range(struct.unpack_from('<I', data, 16)[0]):
        command, size = struct.unpack_from('<II', data, offset)
        if command == 0x19:  # LC_SEGMENT_64
            count = struct.unpack_from('<I', data, offset + 64)[0]
            for index in range(count):
                section = offset + 72 + index * 80
                name = data[section:section + 16].split(b'\0')[0]
                segment = data[section + 16:section + 32].split(b'\0')[0]
                if (segment, name) == (b'__TEXT', b'__text'):
                    length, start = struct.unpack_from('<QI', data, section + 40)
                    return hashlib.sha256(data[start:start + length]).hexdigest()
        offset += size
    raise ValueError(f'Missing executable __text section: {path}')


def dependencies(path):
    return [line.strip().split(' (compatibility version', 1)[0]
            for line in run('otool', '-L', path).splitlines()[1:]]


def rpaths(path):
    lines = run('otool', '-l', path).splitlines()
    return [lines[index + 2].strip()[5:].split(' (offset ', 1)[0]
            for index, line in enumerate(lines) if line.strip() == 'cmd LC_RPATH']


def system_library(name):
    return name.startswith(('/System/Library/', '/usr/lib/'))


def resolve_dependency(name, image, executable):
    def expand(value):
        return value.replace('@loader_path', str(image.parent)).replace(
            '@executable_path', str(executable.parent))
    if name.startswith('@rpath/'):
        candidates = [Path(expand(base)) / name[7:]
                      for base in rpaths(image) + rpaths(executable)]
    else:
        candidates = [Path(expand(name))]
    for candidate in candidates:
        if candidate.is_file():
            return candidate.resolve()
    raise FileNotFoundError(f'Cannot resolve {name} referenced by {image}')


def collect_notices(source, resources, formulas):
    parts = source.parts
    if 'Cellar' not in parts:
        return
    index = parts.index('Cellar')
    root = Path(*parts[:index + 3])
    formula, version = parts[index + 1:index + 3]
    key = f'{formula}-{version}'
    if key in formulas:
        return
    destination = resources / 'ThirdPartyNotices' / key
    copied = []
    for path in root.rglob('*'):
        if not path.is_file():
            continue
        if (path.name.upper().startswith(('LICENSE', 'COPYING', 'NOTICE', 'AUTHORS'))
                or (path.parent.name == '.brew' and path.suffix == '.rb')):
            relative = path.relative_to(root)
            target = destination / relative
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(path, target)
            copied.append(str(relative))
    formulas[key] = {'formula': formula, 'version': version, 'notices': copied}


def verify(app):
    executable = app / 'Contents/MacOS/Wallpaper Engine'
    frameworks = app / 'Contents/Frameworks'
    images = [executable] + sorted(frameworks.glob('*.dylib'))
    for image in images:
        for dependency in dependencies(image):
            if system_library(dependency):
                continue
            prefix = ('@executable_path/../Frameworks/' if image == executable
                      else '@loader_path/')
            if not dependency.startswith(prefix):
                raise ValueError(f'External dependency: {image}: {dependency}')
            if not (frameworks / dependency[len(prefix):]).is_file():
                raise ValueError(f'Missing bundled dependency: {dependency}')
        if rpaths(image):
            raise ValueError(f'Unexpected rpaths in {image}: {rpaths(image)}')
        if 'arm64' not in run('lipo', '-archs', image).split():
            raise ValueError(f'Missing arm64 architecture: {image}')
    icd = json.loads((app / 'Contents/Resources/MoltenVK_icd.json').read_text())
    if icd['ICD']['library_path'] != '../Frameworks/libMoltenVK.dylib':
        raise ValueError('MoltenVK ICD must use the bundled relative path')
    run('codesign', '--verify', '--deep', '--strict', '--verbose=2', app)
    print(f'Verified {len(images)} Mach-O images; all non-system links are bundled.')


def package(source_app, app, moltenvk):
    if app.exists():
        raise FileExistsError(f'Refusing to replace an existing bundle: {app}')
    source_executable = source_app / 'Contents/MacOS/Wallpaper Engine'
    original_hash = digest(source_executable)
    original_text = text_digest(source_executable)
    run('ditto', source_app, app)
    executable = app / 'Contents/MacOS/Wallpaper Engine'
    frameworks = app / 'Contents/Frameworks'
    resources = app / 'Contents/Resources'
    frameworks.mkdir(exist_ok=True)
    queue = [(source_executable, executable)]
    names, formulas, records = {}, {}, []

    def add_library(source):
        source = source.resolve()
        name = source.name
        if name in names:
            if names[name] != source and digest(names[name]) != digest(source):
                raise ValueError(f'Conflicting dylib name: {name}')
            return frameworks / name
        names[name] = source
        target = frameworks / name
        shutil.copy2(source, target)
        target.chmod(target.stat().st_mode | 0o200)
        queue.append((source, target))
        collect_notices(source, resources, formulas)
        records.append({'file': name, 'input_sha256': digest(source)})
        return target

    add_library(moltenvk)
    for source, target in queue:
        changes = []
        for dependency in dependencies(source):
            if system_library(dependency):
                continue
            resolved = resolve_dependency(dependency, source, source_executable)
            bundled = add_library(resolved)
            prefix = '@executable_path/../Frameworks/' if target == executable else '@loader_path/'
            replacement = prefix + bundled.name
            changes.extend(['-change', dependency, replacement])
        for old in dict.fromkeys(rpaths(target)):
            run('install_name_tool', '-delete_rpath', old, target)
        args = ['install_name_tool']
        if target != executable:
            args.extend(['-id', '@loader_path/' + target.name])
        if len(args) > 1 or changes:
            run(*args, *changes, target)

    icd_path = resources / 'MoltenVK_icd.json'
    icd = json.loads(icd_path.read_text())
    icd['ICD']['library_path'] = '../Frameworks/libMoltenVK.dylib'
    icd_path.write_text(json.dumps(icd, indent=2) + '\n')
    license_path = Path(__file__).resolve().parent.parent / 'LICENSE'
    if license_path.is_file():
        shutil.copy2(license_path, resources / 'LICENSE')
    metadata = plistlib.loads((app / 'Contents/Info.plist').read_bytes())
    report = {'version': metadata['CFBundleShortVersionString'],
              'build': metadata['CFBundleVersion'],
              'input_executable_sha256': original_hash,
              'executable_text_sha256': original_text,
              'libraries': records, 'formulas': formulas}
    (resources / 'BundleManifest.json').write_text(json.dumps(report, indent=2) + '\n')
    for image in sorted(frameworks.glob('*.dylib')):
        run('codesign', '--force', '--sign', '-', image)
    run('codesign', '--force', '--sign', '-', app)
    if text_digest(executable) != original_text:
        raise ValueError('Packaging changed executable instructions')
    verify(app)
    print(f'App version {report["version"]} ({report["build"]}); '
          f'executable instructions unchanged; {len(names)} dylibs bundled.')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--source-app', type=Path)
    parser.add_argument('--output-app', type=Path, required=True)
    parser.add_argument('--moltenvk', type=Path)
    parser.add_argument('--verify-only', action='store_true')
    args = parser.parse_args()
    if args.verify_only:
        verify(args.output_app.resolve())
    elif args.source_app and args.moltenvk:
        package(args.source_app.resolve(), args.output_app.resolve(), args.moltenvk.resolve())
    else:
        parser.error('--source-app and --moltenvk are required when packaging')
