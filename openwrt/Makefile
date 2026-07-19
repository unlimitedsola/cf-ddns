include $(TOPDIR)/rules.mk

PKG_NAME:=cf-ddns
PKG_VERSION:=0.5.0
PKG_RELEASE:=1

PKG_SOURCE:=$(PKG_NAME)-$(PKG_VERSION).tar.gz
PKG_SOURCE_URL:=https://codeload.github.com/unlimitedsola/cf-ddns/tar.gz/v$(PKG_VERSION)?
PKG_HASH:=skip

PKG_LICENSE:=AGPL-3.0-or-later
PKG_LICENSE_FILES:=LICENSE

PKG_BUILD_DEPENDS:=rust/host
RUST_PKG_LOCKED:=1
RUST_PKG_FEATURES:=service

include $(INCLUDE_DIR)/package.mk
include $(TOPDIR)/feeds/packages/lang/rust/rust-package.mk

define Package/cf-ddns
  SECTION:=net
  CATEGORY:=Network
  TITLE:=Cloudflare Dynamic DNS Updater
  URL:=https://github.com/unlimitedsola/cf-ddns
  DEPENDS:=$(RUST_ARCH_DEPENDS)
endef

define Package/cf-ddns/description
  A lightweight, resource-efficient Cloudflare Dynamic DNS updater written in Rust,
  designed to run as a daemon under procd.
endef

define Package/cf-ddns/install
	$(INSTALL_DIR) $(1)/usr/libexec/cf-ddns
	$(INSTALL_BIN) $(PKG_INSTALL_DIR)/bin/cf-ddns $(1)/usr/libexec/cf-ddns/cf-ddns

	$(INSTALL_DIR) $(1)/usr/bin
	ln -sf /usr/libexec/cf-ddns/cf-ddns $(1)/usr/bin/cf-ddns

	$(INSTALL_DIR) $(1)/etc/cf-ddns
	$(INSTALL_CONF) $(PKG_BUILD_DIR)/config.example.toml $(1)/etc/cf-ddns/config.toml

	$(INSTALL_DIR) $(1)/etc/init.d
	$(INSTALL_BIN) ./files/cf-ddns.init $(1)/etc/init.d/cf-ddns
endef

$(eval $(call RustBinPackage,cf-ddns))
$(eval $(call BuildPackage,cf-ddns))
