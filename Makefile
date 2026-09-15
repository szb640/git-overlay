VERSION := $(shell cargo metadata --no-deps --format-version 1 | jq -r '.packages[0].version')

all: \
	out/git-overlay-$(VERSION)-x86_64-windows.zip \
	out/git-overlay-$(VERSION)-x86_64-linux.zip \
	out/git-overlay-$(VERSION)-aarch64-linux.zip \
	out/git-overlay-$(VERSION)-x86_64-linux.deb \
	out/git-overlay-$(VERSION)-aarch64-linux.deb

RUST_SOURCES := $(shell find src -type f -name '*.rs')
CARGO_FILES := Cargo.toml Cargo.lock .cargo/config.toml
FLAKE_FILES := flake.lock

TARGET_x86_64-windows := x86_64-pc-windows-gnu
TARGET_x86_64-linux := x86_64-unknown-linux-musl
TARGET_aarch64-linux := aarch64-unknown-linux-musl

DEBIAN_ARCH_x86_64-linux := amd64
DEBIAN_ARCH_aarch64-linux := arm64

target/%/release: $(RUST_SOURCES) $(CARGO_FILES) $(FLAKE_FILES)
	nix develop .#crossCompile --command bash -c 'cargo build --release --target $*'

.SECONDEXPANSION:

out/git-overlay-$(VERSION)-%.zip: target/$$(TARGET_$$*)/release
	mkdir -p out/
	rm -f "$(abspath $@)"
	zip -j "$@" "$(firstword $(wildcard $</git-overlay $</git-overlay.exe))"

# Build a Debian package from the compiled binary. Only valid for Linux
# targets; the prerequisite expands to nothing for non-Linux targets.
out/git-overlay-$(VERSION)-%.deb: target/$$(TARGET_$$*)/release
	mkdir -p build/staging/$*/DEBIAN build/staging/$*/usr/bin $(dir $@)
	cp "$</git-overlay" build/staging/$*/usr/bin/git-overlay
	chmod 0755 build/staging/$*/usr/bin/git-overlay
	@printf 'Package: git-overlay\nVersion: $(VERSION)\nSection: utils\nPriority: optional\nArchitecture: $(DEBIAN_ARCH_$*)\nMaintainer: Bence Szikszai <szb640@gmail.com>\nDescription: git-overlay\n Software for overlaying personal files onto a git repository\n' > build/staging/$*/DEBIAN/control
	nix develop .#crossCompile --command dpkg-deb --build --root-owner-group build/staging/$* "$(abspath $@)"
	rm -rf build/staging/$*

clean:
	rm -rf out/ build/ target/
