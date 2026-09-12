ARCHITECTURES := $(shell nix flake show --json 2>/dev/null | jq -r '.packages | keys[]')
VERSION := $(shell cargo metadata --no-deps --format-version 1 | jq -r '.packages[0].version')

all: \
	out/git-overlay-$(VERSION)-x86_64-windows.zip \
	out/git-overlay-$(VERSION)-x86_64-linux.zip \
	out/git-overlay-$(VERSION)-armv7l-linux.zip \
	out/git-overlay-$(VERSION)-aarch64-linux.zip \
	out/git-overlay-$(VERSION)-aarch64-linux.deb \
	out/git-overlay-$(VERSION)-armv7l-linux.deb \
	out/git-overlay-$(VERSION)-x86_64-linux.deb

RUST_SOURCES := $(shell find src -type f -name '*.rs')
CARGO_FILES := Cargo.toml Cargo.lock
FLAKE_FILES := flake.lock

define NIX_PROJECT_template

.PHONY: $(1)

# Compile binary with flake
build/$(1): $(RUST_SOURCES) $(CARGO_FILES) $(FLAKE_FILES)
	nix build .#packages.$(1).git-overlay --out-link "build/$(1)"

# Compile debian package with flake
build/$(1)-deb/git-overlay-$(VERSION)-$(1).deb: $(RUST_SOURCES) $(CARGO_FILES) $(FLAKE_FILES) build/$(1)
	nix build .#packages.$(1).git-overlay-debian --out-link "build/$(1)-deb"

# Package debian package
out/git-overlay-$(VERSION)-$(1).deb: build/$(1)-deb/git-overlay-$(VERSION)-$(1).deb
	mkdir -p out
	cp -f "$$<" "$$@"

# Package binaries
out/git-overlay-$(VERSION)-$(1).zip: build/$(1)
	rm -f "$$(abspath $$@)"
	mkdir -p out
	cd "build/$(1)" && zip -r "$$(abspath $$@)" .

endef

$(foreach p,$(ARCHITECTURES),$(eval $(call NIX_PROJECT_template,$(p))))

clean:
	rm -rf out/
