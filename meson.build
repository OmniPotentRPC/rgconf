# Copyright 2023--present rgconf developers

project(
    'rgconf',
    'c',
    version: '0.1.0',
    meson_version: '>=1.3.0',
    default_options: ['warning_level=3'],
    license: 'MIT',
)

fs = import('fs')
cargo_prog = find_program('cargo', required: true)
py = find_program('python3', 'python')
cc = meson.get_compiler('c')

so_name = host_machine.system() == 'darwin' ? 'librgconf_c_abi.dylib' : 'librgconf_c_abi.so'
profile = get_option('buildtype') == 'debug' ? 'debug' : 'release'

rgconf_cargo = custom_target(
    'rgconf-cargo',
    input: files('Cargo.toml', 'crates/rgconf-c-abi/src/lib.rs', 'include/rgconf.h'),
    output: [so_name, 'librgconf_c_abi.a'],
    command: [
        py,
        files('scripts/meson_cargo_build.py'),
        cargo_prog.full_path(),
        meson.project_source_root(),
        meson.current_build_dir() / 'cargo-target',
        profile,
        so_name,
        'librgconf_c_abi.a',
        '@OUTPUT0@',
        '@OUTPUT1@',
    ],
    install: false,
    build_by_default: true,
)

install_headers('include/rgconf.h')
