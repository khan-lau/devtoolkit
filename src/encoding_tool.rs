//! 字符集编码转换工具
//!
//! 支持 UTF-8 / GBK / UTF-16(LE/BE) 与 hex 字符串的双向转换:
//! - hex 字符串解码为指定字符集的文本
//! - 指定字符集的文本编码为 hex 字符串

use eframe::egui;

use crate::i18n::Texts;
use crate::theme;

/// 支持的字符集
#[derive(Clone, Copy, PartialEq, Eq)]
enum Charset {
    Utf8,
    Gbk,
    Gb18030, /// GB18030: 完整覆盖 Unicode(含 4 字节扩展序列)
    Big5,    /// Big5: 繁体中文编码
    Utf16Le,
    Utf16Be,
}

impl Charset {
    fn name(self) -> &'static str {
        match self {
            Charset::Utf8 => "UTF-8",
            Charset::Gbk => "GBK",
            Charset::Gb18030 => "GB18030",
            Charset::Big5 => "Big5",
            Charset::Utf16Le => "UTF-16 LE",
            Charset::Utf16Be => "UTF-16 BE",
        }
    }
}

/// 编码转换错误
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum EncErr {
    /// hex 长度不是偶数
    OddLength,
    /// hex 包含非十六进制字符
    NonHex,
    /// UTF-16 数据字节数不是偶数
    Utf16Odd,
    /// UTF-16 包含无效代理项
    Utf16Pair,
    /// Unicode 码点转义(\u/U+)仅支持 UTF-16 字符集
    EscapeUtf16Only,
}

impl EncErr {
    /// 转换为当前语言的错误文案
    fn msg(self, t: &Texts) -> String {
        match self {
            EncErr::OddLength => t.enc_err_odd.to_string(),
            EncErr::NonHex => t.enc_err_nonhex.to_string(),
            EncErr::Utf16Odd => t.enc_err_utf16_odd.to_string(),
            EncErr::Utf16Pair => t.enc_err_utf16_pair.to_string(),
            EncErr::EscapeUtf16Only => t.enc_err_utf16_escape.to_string(),
        }
    }
}

/// 转换状态: 无输入 / 有警告 / 出错
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Status {
    Idle,
    /// 警告标识: "replace" | "gbk"
    Warn(&'static str),
    Err(EncErr),
}

/// 字符集转换工具界面状态
pub struct EncodingTool {
    t: Texts,

    charset: Charset,

    // hex -> 文本
    hex_input: String,
    decoded: String,
    decode_status: Status,
    /// 可能的编码列表: 第一项为 chardetng 最佳猜测, 其余为可无损解码的候选 (空表示无法猜测)
    decode_possible: Vec<&'static str>,
    decode_last_key: (Charset, String),

    // 文本 -> hex
    text_input: String,
    encoded: String,
    encode_status: Status,
    encode_last_key: (Charset, String),
}

impl EncodingTool {
    pub fn new(t: Texts) -> Self {
        Self {
            t,
            charset: Charset::Utf8,
            hex_input: String::new(),
            decoded: String::new(),
            decode_status: Status::Idle,
            decode_possible: Vec::new(),
            decode_last_key: (Charset::Utf8, String::new()),
            text_input: String::new(),
            encoded: String::new(),
            encode_status: Status::Idle,
            encode_last_key: (Charset::Utf8, String::new()),
        }
    }

    /// 语言切换时更新文案
    pub fn set_lang(&mut self, t: Texts) {
        self.t = t;
    }

    /// 渲染工具界面
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        let t = self.t.clone();
        self.update_decode();
        self.update_encode();

        egui::ScrollArea::vertical().show(ui, |ui| {
            ui.add_space(4.0);
            ui.label(egui::RichText::new(&t.enc_title).size(20.0).strong());
            ui.add_space(8.0);
            theme::card(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(&t.enc_charset);
                    egui::ComboBox::from_id_salt("charset")
                        .selected_text(self.charset.name())
                        .show_ui(ui, |ui| {
                            if theme::selectable_label(ui, self.charset == Charset::Utf8, "UTF-8")
                            {
                                self.charset = Charset::Utf8;
                            }
                            if theme::selectable_label(ui, self.charset == Charset::Gbk, "GBK") {
                                self.charset = Charset::Gbk;
                            }
                            if theme::selectable_label(ui, self.charset == Charset::Gb18030, "GB18030")
                            {
                                self.charset = Charset::Gb18030;
                            }
                            if theme::selectable_label(ui, self.charset == Charset::Big5, "Big5") {
                                self.charset = Charset::Big5;
                            }
                            if theme::selectable_label(ui, self.charset == Charset::Utf16Le, "UTF-16 LE")
                            {
                                self.charset = Charset::Utf16Le;
                            }
                            if theme::selectable_label(ui, self.charset == Charset::Utf16Be, "UTF-16 BE")
                            {
                                self.charset = Charset::Utf16Be;
                            }
                        });
                });
            });
            ui.add_space(12.0);
            theme::card(ui, |ui| self.section_decode(ui));
            ui.add_space(12.0);
            theme::card(ui, |ui| self.section_encode(ui));
            ui.add_space(8.0);
        });
    }

    /// 渲染状态提示(警告/错误)
    fn status_line(ui: &mut egui::Ui, t: &Texts, status: Status) {
        match status {
            Status::Idle => {}
            Status::Warn(key) => {
                let text = match key {
                    "replace" => &t.enc_warn_replace,
                    "big5" => &t.enc_warn_big5,
                    _ => &t.enc_warn_gbk,
                };
                ui.colored_label(theme::WARN, text);
            }
            Status::Err(e) => {
                ui.colored_label(theme::ERROR, e.msg(t));
            }
        }
    }

    /// hex -> 文本 区
    fn section_decode(&mut self, ui: &mut egui::Ui) {
        let t = self.t.clone();
        theme::section_title(ui, &t.enc_decode);

        ui.add(
            egui::TextEdit::singleline(&mut self.hex_input)
                .hint_text(t.enc_hint_hex.clone())
                .margin(egui::Margin::symmetric(8, 14))
                .desired_width(f32::INFINITY)
                .vertical_align(egui::Align::Center),
        );

        Self::status_line(ui, &t, self.decode_status);

        // 编码猜测: 第一项为 chardetng 最佳猜测, 其余为可无损解码的其他候选
        if let Some((best, rest)) = self.decode_possible.split_first() {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(&t.enc_guess).weak());
                ui.label(egui::RichText::new(*best).strong().color(theme::ACCENT));
            });
            if !rest.is_empty() {
                let detail = format!("{}: {}", t.enc_guess_alt, rest.join(", "));
                ui.label(egui::RichText::new(detail).weak());
            }
        }

        ui.label(&t.enc_text);
        ui.add(
            egui::TextEdit::multiline(&mut self.decoded)
                .desired_rows(4)
                .desired_width(f32::INFINITY),
        );
        if !self.decoded.is_empty() {
            theme::copy_button(ui, &t.enc_copy, &self.decoded);
        }
    }

    /// 文本 -> hex 区
    fn section_encode(&mut self, ui: &mut egui::Ui) {
        let t = self.t.clone();
        theme::section_title(ui, &t.enc_encode);

        ui.add(
            egui::TextEdit::singleline(&mut self.text_input)
                .hint_text(t.enc_hint_text.clone())
                .margin(egui::Margin::symmetric(8, 18))
                .desired_width(f32::INFINITY)
                .vertical_align(egui::Align::Center),
        );

        Self::status_line(ui, &t, self.encode_status);

        ui.label(&t.enc_hex);
        ui.add(
            egui::TextEdit::multiline(&mut self.encoded)
                .desired_rows(4)
                .desired_width(f32::INFINITY),
        );

        ui.horizontal(|ui| {
            if !self.encoded.is_empty() {
                theme::copy_button(ui, &t.enc_copy, &self.encoded);
            }
            // 操作优化: 将解码结果填入文本输入框, 便于继续编辑/重新编码
            if !self.decoded.is_empty() {
                if ui.button(&t.enc_fill_text).clicked() {
                    self.text_input = self.decoded.clone();
                }
            }
        });
    }

    /// 输入变化时重新计算 hex -> 文本
    fn update_decode(&mut self) {
        let key = (self.charset, self.hex_input.clone());
        if self.decode_last_key == key {
            return;
        }
        self.decode_last_key = key;

        let (text, status, possible) = decode_hex(&self.hex_input, self.charset);
        self.decoded = text;
        self.decode_status = status;
        self.decode_possible = possible;
    }

    /// 输入变化时重新计算 文本 -> hex
    fn update_encode(&mut self) {
        let key = (self.charset, self.text_input.clone());
        if self.encode_last_key == key {
            return;
        }
        self.encode_last_key = key;

        let (hex, status) = encode_hex(&self.text_input, self.charset);
        self.encoded = hex;
        self.encode_status = status;
    }
}

/// 解析 hex 字符串为字节序列
///
/// 支持的格式(空白均忽略, 可任意混用):
/// - 连续或空白分隔的 hex 对: `48656C6C6F` / `48 65 6C`
/// - 任意位置、可重复的 `0x`/`0X` 前缀: `0x48 0x65 6C`
/// - C 风格字节转义 `\xHH`: `\x48\x65`
/// - Unicode 码点转义(仅 UTF-16 字符集): 定长 `\uXXXX`、变长 `\u{...}` 与 `U+XXXX`
fn hex_to_bytes(input: &str, charset: Charset) -> Result<Vec<u8>, EncErr> {
    let chars: Vec<char> = input.chars().filter(|c| !c.is_whitespace()).collect();
    let mut bytes = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        // `0x`/`0X` 前缀: 任意位置可重复, 直接跳过
        if c == '0' && chars.get(i + 1).is_some_and(|&n| n == 'x' || n == 'X') {
            i += 2;
            continue;
        }
        // `U+XXXX` / `u+XXXX`: Unicode 码点表示, 变长 1-6 位, 仅 UTF-16 字符集有效
        if (c == 'U' || c == 'u') && chars.get(i + 1) == Some(&'+') {
            if !matches!(charset, Charset::Utf16Le | Charset::Utf16Be) {
                return Err(EncErr::EscapeUtf16Only);
            }
            let mut j = i + 2;
            let mut cp: u32 = 0;
            let mut any = false;
            while let Some(&ch) = chars.get(j) {
                if !ch.is_ascii_hexdigit() {
                    break;
                }
                cp = cp * 16 + ch.to_digit(16).unwrap();
                any = true;
                j += 1;
            }
            if !any {
                return Err(EncErr::NonHex);
            }
            let ch = char::from_u32(cp).ok_or(EncErr::NonHex)?;
            append_char_bytes(&mut bytes, ch, charset);
            i = j;
            continue;
        }
        // 转义序列
        if c == '\\' {
            let next = *chars.get(i + 1).ok_or(EncErr::NonHex)?;
            match next {
                // \xHH: 1 字节
                'x' => {
                    let hi = hex_val(*chars.get(i + 2).ok_or(EncErr::NonHex)?)?;
                    let lo = hex_val(*chars.get(i + 3).ok_or(EncErr::NonHex)?)?;
                    bytes.push((hi << 4) | lo);
                    i += 4;
                }
                // \uXXXX / \u{...}: Unicode 码点, 仅 UTF-16 字符集有效
                // \uXXXX 定长 4 位; \u{...} 变长 1-6 位
                'u' => {
                    if !matches!(charset, Charset::Utf16Le | Charset::Utf16Be) {
                        return Err(EncErr::EscapeUtf16Only);
                    }
                    let (cp, consumed) = if chars.get(i + 2) == Some(&'{') {
                        let mut j = i + 3;
                        let mut cp: u32 = 0;
                        let mut any = false;
                        while let Some(&ch) = chars.get(j) {
                            if ch == '}' {
                                break;
                            }
                            if !ch.is_ascii_hexdigit() {
                                return Err(EncErr::NonHex);
                            }
                            cp = cp * 16 + ch.to_digit(16).unwrap();
                            any = true;
                            j += 1;
                        }
                        if !any || chars.get(j) != Some(&'}') {
                            return Err(EncErr::NonHex);
                        }
                        (cp, j + 1 - i)
                    } else {
                        // \uXXXX 定长 4 位
                        let mut cp: u32 = 0;
                        for k in 0..4 {
                            let ch = *chars.get(i + 2 + k).ok_or(EncErr::NonHex)?;
                            if !ch.is_ascii_hexdigit() {
                                return Err(EncErr::NonHex);
                            }
                            cp = cp * 16 + ch.to_digit(16).unwrap();
                        }
                        (cp, 6)
                    };
                    let ch = char::from_u32(cp).ok_or(EncErr::NonHex)?;
                    append_char_bytes(&mut bytes, ch, charset);
                    i += consumed;
                }
                _ => return Err(EncErr::NonHex),
            }
            continue;
        }
        // 普通 hex 对
        if !c.is_ascii_hexdigit() {
            return Err(EncErr::NonHex);
        }
        let hi = c.to_digit(16).unwrap() as u8;
        let lo = match chars.get(i + 1) {
            Some(&n) if n.is_ascii_hexdigit() => n.to_digit(16).unwrap() as u8,
            Some(_) => return Err(EncErr::NonHex), // 成对中的第二个字符非 hex
            None => return Err(EncErr::OddLength), // 末尾仅剩奇数个 hex 字符
        };
        bytes.push((hi << 4) | lo);
        i += 2;
    }
    Ok(bytes)
}

/// hex 字符转数值
fn hex_val(c: char) -> Result<u8, EncErr> {
    c.to_digit(16).map(|v| v as u8).ok_or(EncErr::NonHex)
}

/// 将 Unicode 码点字符按当前字符集编码为字节追加到输出
fn append_char_bytes(out: &mut Vec<u8>, c: char, charset: Charset) {
    match charset {
        Charset::Utf8 => {
            let mut buf = [0u8; 4];
            out.extend_from_slice(c.encode_utf8(&mut buf).as_bytes());
        }
        Charset::Gbk => {
            let s = c.to_string();
            let (b, _, _) = encoding_rs::GBK.encode(&s);
            out.extend_from_slice(&b);
        }
        Charset::Gb18030 => {
            let s = c.to_string();
            let (b, _, _) = encoding_rs::GB18030.encode(&s);
            out.extend_from_slice(&b);
        }
        Charset::Big5 => {
            let s = c.to_string();
            let (b, _, _) = encoding_rs::BIG5.encode(&s);
            out.extend_from_slice(&b);
        }
        Charset::Utf16Le => {
            let mut buf = [0u16; 2];
            for u in c.encode_utf16(&mut buf) {
                out.extend_from_slice(&u.to_le_bytes());
            }
        }
        Charset::Utf16Be => {
            let mut buf = [0u16; 2];
            for u in c.encode_utf16(&mut buf) {
                out.extend_from_slice(&u.to_be_bytes());
            }
        }
    }
}

/// 按指定字符集将 hex 字符串解码为文本
///
/// 返回值: (解码文本, 状态, 可能的编码列表)
/// 可能的编码列表: 第一项为 chardetng 最佳猜测, 其余为能无损解码该字节序列的候选编码。
fn decode_hex(input: &str, charset: Charset) -> (String, Status, Vec<&'static str>) {
    let bytes = match hex_to_bytes(input, charset) {
        Ok(bytes) => bytes,
        Err(e) => return (String::new(), Status::Err(e), Vec::new()),
    };
    if bytes.is_empty() {
        return (String::new(), Status::Idle, Vec::new());
    }

    let (text, status) = match charset {
        Charset::Utf8 => {
            let (text, _, had_errors) = encoding_rs::UTF_8.decode(&bytes);
            let status = had_errors.then_some(Status::Warn("replace")).unwrap_or(Status::Idle);
            (text.into_owned(), status)
        }
        Charset::Gbk => {
            let (text, _, had_errors) = encoding_rs::GBK.decode(&bytes);
            let status = had_errors.then_some(Status::Warn("replace")).unwrap_or(Status::Idle);
            (text.into_owned(), status)
        }
        Charset::Gb18030 => {
            // GB18030 解码器与 GBK 相同(两者互为超集/子集关系中的解码端), 可无损解码 GBK 数据
            let (text, _, had_errors) = encoding_rs::GB18030.decode(&bytes);
            let status = had_errors.then_some(Status::Warn("replace")).unwrap_or(Status::Idle);
            (text.into_owned(), status)
        }
        Charset::Big5 => {
            let (text, _, had_errors) = encoding_rs::BIG5.decode(&bytes);
            let status = had_errors.then_some(Status::Warn("replace")).unwrap_or(Status::Idle);
            (text.into_owned(), status)
        }
        Charset::Utf16Le => {
            let data = strip_bom(&bytes, [0xFF, 0xFE]);
            match utf16_bytes_to_string(data, u16::from_le_bytes) {
                Ok(text) => (text, Status::Idle),
                Err(e) => (String::new(), Status::Err(e)),
            }
        }
        Charset::Utf16Be => {
            let data = strip_bom(&bytes, [0xFE, 0xFF]);
            match utf16_bytes_to_string(data, u16::from_be_bytes) {
                Ok(text) => (text, Status::Idle),
                Err(e) => (String::new(), Status::Err(e)),
            }
        }
    };

    (text, status, possible_encodings(&bytes))
}

/// 参与"可能编码"展示的多字节编码
///
/// 单字节编码(如 windows-1252)可无损解码任意字节序列, 列出来没有区分意义, 故排除。
/// GBK 与 GB18030 的解码器相同, 会在 [`possible_encodings`] 中按数据是否含
/// 4 字节扩展序列二选一, 避免重复列出。
const MULTIBYTE_ENCODINGS: &[&encoding_rs::Encoding] = &[
    encoding_rs::UTF_8,
    encoding_rs::GBK,
    encoding_rs::GB18030,
    encoding_rs::BIG5,
    encoding_rs::EUC_KR,
    encoding_rs::SHIFT_JIS,
    encoding_rs::EUC_JP,
];

/// 找出字节序列的所有可能编码: chardetng 最佳猜测排第一, 其余为能无损解码的候选
fn possible_encodings(bytes: &[u8]) -> Vec<&'static str> {
    // 验证各多字节编码能否无损解码
    let mut list: Vec<&'static str> = MULTIBYTE_ENCODINGS
        .iter()
        .filter(|enc| enc.decode_without_bom_handling_and_without_replacement(bytes).is_some())
        .map(|enc| {
            let name = enc.name();
            // GB18030 的规范名为小写 "gb18030", 展示时统一为大写
            if name == "gb18030" { "GB18030" } else { name }
        })
        .collect();
    // GBK 与 GB18030 的解码器相同, 二者总会同时通过验证; 按数据是否含
    // 4 字节扩展序列保留更精确的一个, 避免冗余。
    if list.iter().any(|&e| e == "GBK") && list.iter().any(|&e| e == "GB18030") {
        if is_gb18030_ext(bytes) {
            list.retain(|&e| e != "GBK");
        } else {
            list.retain(|&e| e != "GB18030");
        }
    }
    // UTF-16 的"无损解码"验证无区分度(任意偶数长度的字节序列都能解码), 不能靠
    // 解码验证识别; 带 BOM 时直接识别, 无 BOM 时用 NUL 字节占比启发式补充。
    if bytes.starts_with(&[0xFF, 0xFE]) {
        list.push("UTF-16LE");
    } else if bytes.starts_with(&[0xFE, 0xFF]) {
        list.push("UTF-16BE");
    } else if !bytes.is_empty() && bytes.len() % 2 == 0 {
        // 无 BOM: ASCII/半角混合的 UTF-16 文本中 NUL 字节占比约 50%, 据此识别
        let nul = bytes.iter().filter(|&&b| b == 0).count();
        if nul * 4 >= bytes.len() {
            // 按 NUL 出现位置判断字节序: LE 的 NUL 落在奇数位, BE 落在偶数位
            let odd_nul = bytes
                .iter()
                .enumerate()
                .filter(|&(i, b)| i % 2 == 1 && *b == 0)
                .count();
            if odd_nul * 2 >= nul {
                list.push("UTF-16LE");
            } else {
                list.push("UTF-16BE");
            }
        }
    }
    // chardetng 最佳猜测置顶
    if let Some(best) = guess_encoding(bytes) {
        // chardetng 不识别 GB18030(只认 GBK): 数据实为 4 字节扩展时修正猜测
        let best = if best == "GBK" && list.contains(&"GB18030") {
            "GB18030"
        } else {
            best
        };
        list.retain(|&e| e != best);
        list.insert(0, best);
    }
    list
}

/// 检测字节序列中是否包含 GB18030 的 4 字节扩展序列
///
/// GB18030 4 字节序列格式: 0x81-0xFE + 0x30-0x39 + 0x81-0xFE + 0x30-0x39。
/// GBK 2 字节序列的 trail 范围为 0x40-0xFE(除 0x7F), 与 4 字节序列的第二个
/// 字节 0x30-0x39 范围不重叠, 因此从 lead 字节即可直接判断序列类型。
fn is_gb18030_ext(bytes: &[u8]) -> bool {
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        if b >= 0x81 && b <= 0xFE {
            if i + 3 < bytes.len()
                && (0x30..=0x39).contains(&bytes[i + 1])
                && (0x81..=0xFE).contains(&bytes[i + 2])
                && (0x30..=0x39).contains(&bytes[i + 3])
            {
                return true;
            }
            i += 2; // 2 字节 GBK 序列
        } else {
            i += 1;
        }
    }
    false
}

/// 使用 chardetng 启发式猜测字节序列的最佳编码
///
/// 返回 WHATWG 编码标签(如 `UTF-8`、`GBK`、`Shift_JIS`), 数据过少时返回 None。
fn guess_encoding(bytes: &[u8]) -> Option<&'static str> {
    if bytes.len() < 2 {
        return None;
    }
    let mut detector = chardetng::EncodingDetector::new();
    detector.feed(bytes, true);
    Some(detector.guess(None, true).name())
}

/// 按指定字符集将文本编码为 hex 字符串
fn encode_hex(text: &str, charset: Charset) -> (String, Status) {
    let (bytes, warning) = match charset {
        Charset::Utf8 => (text.as_bytes().to_vec(), None),
        Charset::Gbk => {
            let (bytes, _, had_errors) = encoding_rs::GBK.encode(text);
            (bytes.into_owned(), had_errors.then_some("gbk"))
        }
        Charset::Gb18030 => {
            // GB18030 编码器覆盖全部 Unicode, 不会出现无法表示的字符
            let (bytes, _, had_errors) = encoding_rs::GB18030.encode(text);
            (bytes.into_owned(), had_errors.then_some("gbk"))
        }
        Charset::Big5 => {
            let (bytes, _, had_errors) = encoding_rs::BIG5.encode(text);
            (bytes.into_owned(), had_errors.then_some("big5"))
        }
        Charset::Utf16Le => (
            text.encode_utf16().flat_map(u16::to_le_bytes).collect(),
            None,
        ),
        Charset::Utf16Be => (
            text.encode_utf16().flat_map(u16::to_be_bytes).collect(),
            None,
        ),
    };

    let status = warning.map(Status::Warn).unwrap_or(Status::Idle);
    (bytes.iter().map(|b| format!("{b:02X}")).collect(), status)
}

/// 将 UTF-16 字节序列转为字符串
fn utf16_bytes_to_string(
    bytes: &[u8],
    from_bytes: impl Fn([u8; 2]) -> u16,
) -> Result<String, EncErr> {
    if bytes.len() % 2 != 0 {
        return Err(EncErr::Utf16Odd);
    }
    let units: Vec<u16> = bytes
        .chunks_exact(2)
        .map(|c| from_bytes([c[0], c[1]]))
        .collect();
    String::from_utf16(&units).map_err(|_| EncErr::Utf16Pair)
}

/// 跳过可选的 BOM 前缀
fn strip_bom(bytes: &[u8], bom: [u8; 2]) -> &[u8] {
    if bytes.starts_with(&bom) {
        &bytes[2..]
    } else {
        bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(s: &str) -> Vec<u8> {
        s.as_bytes().to_vec()
    }

    fn utf16le(s: &str) -> Vec<u8> {
        s.encode_utf16().flat_map(u16::to_le_bytes).collect()
    }

    fn utf16be(s: &str) -> Vec<u8> {
        s.encode_utf16().flat_map(u16::to_be_bytes).collect()
    }

    #[test]
    fn hex_basic_formats() {
        assert_eq!(hex_to_bytes("48656C6C6F", Charset::Utf8).unwrap(), b("Hello"));
        assert_eq!(hex_to_bytes("48 65 6C 6C 6F", Charset::Utf8).unwrap(), b("Hello"));
        assert_eq!(
            hex_to_bytes("48\t65\n6C 6C6F", Charset::Utf8).unwrap(),
            b("Hello")
        );
    }

    #[test]
    fn hex_0x_prefixes() {
        assert_eq!(hex_to_bytes("0x48 0x65 6C 6C 6F", Charset::Utf8).unwrap(), b("Hello"));
        assert_eq!(hex_to_bytes("0X480X65", Charset::Utf8).unwrap(), b("He"));
    }

    #[test]
    fn hex_x_escapes() {
        assert_eq!(hex_to_bytes(r"\x48\x65\x6C\x6C\x6F", Charset::Utf8).unwrap(), b("Hello"));
    }

    #[test]
    fn hex_u_escapes() {
        // UTF-16 LE/BE: 码点 → UTF-16 码元
        assert_eq!(hex_to_bytes(r"\u4F60", Charset::Utf16Le).unwrap(), utf16le("\u{4F60}"));
        assert_eq!(hex_to_bytes(r"\u4F60", Charset::Utf16Be).unwrap(), utf16be("\u{4F60}"));
        assert_eq!(hex_to_bytes(r"\u{4F60}", Charset::Utf16Le).unwrap(), utf16le("\u{4F60}"));
        // \uXXXX 定长 4 位, \u{...} 变长支持增补平面(代理对)
        assert_eq!(hex_to_bytes(r"\u{1F600}", Charset::Utf16Le).unwrap(), utf16le("\u{1F600}"));
        // 非 UTF-16 字符集不支持
        assert_eq!(hex_to_bytes(r"\u4F60", Charset::Utf8), Err(EncErr::EscapeUtf16Only));
        assert_eq!(hex_to_bytes(r"\u4F60", Charset::Gbk), Err(EncErr::EscapeUtf16Only));
    }

    #[test]
    fn hex_mixed() {
        // \x 字节转义与 \u 码点转义混用(UTF-16LE)
        assert_eq!(
            hex_to_bytes(r"\x48\x65 \u4F60", Charset::Utf16Le).unwrap(),
            [b'H', b'e', utf16le("\u{4F60}")[0], utf16le("\u{4F60}")[1]].to_vec()
        );
    }

    #[test]
    fn hex_u_plus_formats() {
        assert_eq!(hex_to_bytes("U+4F60", Charset::Utf16Le).unwrap(), utf16le("\u{4F60}"));
        assert_eq!(hex_to_bytes("u+4f60", Charset::Utf16Be).unwrap(), utf16be("\u{4F60}"));
        // 变长码点(emoji → 代理对)
        assert_eq!(hex_to_bytes("U+1F600", Charset::Utf16Le).unwrap(), utf16le("\u{1F600}"));
        // 与 0x 前缀混用
        assert_eq!(
            hex_to_bytes("0x48 U+4F60", Charset::Utf16Le).unwrap(),
            [b'H', utf16le("\u{4F60}")[0], utf16le("\u{4F60}")[1]].to_vec()
        );
        // 非 UTF-16 字符集不支持
        assert_eq!(hex_to_bytes("U+4F60", Charset::Utf8), Err(EncErr::EscapeUtf16Only));
        // 无码点数字
        assert_eq!(hex_to_bytes("U+", Charset::Utf16Le), Err(EncErr::NonHex));
        // 超出 Unicode 范围
        assert_eq!(hex_to_bytes("U+110000", Charset::Utf16Le), Err(EncErr::NonHex));
        // U 后无 '+' 视为普通非 hex 字符
        assert_eq!(hex_to_bytes("U4F60", Charset::Utf16Le), Err(EncErr::NonHex));
    }

    #[test]
    fn hex_invalid() {
        // 奇数个 hex 字符
        assert_eq!(hex_to_bytes("486", Charset::Utf8), Err(EncErr::OddLength));
        // 非 hex 字符
        assert_eq!(hex_to_bytes("4G", Charset::Utf8), Err(EncErr::NonHex));
        // 孤立反斜杠
        assert_eq!(hex_to_bytes(r"\x4", Charset::Utf8), Err(EncErr::NonHex));
        // 空 \u
        assert_eq!(hex_to_bytes(r"\u{}", Charset::Utf16Le), Err(EncErr::NonHex));
        // 无效码点(超出 Unicode 范围)
        assert_eq!(hex_to_bytes(r"\u{110000}", Charset::Utf16Le), Err(EncErr::NonHex));
    }

    #[test]
    fn guess_encoding_detects() {
        // UTF-8 中文
        assert_eq!(guess_encoding("中文".as_bytes()), Some("UTF-8"));
        // 纯 ASCII(允许 UTF-8 时判为 utf-8)
        assert_eq!(guess_encoding(b"hello"), Some("UTF-8"));
        // GBK 中文
        let (gbk_bytes, _, _) = encoding_rs::GBK.encode("中文");
        assert_eq!(guess_encoding(&gbk_bytes), Some("GBK"));
        // 数据过少无法猜测
        assert_eq!(guess_encoding(&[0x41]), None);
    }

    #[test]
    fn possible_encodings_lists_candidates() {
        // GBK "你好" (C4 E3 BA C3): 字节对落在 EUC-KR 韩文区, 最佳猜测为 EUC-KR, GBK 也在候选
        let (gbk_bytes, _, _) = encoding_rs::GBK.encode("你好");
        let list = possible_encodings(&gbk_bytes);
        assert_eq!(list.first(), Some(&"EUC-KR"));
        assert!(list.contains(&"GBK"));
        assert!(list.contains(&"EUC-KR"));
        // UTF-8 中文: 最佳猜测为 UTF-8, 且存在其他可无损解码的候选
        let list = possible_encodings("中文".as_bytes());
        assert_eq!(list.first(), Some(&"UTF-8"));
        assert!(list.len() >= 2);
        // UTF-16LE BOM: 补充 UTF-16LE
        let mut bom = vec![0xFF, 0xFE];
        bom.extend_from_slice(
            &"你".encode_utf16().flat_map(u16::to_le_bytes).collect::<Vec<u8>>(),
        );
        let list = possible_encodings(&bom);
        assert!(list.contains(&"UTF-16LE"));
    }

    #[test]
    fn possible_encodings_reports_gb18030() {
        // emoji 的 GB18030 4 字节扩展编码: 应报告 GB18030 而非 GBK
        let (hex, _) = encode_hex("😀", Charset::Gb18030);
        let (_, _, list) = decode_hex(&hex, Charset::Gb18030);
        assert!(list.contains(&"GB18030"), "list={list:?}");
        assert!(!list.contains(&"GBK"), "list={list:?}");

        // 纯 2 字节 GBK 数据: 应报告 GBK 而非 GB18030, 两者不冗余
        let (hex, _) = encode_hex("中文", Charset::Gb18030);
        let (_, _, list) = decode_hex(&hex, Charset::Gb18030);
        assert!(list.contains(&"GBK"), "list={list:?}");
        assert!(!list.contains(&"GB18030"), "list={list:?}");

        // U+20000 同为 4 字节扩展
        let (hex, _) = encode_hex("\u{20000}", Charset::Gb18030);
        let (_, _, list) = decode_hex(&hex, Charset::Gb18030);
        assert!(list.contains(&"GB18030"), "list={list:?}");
        assert!(!list.contains(&"GBK"), "list={list:?}");
    }

    #[test]
    fn decode_hex_reports_guess() {
        // UTF-8 中文 hex
        let (text, status, possible) = decode_hex("E4B8ADE69687", Charset::Utf8);
        assert_eq!(text, "中文");
        assert_eq!(status, Status::Idle);
        assert_eq!(possible.first(), Some(&"UTF-8"));
        // 出错时无猜测
        let (text, status, possible) = decode_hex("4G", Charset::Utf8);
        assert!(text.is_empty());
        assert_eq!(status, Status::Err(EncErr::NonHex));
        assert!(possible.is_empty());
        // 空输入无猜测
        let (_, _, possible) = decode_hex("", Charset::Utf8);
        assert!(possible.is_empty());
    }

    #[test]
    fn gb18030_roundtrip() {
        // 简体中文往返
        let (hex, status) = encode_hex("中文测试", Charset::Gb18030);
        assert_eq!(status, Status::Idle);
        let (text, status2, _) = decode_hex(&hex, Charset::Gb18030);
        assert_eq!(status2, Status::Idle);
        assert_eq!(text, "中文测试");

        // emoji: GBK 无法表示, GB18030 通过 4 字节扩展无损往返
        let (hex, status) = encode_hex("😀", Charset::Gb18030);
        assert_eq!(status, Status::Idle);
        let (text, status2, _) = decode_hex(&hex, Charset::Gb18030);
        assert_eq!(status2, Status::Idle);
        assert_eq!(text, "😀");

        // 同一字符在 GBK 下无法表示(警告), GB18030 无警告
        assert_eq!(encode_hex("😀", Charset::Gbk).1, Status::Warn("gbk"));
        // CJK 扩展 B 区字符 U+20000: GB18030 使用 4 字节扩展无损往返
        let (hex, status) = encode_hex("\u{20000}", Charset::Gb18030);
        assert_eq!(status, Status::Idle);
        assert_eq!(hex, "95328236");
        let (text, status2, _) = decode_hex(&hex, Charset::Gb18030);
        assert_eq!(status2, Status::Idle);
        assert_eq!(text, "\u{20000}");
    }

    #[test]
    fn gb18030_decodes_gbk_data() {
        // GB18030 解码器是 GBK 的超集: 可无损解码 GBK 编码的数据
        let (gbk_hex, _) = encode_hex("中文", Charset::Gbk);
        let (text, status, _) = decode_hex(&gbk_hex, Charset::Gb18030);
        assert_eq!(status, Status::Idle);
        assert_eq!(text, "中文");
        // \u 转义仍仅限 UTF-16, GB18030 与 GBK 一致
        assert_eq!(hex_to_bytes(r"\u4F60", Charset::Gb18030), Err(EncErr::EscapeUtf16Only));
    }

    #[test]
    fn big5_roundtrip() {
        // 繁体中文往返
        let (hex, status) = encode_hex("中文測試", Charset::Big5);
        assert_eq!(status, Status::Idle);
        let (text, status2, _) = decode_hex(&hex, Charset::Big5);
        assert_eq!(status2, Status::Idle);
        assert_eq!(text, "中文測試");

        // emoji 无法用 Big5 表示(警告)
        assert_eq!(encode_hex("😀", Charset::Big5).1, Status::Warn("big5"));

        // \u 转义仍仅限 UTF-16
        assert_eq!(hex_to_bytes(r"\u4E2D", Charset::Big5), Err(EncErr::EscapeUtf16Only));
    }

    #[test]
    fn possible_encodings_detects_utf16_without_bom() {
        // 无 BOM UTF-16LE 的 ASCII 文本: NUL 字节占比高, 应识别为 UTF-16LE
        let le = "hello".encode_utf16().flat_map(u16::to_le_bytes).collect::<Vec<u8>>();
        let list = possible_encodings(&le);
        assert!(list.contains(&"UTF-16LE"), "list={list:?}");
        assert!(!list.contains(&"UTF-16BE"), "list={list:?}");

        // 无 BOM UTF-16BE 的 ASCII 文本: 应识别为 UTF-16BE
        let be = "hello".encode_utf16().flat_map(u16::to_be_bytes).collect::<Vec<u8>>();
        let list = possible_encodings(&be);
        assert!(list.contains(&"UTF-16BE"), "list={list:?}");
        assert!(!list.contains(&"UTF-16LE"), "list={list:?}");

        // 带 BOM 的 UTF-16LE 走 BOM 识别分支, 同样正确
        let mut bom = vec![0xFF, 0xFE];
        bom.extend_from_slice(&le);
        let list = possible_encodings(&bom);
        assert!(list.contains(&"UTF-16LE"), "list={list:?}");

        // 纯中文无 BOM UTF-16LE: NUL 占比过低, 不强行猜测(检测的根本盲区)
        let cn = "中文".encode_utf16().flat_map(u16::to_le_bytes).collect::<Vec<u8>>();
        let list = possible_encodings(&cn);
        assert!(!list.contains(&"UTF-16LE"), "list={list:?}");
        assert!(!list.contains(&"UTF-16BE"), "list={list:?}");
    }
}
