//! 国际化支持
//!
//! 语言定义存储在 `langs/*.json` 文件中, 用户无需修改代码即可自行翻译。
//! 程序首次运行时自动创建 `langs/` 并导出内置语言文件作为翻译模板。
//! 目录位置: 可执行文件同目录; macOS `.app` 运行时为 `~/Library/Application Support/devToolkit/langs`。
//!
//! 机制:
//! - 内置 5 种语言(简体中文/繁体中文/英文/日文/韩文)作为默认与兜底
//! - `langs/` 目录下的 JSON 文件(如 `zh-CN.json`)会覆盖同名内置语言
//! - 新增文件(如 `de.json`)会作为新语言出现在界面中
//! - 语言文件中缺失的字段自动回退到内置中文
//!
//! 语言文件格式示例 (`langs/en.json`):
//! ```json
//! {
//!   "name": "English",
//!   "texts": {
//!     "app_title": "Dev Toolkit",
//!     "tab_timestamp": "Timestamp",
//!     ...
//!   }
//! }
//! ```

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// 界面字符串集合(字段名即语言文件中的 key)
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Texts {
    pub app_title: String,
    pub tab_timestamp: String,
    pub tab_encoding: String,
    pub tab_url: String,
    pub tab_base64: String,
    pub btn_about: String,
    pub btn_close: String,

    // 关于界面
    pub about_title: String,
    pub about_version: String,
    pub about_author: String,
    pub about_contact: String,
    pub about_license: String,
    pub about_built: String,
    pub about_desc: String,

    // 时间戳工具
    pub ts_title: String,
    pub ts_section_now: String,
    pub ts_local_time: String,
    pub ts_utc_time: String,
    pub ts_millis: String,
    pub ts_seconds: String,
    pub ts_refresh: String,
    pub ts_fill_ms: String,
    pub ts_fill_s: String,
    pub ts_fill_now_time: String,
    pub ts_ts_to_time: String,
    pub ts_time_to_ts: String,
    pub ts_timestamp: String,
    pub ts_time_str: String,
    pub ts_unit: String,
    pub ts_unit_auto: String,
    pub ts_unit_s: String,
    pub ts_unit_ms: String,
    pub ts_copy: String,
    pub ts_hint_ts: String,
    pub ts_hint_dt: String,
    pub ts_fmt_hint: String,
    pub ts_err_not_int: String,
    pub ts_err_range: String,
    pub ts_err_empty: String,
    pub ts_err_parse: String,
    pub ts_err_ambiguous: String,

    // 字符集转换工具
    pub enc_title: String,
    pub enc_charset: String,
    pub enc_decode: String,
    pub enc_encode: String,
    pub enc_hex: String,
    pub enc_text: String,
    pub enc_copy: String,
    pub enc_fill_text: String,
    pub enc_hint_hex: String,
    pub enc_hint_text: String,
    pub enc_err_odd: String,
    pub enc_err_nonhex: String,
    /// Unicode 码点转义(\u/U+)仅支持 UTF-16 字符集
    pub enc_err_utf16_escape: String,
    pub enc_err_utf16_odd: String,
    pub enc_err_utf16_pair: String,
    pub enc_warn_replace: String,
    pub enc_warn_gbk: String,
    /// Big5 编码时存在无法表示的字符
    pub enc_warn_big5: String,
    /// hex 解码时的编码猜测提示
    pub enc_guess: String,
    /// 除最佳猜测外的其他可能编码
    pub enc_guess_alt: String,

    // 通用编解码文案(URL / Base64 工具共用)
    pub gen_encode: String,
    pub gen_decode: String,
    pub gen_input: String,
    pub gen_output: String,
    pub gen_copy: String,

    // URL 编解码工具
    pub url_title: String,
    pub url_mode: String,
    pub url_mode_std: String,
    pub url_mode_safe: String,
    pub url_hint: String,
    pub url_note: String,
    pub url_warn_invalid: String,

    // Base64 编解码工具
    pub b64_title: String,
    pub b64_alphabet: String,
    pub b64_std: String,
    pub b64_urlsafe: String,
    pub b64_hint: String,
    pub b64_err_invalid: String,

    // 哈希计算工具
    pub tab_hash: String,
    pub hash_title: String,
    pub hash_algorithm: String,
    pub hash_hint: String,

    // 校验算法工具(CRC / BCC / LRC / HMAC)
    pub tab_checksum: String,
    pub checksum_title: String,
    pub checksum_algorithm: String,
    pub checksum_hint: String,
    pub checksum_key: String,

    // HTTP 请求工具(类 Postman)
    pub tab_http: String,
    pub http_title: String,
    pub http_url_hint: String,
    pub http_send: String,
    pub http_sending: String,
    pub http_headers: String,
    pub http_header_key: String,
    pub http_header_val: String,
    pub http_add: String,
    pub http_remove: String,
    pub http_body: String,
    pub http_body_none: String,
    pub http_body_raw: String,
    pub http_body_form: String,
    pub http_body_multipart: String,
    pub http_raw_text: String,
    pub http_multipart_hint: String,
    pub http_ct_boundary: String,
    pub http_form_key: String,
    pub http_form_val: String,
    pub http_raw_hint: String,
    pub http_response: String,
    pub http_resp_empty: String,
    pub http_resp_headers: String,
    pub http_resp_body: String,
    pub http_history: String,
    pub http_clear: String,
    pub http_err_url: String,
    pub http_err_header: String,
    pub http_err_send: String,
    pub http_warn_truncated: String,
    pub http_warn_decode: String,
}

impl Default for Texts {
    fn default() -> Self {
        return zh_cn();
    }
}

/// 一种语言的定义
#[derive(Clone, Debug)]
pub struct LangDef {
    /// 语言标识(对应语言文件名, 如 zh-CN)
    pub code: String,
    /// 语言显示名
    pub native_name: String,
    pub texts: Texts,
}

/// 国际化管理器: 内置语言 + 外部语言文件的合并结果
pub struct I18n {
    pub langs: Vec<LangDef>,
    pub current: usize,
}

impl I18n {
    /// 加载全部语言: 内置 + `langs/` 目录下的语言文件
    pub fn load() -> Self {
        let mut langs = builtin_langs();
        if let Some(dir) = langs_dir() {
            // 首次运行创建 `langs/` 并导出内置语言文件作为翻译模板;
            // 后续运行也会补写缺失的内置文件(便于升级后获得新的语言模板)
            export_builtin_langs(&dir, &langs);
            load_lang_files(&dir, &mut langs);
        }
        return Self { langs, current: 0 };
    }

    /// 当前语言的文案
    pub fn texts(&self) -> &Texts {
        return &self.langs[self.current].texts;
    }
}

// ---------------------------------------------------------------------------
// 语言文件读写
// ---------------------------------------------------------------------------

/// 语言文件结构: `name` 为语言显示名, `texts` 为翻译表
#[derive(Serialize, Deserialize)]
struct LangFile {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    texts: Texts,
}

/// 语言文件目录
///
/// - 默认为可执行文件同目录下的 `langs/` (便携式布局)
/// - macOS 上以 `.app` 形式运行时, bundle 内部受代码签名保护、不可写入,
///   改用 `~/Library/Application Support/devToolkit/langs`
fn langs_dir() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let exe_dir = exe.parent()?;

    let in_app_bundle = cfg!(target_os = "macos")
        && exe_dir
            .to_str()
            .is_some_and(|p| p.contains(".app/Contents/MacOS"));
    if in_app_bundle {
        let home = std::env::var_os("HOME")?;
        return Some(
            PathBuf::from(home)
                .join("Library/Application Support/devToolkit/langs"),
        );
    }
    return Some(exe_dir.join("langs"));
}

/// 导出内置语言文件作为翻译模板
fn export_builtin_langs(dir: &Path, langs: &[LangDef]) {
    if fs::create_dir_all(dir).is_err() {
        eprintln!("[i18n] 无法创建语言目录: {}", dir.display());
        return;
    }
    for lang in langs {
        let file = LangFile {
            name: Some(lang.native_name.clone()),
            texts: lang.texts.clone(),
        };
        let Ok(json) = serde_json::to_string_pretty(&file) else {
            continue;
        };
        let path = dir.join(format!("{}.json", lang.code));
        if !path.exists() {
            if let Err(e) = fs::write(&path, json) {
                eprintln!("[i18n] 导出语言文件失败 {}: {e}", path.display());
            }
        }
    }
}

/// 加载 `langs/` 目录下的语言文件, 覆盖同名内置语言或追加新语言
fn load_lang_files(dir: &Path, langs: &mut Vec<LangDef>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let Some(code) = path.file_stem().and_then(|s| s.to_str()).map(String::from)
        else {
            continue;
        };
        let Ok(content) = fs::read_to_string(&path) else {
            eprintln!("[i18n] 读取语言文件失败: {}", path.display());
            continue;
        };

        // 先解析为 JSON, 将缺失字段用同名内置语言的翻译补齐
        // (升级新增文案后, 旧的语言文件无需手动更新即可获得正确显示)
        let mut value: serde_json::Value = match serde_json::from_str(&content) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("[i18n] 语言文件 {} 解析失败: {e}", path.display());
                continue;
            }
        };
        merge_builtin_fields(&mut value, &code, langs);

        let file = match serde_json::from_value::<LangFile>(value) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("[i18n] 语言文件 {} 解析失败: {e}", path.display());
                continue;
            }
        };

        let native_name = file.name.unwrap_or_else(|| code.clone());
        if let Some(existing) = langs.iter_mut().find(|l| l.code == code) {
            // 覆盖内置语言: 仅覆盖文案, 显示名固定使用内置英文名(避免乱码)
            existing.texts = file.texts;
        } else {
            // 追加新语言
            langs.push(LangDef {
                code,
                native_name,
                texts: file.texts,
            });
        }
    }
}

/// 语言文件中缺失的字段用同名内置语言的翻译补齐
///
/// `Texts` 的 serde 缺省兜底是中文, 若不合并, 旧版语言文件在非中文界面下
/// 会出现中英混杂(新版新增的字段显示中文)。
fn merge_builtin_fields(value: &mut serde_json::Value, code: &str, langs: &[LangDef]) {
    let Some(builtin) = langs.iter().find(|l| l.code == code) else {
        return;
    };
    let Ok(template) = serde_json::to_value(&builtin.texts) else {
        return;
    };
    match value.get_mut("texts").and_then(|t| t.as_object_mut()) {
        Some(texts) => {
            if let Some(obj) = template.as_object() {
                for (k, v) in obj {
                    texts.entry(k.clone()).or_insert(v.clone());
                }
            }
        }
        None => {
            // texts 字段缺失或类型错误: 整体使用内置翻译
            value["texts"] = template;
        }
    }
}

// ---------------------------------------------------------------------------
// 内置语言(兜底)
// ---------------------------------------------------------------------------

/// 内置语言列表
///
/// 显示名统一使用英文: 界面字体可能不包含对应语言字符集, 英文显示名可以避免乱码。
fn builtin_langs() -> Vec<LangDef> {
    return vec![
        LangDef { code: "zh-CN".into(), native_name: "Simplified Chinese".into(), texts: zh_cn() },
        LangDef { code: "zh-TW".into(), native_name: "Traditional Chinese".into(), texts: zh_tw() },
        LangDef { code: "en".into(), native_name: "English".into(), texts: en() },
        LangDef { code: "ja".into(), native_name: "Japanese".into(), texts: ja() },
    ];
}

fn zh_cn() -> Texts {
    return Texts {
        app_title: "开发工具包".into(),
        tab_timestamp: "时间戳工具".into(),
        tab_encoding: "字符集转换".into(),
        tab_url: "URL 编解码".into(),
        tab_base64: "Base64".into(),
        btn_about: "关于".into(),
        btn_close: "关闭".into(),
        about_title: "关于开发工具包".into(),
        about_version: "版本".into(),
        about_author: "作者".into(),
        about_contact: "联系方式".into(),
        about_license: "许可证".into(),
        about_built: "编译信息".into(),
        about_desc: "带 UI 界面的常用开发工具集合, 包含时间戳转换与字符集编码转换工具。".into(),
        ts_title: "时间戳工具".into(),
        ts_section_now: "当前时间".into(),
        ts_local_time: "本地时间".into(),
        ts_utc_time: "UTC 时间".into(),
        ts_millis: "毫秒时间戳".into(),
        ts_seconds: "秒时间戳".into(),
        ts_refresh: "刷新".into(),
        ts_fill_ms: "填入毫秒戳".into(),
        ts_fill_s: "填入秒戳".into(),
        ts_fill_now_time: "填入当前时间".into(),
        ts_ts_to_time: "时间戳 → 时间字符串".into(),
        ts_time_to_ts: "时间字符串 → 时间戳".into(),
        ts_timestamp: "时间戳".into(),
        ts_time_str: "时间字符串".into(),
        ts_unit: "单位".into(),
        ts_unit_auto: "自动识别".into(),
        ts_unit_s: "秒".into(),
        ts_unit_ms: "毫秒".into(),
        ts_copy: "复制".into(),
        ts_hint_ts: "输入时间戳, 如 1756600000000 或 1756600000".into(),
        ts_hint_dt: "如 2024-01-01 12:00:00".into(),
        ts_fmt_hint: "支持 YYYY-MM-DD HH:MM:SS(.fff)、YYYY/MM/DD、ISO8601(带时区) 等格式".into(),
        ts_err_not_int: "时间戳必须是整数".into(),
        ts_err_range: "时间戳超出可表示范围".into(),
        ts_err_empty: "请输入时间字符串".into(),
        ts_err_parse: "无法解析时间字符串".into(),
        ts_err_ambiguous: "本地时间解析存在歧义".into(),
        enc_title: "字符集转换工具".into(),
        enc_charset: "字符集".into(),
        enc_decode: "hex → 文本 (解码)".into(),
        enc_encode: "文本 → hex (编码)".into(),
        enc_hex: "hex".into(),
        enc_text: "文本".into(),
        enc_copy: "复制结果".into(),
        enc_fill_text: "使用解码结果".into(),
        enc_hint_hex: r"输入 hex: 支持空格、0x 前缀、\xHH; \uXXXX / U+XXXX 仅限 UTF-16".into(),
        enc_hint_text: "输入要编码的文本".into(),
        enc_err_odd: "hex 长度必须为偶数(两个字符表示一个字节)".into(),
        enc_err_nonhex: "hex 包含非十六进制字符(仅允许 0-9 A-F)".into(),
        enc_err_utf16_escape: r"Unicode 转义(\u / U+)仅支持 UTF-16 字符集".into(),
        enc_err_utf16_odd: "UTF-16 数据字节数必须为偶数".into(),
        enc_err_utf16_pair: "UTF-16 解码失败: 包含无效的代理项".into(),
        enc_warn_replace: "存在无法解码的字节, 已用 U+FFFD 替换".into(),
        enc_warn_gbk: "部分字符无法用 GBK 表示, 已用替代字符替换".into(),
        enc_warn_big5: "部分字符无法用 Big5 表示, 已用替代字符替换".into(),
        enc_guess: "猜测编码".into(),
        enc_guess_alt: "其他可能".into(),
        gen_encode: "编码".into(),
        gen_decode: "解码".into(),
        gen_input: "输入".into(),
        gen_output: "输出".into(),
        gen_copy: "复制结果".into(),
        url_title: "URL 编解码工具".into(),
        url_mode: "模式".into(),
        url_mode_std: "标准 (表单)".into(),
        url_mode_safe: "安全 (RFC 3986)".into(),
        url_hint: "输入要编码或解码的文本".into(),
        url_note: "标准模式: 空格→+ 保留 A-Za-z0-9-_.*; 安全模式: 空格→%20 保留 A-Za-z0-9-_.~!*'()".into(),
        url_warn_invalid: "存在无效的 % 转义序列, 已按字面保留".into(),
        b64_title: "Base64 编解码工具".into(),
        b64_alphabet: "字符表".into(),
        b64_std: "标准".into(),
        b64_urlsafe: "URL 安全".into(),
        b64_hint: "输入要编码或解码的 Base64 文本".into(),
        b64_err_invalid: "无效的 Base64 数据(非法字符或长度不正确)".into(),
        tab_hash: "哈希工具".into(),
        hash_title: "哈希计算工具".into(),
        hash_algorithm: "算法".into(),
        hash_hint: "输入要计算哈希的文本".into(),
        tab_checksum: "校验工具".into(),
        checksum_title: "校验计算工具".into(),
        checksum_algorithm: "算法".into(),
        checksum_hint: "输入要校验的数据".into(),
        checksum_key: "密钥 (Key)".into(),
        tab_http: "HTTP 请求".into(),
        http_title: "HTTP 请求工具".into(),
        http_url_hint: "https://httpbin.org/get".into(),
        http_send: "发送".into(),
        http_sending: "请求中…".into(),
        http_headers: "请求头".into(),
        http_header_key: "名称".into(),
        http_header_val: "值".into(),
        http_add: "添加".into(),
        http_remove: "移除".into(),
        http_body: "请求体".into(),
        http_body_none: "无".into(),
        http_body_raw: "原文".into(),
        http_body_form: "URL 编码".into(),
        http_body_multipart: "Form-Data".into(),
        http_raw_text: "文本".into(),
        http_multipart_hint: r"值以 @ 开头表示发送文件, 如 @C:\a.png 或 @/tmp/a.pdf".into(),
        http_ct_boundary: "boundary 在发送时自动生成".into(),
        http_form_key: "参数名".into(),
        http_form_val: "参数值".into(),
        http_raw_hint: "请求体内容(按原样发送)".into(),
        http_response: "响应".into(),
        http_resp_empty: "点击发送, 响应将显示在这里".into(),
        http_resp_headers: "响应头".into(),
        http_resp_body: "响应体".into(),
        http_history: "历史记录".into(),
        http_clear: "清空".into(),
        http_err_url: "URL 需以 http:// 或 https:// 开头".into(),
        http_err_header: "请求头包含非法字符".into(),
        http_err_send: "请求失败".into(),
        http_warn_truncated: "响应体过大, 已截断至前 10 MB".into(),
        http_warn_decode: "响应体存在无法解码的字节, 已用 U+FFFD 替换".into(),
    };
}

fn zh_tw() -> Texts {
    return Texts {
        app_title: "開發工具包".into(),
        tab_timestamp: "時間戳工具".into(),
        tab_encoding: "字元集轉換".into(),
        tab_url: "URL 編解碼".into(),
        tab_base64: "Base64".into(),
        btn_about: "關於".into(),
        btn_close: "關閉".into(),
        about_title: "關於開發工具包".into(),
        about_version: "版本".into(),
        about_author: "作者".into(),
        about_contact: "聯絡方式".into(),
        about_license: "授權條款".into(),
        about_built: "編譯資訊".into(),
        about_desc: "帶 UI 介面的常用開發工具集合, 包含時間戳轉換與字元集編碼轉換工具。".into(),
        ts_title: "時間戳工具".into(),
        ts_section_now: "目前時間".into(),
        ts_local_time: "本地時間".into(),
        ts_utc_time: "UTC 時間".into(),
        ts_millis: "毫秒時間戳".into(),
        ts_seconds: "秒時間戳".into(),
        ts_refresh: "重新整理".into(),
        ts_fill_ms: "填入毫秒戳".into(),
        ts_fill_s: "填入秒戳".into(),
        ts_fill_now_time: "填入目前時間".into(),
        ts_ts_to_time: "時間戳 → 時間字串".into(),
        ts_time_to_ts: "時間字串 → 時間戳".into(),
        ts_timestamp: "時間戳".into(),
        ts_time_str: "時間字串".into(),
        ts_unit: "單位".into(),
        ts_unit_auto: "自動辨識".into(),
        ts_unit_s: "秒".into(),
        ts_unit_ms: "毫秒".into(),
        ts_copy: "複製".into(),
        ts_hint_ts: "輸入時間戳, 如 1756600000000 或 1756600000".into(),
        ts_hint_dt: "如 2024-01-01 12:00:00".into(),
        ts_fmt_hint: "支援 YYYY-MM-DD HH:MM:SS(.fff)、YYYY/MM/DD、ISO8601(含時區) 等格式".into(),
        ts_err_not_int: "時間戳必須是整數".into(),
        ts_err_range: "時間戳超出可表示範圍".into(),
        ts_err_empty: "請輸入時間字串".into(),
        ts_err_parse: "無法解析時間字串".into(),
        ts_err_ambiguous: "本地時間解析存在歧義".into(),
        enc_title: "字元集轉換工具".into(),
        enc_charset: "字元集".into(),
        enc_decode: "hex → 文字 (解碼)".into(),
        enc_encode: "文字 → hex (編碼)".into(),
        enc_hex: "hex".into(),
        enc_text: "文字".into(),
        enc_copy: "複製結果".into(),
        enc_fill_text: "使用解碼結果".into(),
        enc_hint_hex: r"輸入 hex: 支援空格、0x 前綴、\xHH; \uXXXX / U+XXXX 僅限 UTF-16".into(),
        enc_hint_text: "輸入要編碼的文字".into(),
        enc_err_odd: "hex 長度必須為偶數(兩個字元表示一個位元組)".into(),
        enc_err_nonhex: "hex 包含非十六進位字元(僅允許 0-9 A-F)".into(),
        enc_err_utf16_escape: r"Unicode 跳脫(\u / U+)僅支援 UTF-16 字元集".into(),
        enc_err_utf16_odd: "UTF-16 資料位元組數必須為偶數".into(),
        enc_err_utf16_pair: "UTF-16 解碼失敗: 包含無效的代理項".into(),
        enc_warn_replace: "存在無法解碼的位元組, 已以 U+FFFD 替換".into(),
        enc_warn_gbk: "部分字元無法以 GBK 表示, 已以替代字元替換".into(),
        enc_warn_big5: "部分字元無法以 Big5 表示, 已以替代字元替換".into(),
        enc_guess: "猜測編碼".into(),
        enc_guess_alt: "其他可能".into(),
        gen_encode: "編碼".into(),
        gen_decode: "解碼".into(),
        gen_input: "輸入".into(),
        gen_output: "輸出".into(),
        gen_copy: "複製結果".into(),
        url_title: "URL 編解碼工具".into(),
        url_mode: "模式".into(),
        url_mode_std: "標準 (表單)".into(),
        url_mode_safe: "安全 (RFC 3986)".into(),
        url_hint: "輸入要編碼或解碼的文字".into(),
        url_note: "標準模式: 空白→+ 保留 A-Za-z0-9-_.*; 安全模式: 空白→%20 保留 A-Za-z0-9-_.~!*'()".into(),
        url_warn_invalid: "存在無效的 % 跳脫序列, 已按字面保留".into(),
        b64_title: "Base64 編解碼工具".into(),
        b64_alphabet: "字元表".into(),
        b64_std: "標準".into(),
        b64_urlsafe: "URL 安全".into(),
        b64_hint: "輸入要編碼或解碼的 Base64 文字".into(),
        b64_err_invalid: "無效的 Base64 資料(非法字元或長度不正確)".into(),
        tab_hash: "雜湊工具".into(),
        hash_title: "雜湊計算工具".into(),
        hash_algorithm: "演算法".into(),
        hash_hint: "輸入要計算雜湊的文字".into(),
        tab_checksum: "校驗工具".into(),
        checksum_title: "校驗計算工具".into(),
        checksum_algorithm: "演算法".into(),
        checksum_hint: "輸入要校驗的資料".into(),
        checksum_key: "密鑰 (Key)".into(),
        tab_http: "HTTP 請求".into(),
        http_title: "HTTP 請求工具".into(),
        http_url_hint: "https://httpbin.org/get".into(),
        http_send: "傳送".into(),
        http_sending: "請求中…".into(),
        http_headers: "請求標頭".into(),
        http_header_key: "名稱".into(),
        http_header_val: "值".into(),
        http_add: "新增".into(),
        http_remove: "移除".into(),
        http_body: "請求主體".into(),
        http_body_none: "無".into(),
        http_body_raw: "原文".into(),
        http_body_form: "URL 編碼".into(),
        http_body_multipart: "Form-Data".into(),
        http_raw_text: "文字".into(),
        http_multipart_hint: r"值以 @ 開頭表示傳送檔案, 如 @C:\a.png 或 @/tmp/a.pdf".into(),
        http_ct_boundary: "boundary 於傳送時自動產生".into(),
        http_form_key: "參數名稱".into(),
        http_form_val: "參數值".into(),
        http_raw_hint: "請求主體內容(按原樣傳送)".into(),
        http_response: "回應".into(),
        http_resp_empty: "點擊傳送, 回應將顯示在這裡".into(),
        http_resp_headers: "回應標頭".into(),
        http_resp_body: "回應主體".into(),
        http_history: "歷史記錄".into(),
        http_clear: "清空".into(),
        http_err_url: "URL 需以 http:// 或 https:// 開頭".into(),
        http_err_header: "請求標頭包含非法字元".into(),
        http_err_send: "請求失敗".into(),
        http_warn_truncated: "回應主體過大, 已截斷至前 10 MB".into(),
        http_warn_decode: "回應主體存在無法解碼的位元組, 已用 U+FFFD 取代".into(),
    };
}

fn en() -> Texts {
    return Texts {
        app_title: "Dev Toolkit".into(),
        tab_timestamp: "Timestamp".into(),
        tab_encoding: "Encoding".into(),
        tab_url: "URL".into(),
        tab_base64: "Base64".into(),
        btn_about: "About".into(),
        btn_close: "Close".into(),
        about_title: "About Dev Toolkit".into(),
        about_version: "Version".into(),
        about_author: "Author".into(),
        about_contact: "Contact".into(),
        about_license: "License".into(),
        about_built: "Build".into(),
        about_desc: "A collection of handy UI tools for developers, including timestamp conversion and charset encoding tools.".into(),
        ts_title: "Timestamp Tool".into(),
        ts_section_now: "Current Time".into(),
        ts_local_time: "Local time".into(),
        ts_utc_time: "UTC time".into(),
        ts_millis: "Millis timestamp".into(),
        ts_seconds: "Second timestamp".into(),
        ts_refresh: "Refresh".into(),
        ts_fill_ms: "Use current ms".into(),
        ts_fill_s: "Use current s".into(),
        ts_fill_now_time: "Use current time".into(),
        ts_ts_to_time: "Timestamp → Time String".into(),
        ts_time_to_ts: "Time String → Timestamp".into(),
        ts_timestamp: "Timestamp".into(),
        ts_time_str: "Time string".into(),
        ts_unit: "Unit".into(),
        ts_unit_auto: "Auto detect".into(),
        ts_unit_s: "Seconds".into(),
        ts_unit_ms: "Milliseconds".into(),
        ts_copy: "Copy".into(),
        ts_hint_ts: "Enter a timestamp, e.g. 1756600000000 or 1756600000".into(),
        ts_hint_dt: "e.g. 2024-01-01 12:00:00".into(),
        ts_fmt_hint: "Formats: YYYY-MM-DD HH:MM:SS(.fff), YYYY/MM/DD, ISO8601 (with timezone), etc.".into(),
        ts_err_not_int: "Timestamp must be an integer".into(),
        ts_err_range: "Timestamp out of representable range".into(),
        ts_err_empty: "Please enter a time string".into(),
        ts_err_parse: "Failed to parse time string".into(),
        ts_err_ambiguous: "Ambiguous local time conversion".into(),
        enc_title: "Charset Encoding Tool".into(),
        enc_charset: "Charset".into(),
        enc_decode: "hex → Text (Decode)".into(),
        enc_encode: "Text → hex (Encode)".into(),
        enc_hex: "hex".into(),
        enc_text: "Text".into(),
        enc_copy: "Copy result".into(),
        enc_fill_text: "Use decoded text".into(),
        enc_hint_hex: r"Enter hex: spaces, 0x prefix, \xHH; \uXXXX / U+XXXX only for UTF-16".into(),
        enc_hint_text: "Enter text to encode".into(),
        enc_err_odd: "hex length must be even (two chars per byte)".into(),
        enc_err_nonhex: "hex contains non-hex chars (only 0-9 A-F allowed)".into(),
        enc_err_utf16_escape: r"Unicode escapes (\u / U+) only supported with UTF-16 charsets".into(),
        enc_err_utf16_odd: "UTF-16 data must have an even byte count".into(),
        enc_err_utf16_pair: "UTF-16 decode failed: invalid surrogate pair".into(),
        enc_warn_replace: "Some bytes could not be decoded, replaced with U+FFFD".into(),
        enc_warn_gbk: "Some characters cannot be represented in GBK, replaced with fallback".into(),
        enc_warn_big5: "Some characters cannot be represented in Big5, replaced with fallback".into(),
        enc_guess: "Detected encoding".into(),
        enc_guess_alt: "Other possibilities".into(),
        gen_encode: "Encode".into(),
        gen_decode: "Decode".into(),
        gen_input: "Input".into(),
        gen_output: "Output".into(),
        gen_copy: "Copy result".into(),
        url_title: "URL Encoder/Decoder".into(),
        url_mode: "Mode".into(),
        url_mode_std: "Standard (Form)".into(),
        url_mode_safe: "Safe (RFC 3986)".into(),
        url_hint: "Enter text to encode or decode".into(),
        url_note: "Standard: space→+, keeps A-Za-z0-9-_.*; Safe: space→%20, keeps A-Za-z0-9-_.~!*'()".into(),
        url_warn_invalid: "Invalid % escape sequence found, kept as-is".into(),
        b64_title: "Base64 Encoder/Decoder".into(),
        b64_alphabet: "Alphabet".into(),
        b64_std: "Standard".into(),
        b64_urlsafe: "URL-safe".into(),
        b64_hint: "Enter text to encode or decode as Base64".into(),
        b64_err_invalid: "Invalid Base64 data (bad characters or length)".into(),
        tab_hash: "Hash".into(),
        hash_title: "Hash Tool".into(),
        hash_algorithm: "Algorithm".into(),
        hash_hint: "Enter text to hash".into(),
        tab_checksum: "Checksum".into(),
        checksum_title: "Checksum Tool".into(),
        checksum_algorithm: "Algorithm".into(),
        checksum_hint: "Enter data to verify".into(),
        checksum_key: "Key".into(),
        tab_http: "HTTP Request".into(),
        http_title: "HTTP Request Tool".into(),
        http_url_hint: "https://httpbin.org/get".into(),
        http_send: "Send".into(),
        http_sending: "Sending…".into(),
        http_headers: "Headers".into(),
        http_header_key: "Name".into(),
        http_header_val: "Value".into(),
        http_add: "Add".into(),
        http_remove: "Remove".into(),
        http_body: "Body".into(),
        http_body_none: "None".into(),
        http_body_raw: "Raw".into(),
        http_body_form: "URL-encoded".into(),
        http_body_multipart: "Form-Data".into(),
        http_raw_text: "Text".into(),
        http_multipart_hint: "Prefix a value with @ to send a file, e.g. @C:\\a.png or @/tmp/a.pdf".into(),
        http_ct_boundary: "boundary is generated automatically on send".into(),
        http_form_key: "Field name".into(),
        http_form_val: "Field value".into(),
        http_raw_hint: "Request body (sent as-is)".into(),
        http_response: "Response".into(),
        http_resp_empty: "Send a request — the response will appear here".into(),
        http_resp_headers: "Response Headers".into(),
        http_resp_body: "Response Body".into(),
        http_history: "History".into(),
        http_clear: "Clear".into(),
        http_err_url: "URL must start with http:// or https://".into(),
        http_err_header: "Header contains invalid characters".into(),
        http_err_send: "Request failed".into(),
        http_warn_truncated: "Response too large, truncated to first 10 MB".into(),
        http_warn_decode: "Some bytes could not be decoded, replaced with U+FFFD".into(),
    };
}

fn ja() -> Texts {
    return Texts {
        app_title: "開発ツールキット".into(),
        tab_timestamp: "タイムスタンプ".into(),
        tab_encoding: "文字コード変換".into(),
        tab_url: "URL 変換".into(),
        tab_base64: "Base64".into(),
        btn_about: "このアプリについて".into(),
        btn_close: "閉じる".into(),
        about_title: "開発ツールキットについて".into(),
        about_version: "バージョン".into(),
        about_author: "作者".into(),
        about_contact: "連絡先".into(),
        about_license: "ライセンス".into(),
        about_built: "ビルド情報".into(),
        about_desc: "開発者向けの UI 付き便利ツール集です。タイムスタンプ変換と文字コード変換を含みます。".into(),
        ts_title: "タイムスタンプツール".into(),
        ts_section_now: "現在時刻".into(),
        ts_local_time: "ローカル時刻".into(),
        ts_utc_time: "UTC 時刻".into(),
        ts_millis: "ミリ秒タイムスタンプ".into(),
        ts_seconds: "秒タイムスタンプ".into(),
        ts_refresh: "更新".into(),
        ts_fill_ms: "ミリ秒を入力".into(),
        ts_fill_s: "秒を入力".into(),
        ts_fill_now_time: "現在時刻を入力".into(),
        ts_ts_to_time: "タイムスタンプ → 時刻文字列".into(),
        ts_time_to_ts: "時刻文字列 → タイムスタンプ".into(),
        ts_timestamp: "タイムスタンプ".into(),
        ts_time_str: "時刻文字列".into(),
        ts_unit: "単位".into(),
        ts_unit_auto: "自動判別".into(),
        ts_unit_s: "秒".into(),
        ts_unit_ms: "ミリ秒".into(),
        ts_copy: "コピー".into(),
        ts_hint_ts: "タイムスタンプを入力(例: 1756600000000 または 1756600000)".into(),
        ts_hint_dt: "例: 2024-01-01 12:00:00".into(),
        ts_fmt_hint: "対応形式: YYYY-MM-DD HH:MM:SS(.fff)、YYYY/MM/DD、ISO8601(タイムゾーン付き) など".into(),
        ts_err_not_int: "タイムスタンプは整数である必要があります".into(),
        ts_err_range: "タイムスタンプが表現可能な範囲を超えています".into(),
        ts_err_empty: "時刻文字列を入力してください".into(),
        ts_err_parse: "時刻文字列を解析できません".into(),
        ts_err_ambiguous: "ローカル時刻の変換に曖昧さがあります".into(),
        enc_title: "文字コード変換ツール".into(),
        enc_charset: "文字コード".into(),
        enc_decode: "hex → テキスト (デコード)".into(),
        enc_encode: "テキスト → hex (エンコード)".into(),
        enc_hex: "hex".into(),
        enc_text: "テキスト".into(),
        enc_copy: "結果をコピー".into(),
        enc_fill_text: "デコード結果を使用".into(),
        enc_hint_hex: r"hex 入力: スペース・0x 接頭辞・\xHH 対応; \uXXXX / U+XXXX は UTF-16 のみ".into(),
        enc_hint_text: "エンコードするテキストを入力".into(),
        enc_err_odd: "hex の長さは偶数である必要があります(1バイト=2文字)".into(),
        enc_err_nonhex: "hex に16進数以外の文字が含まれています(0-9 A-F のみ)".into(),
        enc_err_utf16_escape: r"Unicode エスケープ(\u / U+)は UTF-16 文字セットのみ対応".into(),
        enc_err_utf16_odd: "UTF-16 データのバイト数は偶数である必要があります".into(),
        enc_err_utf16_pair: "UTF-16 デコード失敗: 無効なサロゲートペア".into(),
        enc_warn_replace: "デコードできないバイトを U+FFFD に置換しました".into(),
        enc_warn_gbk: "GBK で表現できない文字を代替文字に置換しました".into(),
        enc_warn_big5: "Big5 で表現できない文字を代替文字に置換しました".into(),
        enc_guess: "推定エンコーディング".into(),
        enc_guess_alt: "その他の可能性".into(),
        gen_encode: "エンコード".into(),
        gen_decode: "デコード".into(),
        gen_input: "入力".into(),
        gen_output: "出力".into(),
        gen_copy: "結果をコピー".into(),
        url_title: "URL エンコード/デコード".into(),
        url_mode: "モード".into(),
        url_mode_std: "標準".into(),
        url_mode_safe: "セーフ".into(),
        url_hint: "エンコードまたはデコードするテキストを入力".into(),
        url_note: "標準: 空白→+、A-Za-z0-9-_.* を保持; セーフ: 空白→%20、A-Za-z0-9-_.~!*'() を保持".into(),
        url_warn_invalid: "無効な % エスケープが見つかりましたが、そのまま保持しました".into(),
        b64_title: "Base64 エンコード/デコード".into(),
        b64_alphabet: "文字表".into(),
        b64_std: "標準".into(),
        b64_urlsafe: "URL セーフ".into(),
        b64_hint: "Base64 にエンコード/デコードするテキストを入力".into(),
        b64_err_invalid: "無効な Base64 データ(不正な文字または長さ)".into(),
        tab_hash: "ハッシュ".into(),
        hash_title: "ハッシュ計算ツール".into(),
        hash_algorithm: "アルゴリズム".into(),
        hash_hint: "ハッシュを計算するテキストを入力".into(),
        tab_checksum: "チェックサム".into(),
        checksum_title: "チェックサム計算ツール".into(),
        checksum_algorithm: "アルゴリズム".into(),
        checksum_hint: "検証するデータを入力".into(),
        checksum_key: "鍵 (Key)".into(),
        tab_http: "HTTP リクエスト".into(),
        http_title: "HTTP リクエストツール".into(),
        http_url_hint: "https://httpbin.org/get".into(),
        http_send: "送信".into(),
        http_sending: "送信中…".into(),
        http_headers: "ヘッダー".into(),
        http_header_key: "名前".into(),
        http_header_val: "値".into(),
        http_add: "追加".into(),
        http_remove: "削除".into(),
        http_body: "ボディ".into(),
        http_body_none: "なし".into(),
        http_body_raw: "テキスト".into(),
        http_body_form: "URLエンコード".into(),
        http_body_multipart: "Form-Data".into(),
        http_raw_text: "テキスト".into(),
        http_multipart_hint: "値の先頭に @ を付けるとファイルを送信(例: @C:\\a.png または @/tmp/a.pdf)".into(),
        http_ct_boundary: "boundary は送信時に自動生成されます".into(),
        http_form_key: "項目名".into(),
        http_form_val: "値".into(),
        http_raw_hint: "リクエストボディ(そのまま送信)".into(),
        http_response: "レスポンス".into(),
        http_resp_empty: "送信すると、レスポンスがここに表示されます".into(),
        http_resp_headers: "レスポンスヘッダー".into(),
        http_resp_body: "レスポンスボディ".into(),
        http_history: "履歴".into(),
        http_clear: "クリア".into(),
        http_err_url: "URL は http:// または https:// で始まる必要があります".into(),
        http_err_header: "ヘッダーに不正な文字が含まれています".into(),
        http_err_send: "リクエスト失敗".into(),
        http_warn_truncated: "レスポンスが大きすぎるため、先頭 10 MB に切り詰めました".into(),
        http_warn_decode: "デコードできないバイトを U+FFFD に置換しました".into(),
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 旧版本语言文件缺少新增字段时, 用同名内置语言的翻译补齐(而非中文兜底)
    #[test]
    fn lang_file_missing_fields_fall_back_to_builtin() {
        let dir = std::env::temp_dir().join(format!("devtoolkit-i18n-test-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        // 模拟升级前的 en.json: 只有最早的几个字段
        fs::write(
            dir.join("en.json"),
            r#"{"name":"English","texts":{"app_title":"Dev Toolkit"}}"#,
        )
        .unwrap();

        let mut langs = builtin_langs();
        load_lang_files(&dir, &mut langs);
        let _ = fs::remove_dir_all(&dir);

        let en = langs.iter().find(|l| l.code == "en").unwrap();
        // 用户已有的翻译保留
        assert_eq!(en.texts.app_title, "Dev Toolkit");
        // 缺失字段用内置英文补齐, 不回退到中文兜底
        assert_eq!(en.texts.tab_checksum, "Checksum");
        assert_eq!(en.texts.tab_http, "HTTP Request");
        assert_eq!(en.texts.http_send, "Send");
        assert_ne!(en.texts.tab_checksum, "校验工具");
    }
}
