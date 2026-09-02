//! 校验计算工具
//!
//! 支持常用通信校验算法:
//! - CRC 系列: CRC-8 (SMBUS) / CRC-16 (IBM) / CRC-16 (MODBUS) / CRC-32 (IEEE) / CRC-32C (ISCSI) / CRC-64 (ECMA)
//! - 简单校验: BCC (字节异或) / LRC (Modbus 纵向冗余校验) / SUM-8 (字节累加)
//! - HMAC 消息认证码: HMAC-MD5 / HMAC-SHA1 / HMAC-SHA256 / HMAC-SHA512 (需密钥)

use eframe::egui;

use crate::i18n::Texts;
use crate::theme;

/// 基于给定哈希算法计算 HMAC 十六进制结果
/// (HMAC 支持任意长度密钥; 使用宏避免为每种哈希编写泛型边界)
macro_rules! hmac_compute {
    ($hash:ty, $bytes:expr, $key:expr) => {{
        use hmac::{Hmac, Mac};
        let mut mac = Hmac::<$hash>::new_from_slice($key).expect("HMAC 密钥");
        mac.update($bytes);
        to_hex(&mac.finalize().into_bytes())
    }};
}

/// 校验算法
#[derive(Clone, Copy, PartialEq, Eq)]
enum ChecksumAlgo {
    Crc8Smbus,
    Crc16Ibm,
    Crc16Modbus,
    Crc32,
    Crc32C,
    Crc64Ecma,
    Bcc,
    Lrc,
    Sum8,
    HmacMd5,
    HmacSha1,
    HmacSha256,
    HmacSha512,
}

impl ChecksumAlgo {
    /// 所有算法(UI 展示顺序)
    const ALL: [ChecksumAlgo; 13] = [
        ChecksumAlgo::Crc8Smbus,
        ChecksumAlgo::Crc16Ibm,
        ChecksumAlgo::Crc16Modbus,
        ChecksumAlgo::Crc32,
        ChecksumAlgo::Crc32C,
        ChecksumAlgo::Crc64Ecma,
        ChecksumAlgo::Bcc,
        ChecksumAlgo::Lrc,
        ChecksumAlgo::Sum8,
        ChecksumAlgo::HmacMd5,
        ChecksumAlgo::HmacSha1,
        ChecksumAlgo::HmacSha256,
        ChecksumAlgo::HmacSha512,
    ];

    /// 算法显示名
    fn name(self) -> &'static str {
        match self {
            ChecksumAlgo::Crc8Smbus => "CRC-8 (SMBUS)",
            ChecksumAlgo::Crc16Ibm => "CRC-16 (IBM)",
            ChecksumAlgo::Crc16Modbus => "CRC-16 (MODBUS)",
            ChecksumAlgo::Crc32 => "CRC-32 (IEEE)",
            ChecksumAlgo::Crc32C => "CRC-32C (ISCSI)",
            ChecksumAlgo::Crc64Ecma => "CRC-64 (ECMA)",
            ChecksumAlgo::Bcc => "BCC (XOR)",
            ChecksumAlgo::Lrc => "LRC (Modbus)",
            ChecksumAlgo::Sum8 => "SUM-8",
            ChecksumAlgo::HmacMd5 => "HMAC-MD5",
            ChecksumAlgo::HmacSha1 => "HMAC-SHA1",
            ChecksumAlgo::HmacSha256 => "HMAC-SHA256",
            ChecksumAlgo::HmacSha512 => "HMAC-SHA512",
        }
    }

    /// 是否为 HMAC 算法(需要密钥输入)
    fn is_hmac(self) -> bool {
        matches!(
            self,
            ChecksumAlgo::HmacMd5
                | ChecksumAlgo::HmacSha1
                | ChecksumAlgo::HmacSha256
                | ChecksumAlgo::HmacSha512
        )
    }

    /// 计算输入字节的校验结果(小写十六进制)
    fn compute(self, bytes: &[u8], key: &[u8]) -> String {
        match self {
            // CRC 系列
            ChecksumAlgo::Crc8Smbus => to_hex(&crc::Crc::<u8>::new(&crc::CRC_8_SMBUS).checksum(bytes).to_be_bytes()),
            ChecksumAlgo::Crc16Ibm => to_hex(&crc::Crc::<u16>::new(&crc::CRC_16_ARC).checksum(bytes).to_be_bytes()),
            ChecksumAlgo::Crc16Modbus => to_hex(&crc::Crc::<u16>::new(&crc::CRC_16_MODBUS).checksum(bytes).to_be_bytes()),
            ChecksumAlgo::Crc32 => to_hex(&crc::Crc::<u32>::new(&crc::CRC_32_ISO_HDLC).checksum(bytes).to_be_bytes()),
            ChecksumAlgo::Crc32C => to_hex(&crc::Crc::<u32>::new(&crc::CRC_32_ISCSI).checksum(bytes).to_be_bytes()),
            ChecksumAlgo::Crc64Ecma => to_hex(&crc::Crc::<u64>::new(&crc::CRC_64_ECMA_182).checksum(bytes).to_be_bytes()),
            // 简单校验
            ChecksumAlgo::Bcc => to_hex(&[bytes.iter().fold(0u8, |acc, b| acc ^ b)]),
            ChecksumAlgo::Lrc => {
                let sum = bytes.iter().fold(0u8, |acc, b| acc.wrapping_add(*b));
                let lrc = (0x100u16 - sum as u16) & 0xFF;
                to_hex(&[lrc as u8])
            }
            ChecksumAlgo::Sum8 => to_hex(&[bytes.iter().fold(0u8, |acc, b| acc.wrapping_add(*b))]),
            // HMAC 系列
            ChecksumAlgo::HmacMd5 => hmac_compute!(md5::Md5, bytes, key),
            ChecksumAlgo::HmacSha1 => hmac_compute!(sha1::Sha1, bytes, key),
            ChecksumAlgo::HmacSha256 => hmac_compute!(sha2::Sha256, bytes, key),
            ChecksumAlgo::HmacSha512 => hmac_compute!(sha2::Sha512, bytes, key),
        }
    }
}

/// 校验计算工具界面状态
pub struct ChecksumTool {
    t: Texts,

    algo: ChecksumAlgo,
    input: String,
    key: String,
    output: String,
    last_key: (ChecksumAlgo, String, String),
}

impl ChecksumTool {
    pub fn new(t: Texts) -> Self {
        Self {
            t,
            algo: ChecksumAlgo::Crc32,
            input: String::new(),
            key: String::new(),
            output: String::new(),
            last_key: (ChecksumAlgo::Crc32, String::new(), String::new()),
        }
    }

    /// 语言切换时更新文案
    pub fn set_lang(&mut self, t: Texts) {
        self.t = t;
    }

    /// 渲染工具界面
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        let t = self.t.clone();
        self.update();

        theme::card(ui, |ui| {
            let names: Vec<&str> = ChecksumAlgo::ALL.iter().map(|a| a.name()).collect();
            let selected = ChecksumAlgo::ALL
                .iter()
                .position(|&a| a == self.algo)
                .unwrap_or(0);
            if let Some(i) = theme::chip_group(ui, &t.checksum_algorithm, selected, &names) {
                self.algo = ChecksumAlgo::ALL[i];
            }
        });
        theme::card_gap(ui);

        theme::card(ui, |ui| {
            theme::field_label(ui, &t.gen_input);
            theme::text_area(ui, &mut self.input, &t.checksum_hint, 3, false);

            // HMAC 算法需要密钥输入
            if self.algo.is_hmac() {
                ui.add_space(6.0);
                theme::field_label(ui, &t.checksum_key);
                theme::text_input(ui, &mut self.key, &t.checksum_key, f32::INFINITY);
            }

            ui.add_space(6.0);
            theme::output_header(ui, &t.gen_output, &t.gen_copy, &self.output, |_| {});
            theme::text_area(ui, &mut self.output, "", 2, true);
        });
    }

    /// 输入、密钥或算法变化时重新计算校验值
    fn update(&mut self) {
        let key = (self.algo, self.input.clone(), self.key.clone());
        if self.last_key == key {
            return;
        }
        self.last_key = key;
        self.output = if self.input.is_empty() {
            String::new()
        } else {
            self.algo.compute(self.input.as_bytes(), self.key.as_bytes())
        };
    }
}

/// 字节数组转为小写十六进制字符串
fn to_hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{:02x}", b));
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 使用标准检查值("123456789")与 RFC 4231/2202 测试向量验证
    #[test]
    fn known_vectors() {
        let data = b"123456789";
        assert_eq!(ChecksumAlgo::Crc8Smbus.compute(data, b""), "f4");
        assert_eq!(ChecksumAlgo::Crc16Ibm.compute(data, b""), "bb3d");
        assert_eq!(ChecksumAlgo::Crc16Modbus.compute(data, b""), "4b37");
        assert_eq!(ChecksumAlgo::Crc32.compute(data, b""), "cbf43926");
        assert_eq!(ChecksumAlgo::Crc32C.compute(data, b""), "e3069283");
        assert_eq!(ChecksumAlgo::Crc64Ecma.compute(data, b""), "6c40df5f0b497347");
        assert_eq!(ChecksumAlgo::Bcc.compute(data, b""), "31");
        assert_eq!(ChecksumAlgo::Lrc.compute(data, b""), "23");
        assert_eq!(ChecksumAlgo::Sum8.compute(data, b""), "dd");
    }

    /// RFC 2202 / RFC 4231 "Hi There" 测试向量
    ///
    /// RFC 2202: HMAC-MD5 用 key=0x0b x16, HMAC-SHA-1 用 key=0x0b x20
    /// RFC 4231: HMAC-SHA-256/512 用 key=0x0b x20
    #[test]
    fn hmac_vectors() {
        let data = b"Hi There";

        // HMAC-MD5 (RFC 2202, key=0x0b x16)
        let key16 = [0x0b; 16];
        assert_eq!(
            ChecksumAlgo::HmacMd5.compute(data, &key16),
            "9294727a3638bb1c13f48ef8158bfc9d"
        );

        // HMAC-SHA-1 (RFC 2202, key=0x0b x20)
        let key20 = [0x0b; 20];
        assert_eq!(
            ChecksumAlgo::HmacSha1.compute(data, &key20),
            "b617318655057264e28bc0b6fb378c8ef146be00"
        );

        // HMAC-SHA-256 / HMAC-SHA-512 (RFC 4231, key=0x0b x20)
        assert_eq!(
            ChecksumAlgo::HmacSha256.compute(data, &key20),
            "b0344c61d8db38535ca8afceaf0bf12b881dc200c9833da726e9376c2e32cff7"
        );
        assert_eq!(
            ChecksumAlgo::HmacSha512.compute(data, &key20),
            "87aa7cdea5ef619d4ff0b4241a1d6cb02379f4e2ce4ec2787ad0b30545e17cde\
             daa833b7d6b8a702038b274eaea3f4e4be9d914eeb61f1702e696c203a126854"
        );
    }
}
