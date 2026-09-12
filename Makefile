ARCHITECTURES := $(shell nix flake show --json 2>/dev/null | jq -r '.packages | keys[]')

all: out/git-overlay-x86_64-windows.zip \
	out/git-overlay-x86_64-linux.zip \
	out/git-overlay-armv7l-linux.zip \
	out/git-overlay-aarch64-linux.zip \
	out/aarch64-linux-deb \
	out/armv7l-linux-deb \
	out/x86_64-linux-deb

RUST_SOURCES := $(shell find src -type f -name '*.rs')
CARGO_FILES := Cargo.toml Cargo.lock
FLAKE_FILES := flake.lock

define NIX_PROJECT_template
NIX_PROJECT_ZIP_$(1) := $(abspath out/git-overlay-$(1).zip)

.PHONY: $(1)

$(1): out/$(1) out/git-overlay-$(1).zip out/$(1)-deb

out/$(1): $(RUST_SOURCES) $(CARGO_FILES) $(FLAKE_FILES)
	nix build .#packages.$(1).git-overlay --out-link "out/$(1)"

out/$(1)-deb: $(RUST_SOURCES) $(CARGO_FILES) $(FLAKE_FILES) out/$(1)
	nix build .#packages.$(1).git-overlay-debian --out-link "out/$(1)-deb"

out/git-overlay-$(1).zip: out/$(1)
	rm -f "$$(NIX_PROJECT_ZIP_$(1))"
	cd "out/$(1)" && zip -r "$$(NIX_PROJECT_ZIP_$(1))" .

endef

$(foreach p,$(ARCHITECTURES),$(eval $(call NIX_PROJECT_template,$(p))))

clean:
	rm -rf out/
