//! 哈希计算工具
//!
//! 支持 MD5 / SHA-1 / SHA-224 / SHA-256 / SHA-384 / SHA-512
//! 与 BLAKE2b-512 / BLAKE2s-256 / BLAKE3,
//! 输入文本(UTF-8) 即时输出十六进制摘要。

use eframe::egui;

use crate::i18n::Texts;
use crate::theme;

/// 哈希算法
#[derive(Clone, Copy, PartialEq, Eq)]
enum HashAlgo {
    Md5,
    Sha1,
    Sha224,
    Sha256,
    Sha384,
    Sha512,
    Blake2b512,
    Blake2s256,
    Blake3,
}

impl HashAlgo {
    /// 所有算法(UI 展示顺序)
    const ALL: [HashAlgo; 9] = [
        HashAlgo::Md5,
        HashAlgo::Sha1,
        HashAlgo::Sha224,
        HashAlgo::Sha256,
        HashAlgo::Sha384,
        HashAlgo::Sha512,
        HashAlgo::Blake2b512,
        HashAlgo::Blake2s256,
        HashAlgo::Blake3,
    ];

    /// 算法显示名
    fn name(self) -> &'static str {
        return match self {
            HashAlgo::Md5 => "MD5",
            HashAlgo::Sha1 => "SHA-1",
            HashAlgo::Sha224 => "SHA-224",
            HashAlgo::Sha256 => "SHA-256",
            HashAlgo::Sha384 => "SHA-384",
            HashAlgo::Sha512 => "SHA-512",
            HashAlgo::Blake2b512 => "BLAKE2b-512",
            HashAlgo::Blake2s256 => "BLAKE2s-256",
            HashAlgo::Blake3 => "BLAKE3",
        };
    }

    /// 计算输入字节的十六进制摘要
    fn digest(self, bytes: &[u8]) -> String {
        use sha2::Digest;
        let out: Vec<u8> = match self {
            HashAlgo::Md5 => md5::Md5::digest(bytes).to_vec(),
            HashAlgo::Sha1 => sha1::Sha1::digest(bytes).to_vec(),
            HashAlgo::Sha224 => sha2::Sha224::digest(bytes).to_vec(),
            HashAlgo::Sha256 => sha2::Sha256::digest(bytes).to_vec(),
            HashAlgo::Sha384 => sha2::Sha384::digest(bytes).to_vec(),
            HashAlgo::Sha512 => sha2::Sha512::digest(bytes).to_vec(),
            HashAlgo::Blake2b512 => blake2::Blake2b512::digest(bytes).to_vec(),
            HashAlgo::Blake2s256 => blake2::Blake2s256::digest(bytes).to_vec(),
            HashAlgo::Blake3 => blake3::hash(bytes).as_bytes().to_vec(),
        };
        return to_hex(&out);
    }
}

/// 哈希计算工具界面状态
pub struct HashTool {
    t: Texts,

    algo: HashAlgo,
    input: String,
    output: String,
    last_key: (HashAlgo, String),
}

impl HashTool {
    pub fn new(t: Texts) -> Self {
        return Self {
            t,
            algo: HashAlgo::Md5,
            input: String::new(),
            output: String::new(),
            last_key: (HashAlgo::Md5, String::new()),
        };
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
            let names: Vec<&str> = HashAlgo::ALL.iter().map(|a| a.name()).collect();
            let selected = HashAlgo::ALL.iter().position(|&a| a == self.algo).unwrap_or(0);
            if let Some(i) = theme::chip_group(ui, &t.hash_algorithm, selected, &names) {
                self.algo = HashAlgo::ALL[i];
            }
        });
        theme::card_gap(ui);

        theme::card(ui, |ui| {
            theme::field_label(ui, &t.gen_input);
            theme::text_area(ui, &mut self.input, &t.hash_hint, 3, false);

            ui.add_space(6.0);
            theme::output_header(ui, &t.gen_output, &t.gen_copy, &self.output, |_| {});
            theme::text_area(ui, &mut self.output, "", 2, true);
        });
    }

    /// 输入或算法变化时重新计算哈希
    fn update(&mut self) {
        let key = (self.algo, self.input.clone());
        if self.last_key == key {
            return;
        }
        self.last_key = key;
        self.output = if self.input.is_empty() {
            String::new()
        } else {
            self.algo.digest(self.input.as_bytes())
        };
    }
}

/// 字节数组转为小写十六进制字符串
fn to_hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{:02x}", b));
    }
    return s;
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 使用标准测试向量验证各算法输出
    #[test]
    fn known_vectors() {
        assert_eq!(
            HashAlgo::Md5.digest(b""),
            "d41d8cd98f00b204e9800998ecf8427e"
        );
        assert_eq!(
            HashAlgo::Sha1.digest(b""),
            "da39a3ee5e6b4b0d3255bfef95601890afd80709"
        );
        assert_eq!(
            HashAlgo::Sha224.digest(b""),
            "d14a028c2a3a2bc9476102bb288234c415a2b01f828ea62ac5b3e42f"
        );
        assert_eq!(
            HashAlgo::Sha256.digest(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_eq!(
            HashAlgo::Sha384.digest(b"abc"),
            "cb00753f45a35e8bb5a03d699ac65007272c32ab0eded1631a8b605a43ff5bed\
             8086072ba1e7cc2358baeca134c825a7"
        );
        assert_eq!(
            HashAlgo::Sha512.digest(b"abc"),
            "ddaf35a193617abacc417349ae20413112e6fa4e89a97ea20a9eeee64b55d39a\
             2192992a274fc1a836ba3c23a3feebbd454d4423643ce80e2a9ac94fa54ca49f"
        );
        // BLAKE2b-512 标准测试向量
        assert_eq!(
            HashAlgo::Blake2b512.digest(b""),
            "786a02f742015903c6c6fd852552d272912f4740e15847618a86e217f71f5419\
             d25e1031afee585313896444934eb04b903a685b1448b755d56f701afe9be2ce"
        );
        assert_eq!(
            HashAlgo::Blake2b512.digest(b"abc"),
            "ba80a53f981c4d0d6a2797b69f12f6e94c212f14685ac4b74b12bb6fdbffa2d1\
             7d87c5392aab792dc252d5de4533cc9518d38aa8dbf1925ab92386edd4009923"
        );
        // BLAKE2s-256 标准测试向量
        assert_eq!(
            HashAlgo::Blake2s256.digest(b""),
            "69217a3079908094e11121d042354a7c1f55b6482ca1a51e1b250dfd1ed0eef9"
        );
        assert_eq!(
            HashAlgo::Blake2s256.digest(b"abc"),
            "508c5e8c327c14e2e1a72ba34eeb452f37458b209ed63a294d999b4c86675982"
        );
        // BLAKE3 标准测试向量
        assert_eq!(
            HashAlgo::Blake3.digest(b""),
            "af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262"
        );
        assert_eq!(
            HashAlgo::Blake3.digest(b"abc"),
            "6437b3ac38465133ffb63b75273a8db548c558465d79db03fd359c6cd5bd9d85"
        );
    }
}
