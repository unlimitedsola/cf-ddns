# Include the standard OpenWrt build rules. This defines the core functions and paths used in the buildroot.
include $(TOPDIR)/rules.mk

# Define package metadata variables:
# - PKG_NAME: The name of the package.
# - PKG_VERSION: The version of the package matching the upstream release.
# - PKG_RELEASE: The build release number, incremented for packaging-only changes.
PKG_NAME:=cf-ddns
PKG_VERSION:=0.5.0
PKG_RELEASE:=1

# Configure source downloading settings:
# - PKG_SOURCE: The filename of the downloaded tarball.
# - PKG_SOURCE_URL: The remote repository URL to fetch the source tarball.
# - PKG_HASH: The checksum of the downloaded file. Set to "skip" because this package
#   is built using local source tree overrides (CONFIG_SRC_TREE_OVERRIDE=y) in development.
PKG_SOURCE:=$(PKG_NAME)-$(PKG_VERSION).tar.gz
PKG_SOURCE_URL:=https://codeload.github.com/unlimitedsola/cf-ddns/tar.gz/v$(PKG_VERSION)?
PKG_HASH:=skip

# Licensing metadata for package auditing.
PKG_LICENSE:=AGPL-3.0-or-later
PKG_LICENSE_FILES:=LICENSE

# Rust compiler and dependencies options:
# - PKG_BUILD_DEPENDS: Specifies that we need the host-system Rust compiler (rustc/cargo) to compile the package.
# - RUST_PKG_LOCKED: Runs cargo commands with the `--locked` flag to enforce strict lockfile consistency.
# - RUST_PKG_FEATURES: Compile features passed to cargo. Enables the "service" feature for service runner code.
PKG_BUILD_DEPENDS:=rust/host
RUST_PKG_LOCKED:=1
RUST_PKG_FEATURES:=service

# Include package helpers:
# - package.mk: The standard template defining generic packaging rules (e.g. Build/Prepare, Build/Compile defaults).
# - rust-package.mk: The specialized helper file from the OpenWrt packages feed that configures Rust compiler flags,
#   resolves targets, and manages cross-compiling crates.
include $(INCLUDE_DIR)/package.mk
include $(TOPDIR)/feeds/packages/lang/rust/rust-package.mk

# Define metadata shown in the OpenWrt menuconfig (configuration interface):
# - SECTION: The classification category under the packaging tree (net = Network).
# - CATEGORY: The menu header under which the package appears in menuconfig.
# - TITLE: A short, user-facing summary of the package's purpose.
# - URL: Upstream source repository or project homepage.
# - DEPENDS: Package dependencies. $(RUST_ARCH_DEPENDS) dynamically resolves toolchain requirements
#   (like architecture-specific targets or libraries) needed to execute Rust code on this CPU.
define Package/cf-ddns
  SECTION:=net
  CATEGORY:=Network
  TITLE:=Cloudflare Dynamic DNS Updater
  URL:=https://github.com/unlimitedsola/cf-ddns
  DEPENDS:=$(RUST_ARCH_DEPENDS)
endef

# User-facing description of the package shown in menuconfig and package info.
define Package/cf-ddns/description
  A lightweight, resource-efficient Cloudflare Dynamic DNS updater written in Rust,
  designed to run as a daemon under procd.
endef

# Declare configuration preservation (conffiles):
# Any file listed here will be backed up by the OpenWrt sysupgrade system. When the user updates
# their firmware or packages, their custom configuration file won't be replaced/wiped by the default.
define Package/cf-ddns/conffiles
/etc/cf-ddns/config.toml
endef

# Define installation commands:
# This step maps files from the build staging directory to the layout of the target filesystem on the router.
# - $(1) represents the target root filesystem directory (similar to / or staging directory).
# - $(INSTALL_DIR) helper function creates directories with correct ownership and permissions (usually 0755).
# - $(INSTALL_BIN) copies executable files with executable permissions (0755).
# - $(INSTALL_CONF) copies configuration files with read/write permissions (0600 or 0644).
define Package/cf-ddns/install
	# Install the compiled binary directly into /usr/bin/
	$(INSTALL_DIR) $(1)/usr/bin
	$(INSTALL_BIN) $(PKG_INSTALL_DIR)/bin/cf-ddns $(1)/usr/bin/cf-ddns

	# Staging default config: Copies the example configuration as the starting config.toml file
	$(INSTALL_DIR) $(1)/etc/cf-ddns
	$(INSTALL_CONF) $(PKG_BUILD_DIR)/config.example.toml $(1)/etc/cf-ddns/config.toml

	# Install the service init script to /etc/init.d/
	$(INSTALL_DIR) $(1)/etc/init.d
	$(INSTALL_BIN) ./files/cf-ddns.init $(1)/etc/init.d/cf-ddns
endef

# Execute the macros to build the binary and register the packaging tasks:
# - RustBinPackage: Macro provided by rust-package.mk to automate Rust crate compilation.
# - BuildPackage: Macro that registers all targets for compiling and creating the final package archive (.apk/.ipk).
$(eval $(call RustBinPackage,cf-ddns))
$(eval $(call BuildPackage,cf-ddns))
