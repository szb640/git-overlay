PACKAGES := linux-x64 windows-x64 debian-x64

linux-x64_PKG := git-overlay
windows-x64_PKG := git-overlay-windows-x64
debian-x64_PKG := git-overlay-debian-x64

RUST_SOURCES := $(shell find src -type f -name '*.rs')
CARGO_FILES := Cargo.toml Cargo.lock
FLAKE_FILES := flake.nix flake.lock

define NIX_PROJECT_template
NIX_PROJECT_ZIP_$(1) := $(abspath out/git-overlay-$(1).zip)

.PHONY: $(1)

$(1): out/$(1) out/git-overlay-$(1).zip

out/$(1): $(RUST_SOURCES) $(CARGO_FILES) $(FLAKE_FILES)
	nix build .#$$($(1)_PKG) --out-link "out/$(1)"

out/git-overlay-$(1).zip: out/$(1)
	rm -f "$$(NIX_PROJECT_ZIP_$(1))"
	cd "out/$(1)" && zip -r "$$(NIX_PROJECT_ZIP_$(1))" .

endef

$(foreach p,$(PACKAGES),$(eval $(call NIX_PROJECT_template,$(p))))

all: $(PACKAGES)

release: out/git-overlay-windows-x64.zip \
	out/git-overlay-linux-x64.zip \
	out/debian-x64

clean:
	rm -rf out/
