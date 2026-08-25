#!/usr/bin/env -S just --justfile

project_root := justfile_directory()

[private]
default:
    @just --list

alias b := build
build:
    nix build .#git-overlay

alias t := test
test:
    cargo test
