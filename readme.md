
# 开发工具包

带 UI 界面的开发工具集合, 纯 Rust + egui 实现, 跨平台 (Windows / Linux / macOS), 不依赖 C/C++ 代码。

## 功能

1. 时间戳与时间格式化工具
   - 时间戳(秒 / 毫秒) ↔ 时间字符串 双向转换, 自动识别单位
   - 当前时间实时刷新(每秒定时更新)
2. 字符集编码转换工具 (UTF-8 / GBK / UTF-16)
   - hex 字符串 解码为指定字符集
   - 指定字符集 编码为 hex 字符串
   - hex 解码输入支持格式: 连续 hex / 空白分隔 / `0x` 前缀 (可多处重复) / `\xHH` 字节转义
   - Unicode 码点表达式 `\uXXXX` (定长4位)、`\u{...}`、`U+XXXX` 仅限 UTF-16 字符集下生效
3. URL 编码与解码工具 (标准表单 / RFC 3986 安全模式)
4. Base64 编码与解码工具 (标准字符表 / URL 安全字符表)
5. 哈希计算工具 (MD5 / SHA-1 / SHA-224 / SHA-256 / SHA-384 / SHA-512 / BLAKE2b-512 / BLAKE2s-256 / BLAKE3)
6. 校验计算工具 (CRC-8 / CRC-16 IBM / CRC-16 MODBUS / CRC-32 / CRC-32C / CRC-64 / BCC / LRC / SUM-8 / HMAC-MD5 / HMAC-SHA1 / HMAC-SHA256 / HMAC-SHA512)
   - HMAC 算法需输入密钥 (Key), 其余算法无需密钥

## 特性

- 深色主题界面, 输入框支持复制粘贴, 结果一键复制
- 国际化: 简体中文 / 繁体中文 / English / 日本語
- 语言文案存放于可执行文件同目录 `langs/*.json`, 无需改代码即可自行翻译或新增语言
- Windows 二进制自动嵌入应用图标 (amd64 / arm64)

## 编译

### 环境要求

- Rust 工具链
- [zig](https://ziglang.org/) + [cargo-zigbuild](https://github.com/rust-cross/cargo-zigbuild) (交叉编译用)
- 安装交叉编译目标 (以 aarch64 Windows 为例):

```bash
  $env:RUSTUP_DIST_SERVER="https://rsproxy.cn"; $env:RUSTUP_UPDATE_ROOT="https://rsproxy.cn/rustup"
  rustup target add aarch64-pc-windows-gnullvm # 安装aarch64架构的Windows目标
```

### 命令

```bash
  cargo run --release                           # 本机运行
  cargo build --release                         # 本机发布构建

  # 交叉编译命令, 依赖 cargo-zigbuild插件和 zig 编译器 以及安装的交叉编译目标
  cargo zigbuild --release --target x86_64-pc-windows-gnu       # Windows x86_64 架构
  cargo zigbuild --release --target aarch64-pc-windows-gnullvm  # Windows aarch64 架构
  cargo zigbuild --release --target x86_64-unknown-linux-musl   # Linux x86_64 架构
  cargo zigbuild --release --target aarch64-unknown-linux-musl  # Linux aarch64 架构
  cargo zigbuild --release --target x86_64-apple-darwin         # macOS Intel 架构
  cargo zigbuild --release --target aarch64-apple-darwin        # macOS Apple Silicon 架构
```

### 语言文件

程序首次运行会在可执行文件同目录生成 `langs/` 并导出内置语言文件 (如 `zh-CN.json`),
编辑 JSON 即可修改文案, 新增 JSON 文件 (如 `de.json`) 会作为新语言出现在界面中。
