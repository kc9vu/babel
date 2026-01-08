use std::fs;
use std::path::Path;

use charset_normalizer_rs::from_bytes;
use encoding_rs::*;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("extern crate '{0}': {1}")]
    Extern(&'static str, String),

    #[error("unresolved encoding '{0}'")]
    UnresolvedEncoding(String),

    #[error("unknown encoding")]
    UnknownEncoding,

    #[error("File decoding error using {0}")]
    FailedDecoding(String),
}

/// 检测文本的编码，因为首先读取字节，会占用双倍内存
pub fn read_to_utf8(path: impl AsRef<Path>) -> Result<String> {
    let bytes = fs::read(path).map_err(|er| Error::Extern("io", er.to_string()))?;

    let result = from_bytes(&bytes, None)
        .map_err(|er| Error::Extern("charset_normalizer_rs", er.to_string()))?;
    let best_guess = result.get_best().ok_or(Error::Extern(
        "charset_normalizer_rs",
        "No best charset".to_string(),
    ))?;
    let best_guess_encoding = best_guess.encoding();
    let (cow, _encoding_used, had_errors) = match best_guess_encoding {
        "big5" => BIG5.decode(&bytes),
        "euc-jp" => EUC_JP.decode(&bytes),
        "euc-kr" => EUC_KR.decode(&bytes),
        "gb18030" => GB18030.decode(&bytes),
        "gbk" => GBK.decode(&bytes),
        "utf-8" | "ascii" => UTF_8.decode(&bytes),
        "utf-16be" => UTF_16BE.decode(&bytes),
        "utf-16le" => UTF_16LE.decode(&bytes),
        encoding => return Err(Error::UnresolvedEncoding(encoding.to_string())),
    };

    if had_errors {
        Err(Error::FailedDecoding(best_guess_encoding.to_string()))
    } else {
        Ok(cow.into_owned())
    }
}
