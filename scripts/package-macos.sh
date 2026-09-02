#!/usr/bin/env bash
# macOS 打包脚本: 编译通用二进制 (arm64 + x86_64) -> 组装 .app -> 临时签名 -> 生成 .dmg
#
# 仅依赖 macOS 自带工具 (lipo / codesign / hdiutil / plutil), 无需 cargo-bundle。
#
# 用法:
#   scripts/package-macos.sh                # 通用二进制
#   ARCHS="aarch64-apple-darwin" scripts/package-macos.sh   # 仅 Apple Silicon
#   SIGN_IDENTITY="Developer ID Application: ..." scripts/package-macos.sh   # 使用正式证书签名
#
# 产物位于 dist/:
#   dist/DevToolkit.app
#   dist/DevToolkit-<version>-macos.dmg
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

# ---- 元数据 ----------------------------------------------------------------
PRODUCT_NAME="DevToolkit"          # .app 文件名 (使用 ASCII, 避免路径问题)
DISPLAY_NAME="开发工具包"           # Finder / Dock 显示名
EXECUTABLE="devToolkit"            # Cargo 产出的二进制名 ([package].name)
BUNDLE_ID="com.khan.devToolkit"
MIN_SYSTEM_VERSION="10.13"
VERSION="$(grep -m1 '^version' Cargo.toml | sed -E 's/version *= *"([^"]+)"/\1/')"
ARCHS="${ARCHS:-aarch64-apple-darwin x86_64-apple-darwin}"
SIGN_IDENTITY="${SIGN_IDENTITY:--}"   # "-" 为 ad-hoc 签名 (Apple Silicon 必须至少有此签名)

DIST="$ROOT/dist"
APP="$DIST/$PRODUCT_NAME.app"
DMG="$DIST/$PRODUCT_NAME-$VERSION-macos.dmg"

echo "==> 版本 $VERSION, 目标: $ARCHS"

# ---- 1. 编译 ----------------------------------------------------------------
BINARIES=()
for target in $ARCHS; do
    echo "==> cargo build --release --target $target"
    cargo build --release --target "$target"
    BINARIES+=("target/$target/release/$EXECUTABLE")
done

# ---- 2. 组装 .app -----------------------------------------------------------
rm -rf "$APP"
mkdir -p "$APP/Contents/MacOS" "$APP/Contents/Resources"

if [ "${#BINARIES[@]}" -gt 1 ]; then
    echo "==> lipo 合并为通用二进制"
    lipo -create "${BINARIES[@]}" -output "$APP/Contents/MacOS/$EXECUTABLE"
else
    cp "${BINARIES[0]}" "$APP/Contents/MacOS/$EXECUTABLE"
fi
chmod +x "$APP/Contents/MacOS/$EXECUTABLE"
lipo -info "$APP/Contents/MacOS/$EXECUTABLE"

cp assets/icon.icns "$APP/Contents/Resources/app.icns"

sed -e "s|\${PRODUCT_NAME}|$PRODUCT_NAME|g" \
    -e "s|\${DISPLAY_NAME}|$DISPLAY_NAME|g" \
    -e "s|\${EXECUTABLE}|$EXECUTABLE|g" \
    -e "s|\${BUNDLE_ID}|$BUNDLE_ID|g" \
    -e "s|\${CARGO_PKG_VERSION}|$VERSION|g" \
    -e "s|\${MIN_SYSTEM_VERSION}|$MIN_SYSTEM_VERSION|g" \
    Info.plist > "$APP/Contents/Info.plist"
plutil -lint "$APP/Contents/Info.plist"
printf 'APPL????' > "$APP/Contents/PkgInfo"

# ---- 3. 签名 ----------------------------------------------------------------
echo "==> codesign (identity: $SIGN_IDENTITY)"
if [ "$SIGN_IDENTITY" = "-" ]; then
    codesign --force --deep --sign - "$APP"
else
    # 正式证书: 启用 hardened runtime 以便后续公证
    codesign --force --deep --options runtime --timestamp --sign "$SIGN_IDENTITY" "$APP"
fi
codesign --verify --deep --strict "$APP"

# ---- 4. 生成 .dmg -----------------------------------------------------------
echo "==> hdiutil 生成 $DMG"
STAGING="$(mktemp -d)"
trap 'rm -rf "$STAGING"' EXIT
cp -R "$APP" "$STAGING/"
ln -s /Applications "$STAGING/Applications"
rm -f "$DMG"
hdiutil create -quiet -volname "$DISPLAY_NAME $VERSION" -srcfolder "$STAGING" -ov -format UDZO "$DMG"

echo
echo "完成:"
echo "  $APP"
echo "  $DMG ($(du -h "$DMG" | cut -f1))"
