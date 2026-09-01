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
    Utf16Le,
    Utf16Be,
}

impl Charset {
    fn name(self) -> &'static str {
        match self {
            Charset::Utf8 => "UTF-8",
            Charset::Gbk => "GBK",
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

        let (text, status) = decode_hex(&self.hex_input, self.charset);
        self.decoded = text;
        self.decode_status = status;
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
fn decode_hex(input: &str, charset: Charset) -> (String, Status) {
    let bytes = match hex_to_bytes(input, charset) {
        Ok(bytes) => bytes,
        Err(e) => return (String::new(), Status::Err(e)),
    };
    if bytes.is_empty() {
        return (String::new(), Status::Idle);
    }

    match charset {
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
    }
}

/// 按指定字符集将文本编码为 hex 字符串
fn encode_hex(text: &str, charset: Charset) -> (String, Status) {
    let (bytes, warning) = match charset {
        Charset::Utf8 => (text.as_bytes().to_vec(), None),
        Charset::Gbk => {
            let (bytes, _, had_errors) = encoding_rs::GBK.encode(text);
            (bytes.into_owned(), had_errors.then_some("gbk"))
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
}
