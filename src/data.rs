use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};

use crate::config::Config;
use crate::hunspell::Hunspell;
use crate::SpellError;
use curl::easy::Easy;
use encoding_rs::{Encoding, UTF_8};

// TODO log steps

struct FileProvider<'a> {
    file_path: &'a Path,
    cache_path: &'a Path,
    directories: &'a [String],
    url: Option<&'a str>,
}

impl<'a> FileProvider<'a> {
    fn fetch(&self) -> Result<(), SpellError> {
        match self.url {
            Some(url) => {
                log::debug!("downloading file from {}", url);
                // TODO error handling
                fs::create_dir_all(&self.cache_path.parent().unwrap())
                    .map_err(SpellError::FileCachingError)?;
                let mut dst =
                    File::create(&self.cache_path).map_err(SpellError::FileCachingError)?;
                let mut easy = Easy::new();
                easy.url(url).unwrap();
                let mut transfer = easy.transfer();
                transfer
                    .write_function(|data| {
                        dst.write(data).unwrap();
                        Ok(data.len())
                    })
                    .unwrap();
                transfer.perform().unwrap();
                Ok(())
            }
            None => Err(SpellError::NoDicSource),
        }
    }

    fn find(&self) -> Option<PathBuf> {
        for dir in self.directories {
            let target = PathBuf::from(&dir).join(self.file_path.file_name().unwrap());
            if target.exists() {
                return Some(target);
            }
        }
        None
    }

    fn detect_encoding<P: AsRef<Path>>(path: P) -> io::Result<String> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let mut buf = Vec::new();
        while reader.read_until(0x0A, &mut buf)? > 0 {
            if buf.starts_with(b"SET ") {
                let s = String::from_utf8(buf[4..].to_vec()).unwrap();
                return Ok(String::from(s.trim_end()));
            }
            buf.clear();
        }
        Ok(String::from("utf-8"))
    }

    fn read_raw_source(&self) -> io::Result<Vec<u8>> {
        let mut raw = Vec::new();
        File::open(&self.cache_path)?.read_to_end(&mut raw)?;
        Ok(raw)
    }

    fn convert(&self) -> Result<(), SpellError> {
        if !self.cache_path.exists() {
            if let Some(src) = self.find() {
                log::debug!("found source file: {}", src.display());
                fs::create_dir_all(&self.cache_path.parent().unwrap())
                    .map_err(SpellError::FileCachingError)?;
                fs::copy(src, self.cache_path).map_err(SpellError::FileCachingError)?;
            }
        }
        if !self.cache_path.exists() {
            self.fetch()?;
        }
        let encoding =
            Self::detect_encoding(&self.cache_path).map_err(SpellError::ConversionError)?;
        log::debug!("detected encoding: {}", encoding);
        let decoder = Encoding::for_label(encoding.as_bytes()).unwrap();
        log::debug!("using decoder: {}", decoder.name());
        let raw = self
            .read_raw_source()
            .map_err(SpellError::ConversionError)?;
        fs::create_dir_all(&self.file_path.parent().unwrap())
            .map_err(SpellError::FileCachingError)?;
        if decoder == UTF_8 {
            fs::write(&self.file_path, raw).map_err(SpellError::ConversionError)?;
        } else {
            let (content, _, _) = decoder.decode(&raw);
            let content_tweaked = content.replace(&format!("SET {}", encoding), "SET UTF-8");
            fs::write(&self.file_path, content_tweaked.as_bytes())
                .map_err(SpellError::ConversionError)?;
        }
        Ok(())
    }

    fn ensure(&self) -> Result<(), SpellError> {
        if !self.file_path.exists() {
            self.convert()?;
            fs::remove_file(&self.cache_path).map_err(SpellError::FileCachingError)?;
        }
        log::debug!("file available at {}", self.file_path.display());
        Ok(())
    }
}

/// A structure to ensure that the data files for the given language are available.
///
/// It can look for files on the disk or download directly from internet when possible.
pub struct LangProvider<'a> {
    lang: &'a str,
    config: &'a Config,
    aff_path: PathBuf,
    dic_path: PathBuf,
}

impl LangProvider<'_> {
    /// Creates a new `LanguageProvider` for the given language and configuration.
    ///
    /// The configuration can be loaded using [load_config()](fn.load_config.html).
    pub fn new<'a>(lang: &'a str, config: &'a Config) -> LangProvider<'a> {
        let dics_dir = crate::dirs().data_local_dir().join("dictionaries");
        LangProvider {
            lang,
            config,
            aff_path: dics_dir.join(&format!("{}.aff", lang)),
            dic_path: dics_dir.join(&format!("{}.dic", lang)),
        }
    }

    /// Location of the `.aff` dictionary.
    pub fn aff(&self) -> &Path {
        &self.aff_path
    }

    fn on_disk(&self, filename: &std::ffi::OsStr) -> Vec<PathBuf> {
        self.config
            .dictionaries
            .directories
            .iter()
            .map(|d| PathBuf::from(d).join(filename))
            .filter(|p| p.exists())
            .collect()
    }

    /// Search for corresponding `.aff` dictionaries on the disk.
    pub fn aff_on_disk(&self) -> Vec<PathBuf> {
        self.on_disk(&self.aff_path.file_name().unwrap())
    }

    /// Location of the `.dic` dictionary.
    pub fn dic(&self) -> &Path {
        &self.dic_path
    }

    /// Search for corresponding `.dic` dictionaries on the disk.
    pub fn dic_on_disk(&self) -> Vec<PathBuf> {
        self.on_disk(&self.dic_path.file_name().unwrap())
    }

    /// Ensures that the dictionaries are present in the dictionaries directory.
    ///
    /// If one of the dictionaries is absent it will be fetched from the disk or
    /// directly from internet when possible.
    pub fn ensure_data(&self) -> Result<(), SpellError> {
        let sources = self.config.dictionaries.sources.get(self.lang);
        let cache_dir = crate::dirs().cache_dir().to_owned();
        FileProvider {
            file_path: &self.aff_path,
            cache_path: &cache_dir.join(self.aff_path.file_name().unwrap()),
            directories: &self.config.dictionaries.directories,
            url: sources.map(|s| s.aff.as_str()),
        }
        .ensure()?;
        FileProvider {
            file_path: &self.dic_path,
            cache_path: &cache_dir.join(self.dic_path.file_name().unwrap()),
            directories: &self.config.dictionaries.directories,
            url: sources.map(|s| s.dic.as_str()),
        }
        .ensure()
    }

    /// Removes the dictionaries from the dictionaries directory.
    pub fn remove_data(&self) -> Result<(), SpellError> {
        if self.aff_path.exists() {
            fs::remove_file(&self.aff_path).map_err(SpellError::RemoveDicError)?;
        }
        if self.dic_path.exists() {
            fs::remove_file(&self.dic_path).map_err(SpellError::RemoveDicError)?;
        }
        Ok(())
    }
}

impl Into<Hunspell> for LangProvider<'_> {
    fn into(self) -> Hunspell {
        Hunspell::new(self.aff_path, self.dic_path)
    }
}
