#!/usr/bin/env -S just --justfile

project_root := justfile_directory()

[private]
default:
    @just --list

alias b := build
build:
    nix build .#git-overlay

alias w := windows
windows:
    nix build .#git-overlay-windows

alias t := test
test:
    cargo test

perf:
    cargo test --release --test scale sync_scales -- --ignored --nocapture --test-threads=1
