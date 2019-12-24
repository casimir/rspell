//! Safe minimal wrapper of hunspell.

use std::ffi::CStr;
use std::os::raw::{c_char, c_int};
use std::path::Path;

enum Hunhandle {}

extern "C" {
    fn Hunspell_create(affpath: *const c_char, dpath: *const c_char) -> *mut Hunhandle;
    fn Hunspell_create_key(
        affpath: *const c_char,
        dpath: *const c_char,
        key: *const c_char,
    ) -> *mut Hunhandle;
    fn Hunspell_destroy(pHunspell: *mut Hunhandle);
    // fn Hunspell_add_dic(pHunspell: *mut Hunhandle, dpath: *const c_char) -> c_int;
    fn Hunspell_spell(pHunspell: *mut Hunhandle, word: *const c_char) -> c_int;
    // fn Hunspell_get_dic_encoding(pHunspell: *mut Hunhandle) -> *mut c_char;
    fn Hunspell_suggest(
        pHunspell: *mut Hunhandle,
        slst: *mut *mut *mut c_char,
        word: *const c_char,
    ) -> c_int;

    // fn Hunspell_analyze(
    //     pHunspell: *mut Hunhandle,
    //     slst: *mut *mut *mut c_char,
    //     word: *const c_char,
    // ) -> c_int;
    // fn Hunspell_stem(
    //     pHunspell: *mut Hunhandle,
    //     slst: *mut *mut *mut c_char,
    //     word: *const c_char,
    // ) -> c_int;
    // fn Hunspell_stem2(
    //     pHunspell: *mut Hunhandle,
    //     slst: *mut *mut *mut c_char,
    //     desc: *mut *mut c_char,
    //     n: c_int,
    // ) -> c_int;
    // fn Hunspell_generate(
    //     pHunspell: *mut Hunhandle,
    //     slst: *mut *mut *mut c_char,
    //     word: *const c_char,
    //     word2: *const c_char,
    // ) -> c_int;
    // fn Hunspell_generate2(
    //     pHunspell: *mut Hunhandle,
    //     slst: *mut *mut *mut c_char,
    //     word: *const c_char,
    //     desc: *mut *mut c_char,
    //     n: c_int,
    // ) -> c_int;

    fn Hunspell_add(pHunspell: *mut Hunhandle, word: *const c_char) -> c_int;
    fn Hunspell_add_with_affix(
        pHunspell: *mut Hunhandle,
        word: *const c_char,
        example: *const c_char,
    ) -> c_int;
    fn Hunspell_remove(pHunspell: *mut Hunhandle, word: *const c_char) -> c_int;

    fn Hunspell_free_list(pHunspell: *mut Hunhandle, slst: *mut *mut *mut c_char, n: c_int);
}

macro_rules! cstring {
    ($e:expr) => {
        std::ffi::CString::new($e).expect("build string")
    };
}

macro_rules! path_to_cstring {
    ($p:expr) => {
        cstring!($p.to_str().expect("build &str"))
    };
}

/// Wrapper for hunspell's handle.
pub struct Hunspell {
    handle: *mut Hunhandle,
}

impl Hunspell {
    /// Creates a new handle.
    pub fn new<P: AsRef<Path>>(affix: P, dictionnary: P) -> Hunspell {
        log::debug!("aff file: {}", affix.as_ref().display());
        log::debug!("dic file: {}", dictionnary.as_ref().display());
        log::trace!("create(...)");
        unsafe {
            Hunspell {
                handle: Hunspell_create(
                    path_to_cstring!(affix.as_ref()).as_ptr(),
                    path_to_cstring!(dictionnary.as_ref()).as_ptr(),
                ),
            }
        }
    }

    /// Creates a new handle with key.
    pub fn with_key<P: AsRef<Path>>(affix: P, dictionnary: P, key: &str) -> Hunspell {
        log::debug!("aff file: {}", affix.as_ref().display());
        log::debug!("dic file: {}", dictionnary.as_ref().display());
        log::trace!("create_with_key(...)");
        unsafe {
            Hunspell {
                handle: Hunspell_create_key(
                    path_to_cstring!(affix.as_ref()).as_ptr(),
                    path_to_cstring!(dictionnary.as_ref()).as_ptr(),
                    cstring!(key).as_ptr(),
                ),
            }
        }
    }

    /// Spellchecks the given word.
    pub fn spell(&self, word: &str) -> bool {
        log::trace!("spell({:?})", word);
        unsafe { Hunspell_spell(self.handle, cstring!(word).as_ptr()) != 0 }
    }

    fn consume_slst(&self, mut slst: *mut *mut c_char, n: c_int) -> Vec<String> {
        let mut values = Vec::new();
        unsafe {
            for i in 0..n {
                let it = *slst.offset(i as isize);
                let cs = CStr::from_ptr(it);
                match cs.to_str() {
                    Ok(s) => {
                        values.push(s.to_owned());
                    }
                    Err(_) => {
                        log::error!(
                            "skipped suggestion: invalid utf-8: {:?}",
                            cs.to_string_lossy()
                        );
                    }
                };
            }
            log::trace!("free_list(...)");
            Hunspell_free_list(self.handle, &mut slst, n);
        }
        values
    }

    /// Suggests possible corrections for the given word.
    pub fn suggest(&self, word: &str) -> Vec<String> {
        log::trace!("suggest({:?})", word);
        unsafe {
            let mut slst = std::ptr::null_mut();
            let n = Hunspell_suggest(self.handle, &mut slst, cstring!(word).as_ptr());
            self.consume_slst(slst, n)
        }
    }

    /// Add a word to the in-memory dictionary.
    pub fn add(&self, word: &str) -> bool {
        log::trace!("add({:?})", word);
        unsafe { Hunspell_add(self.handle, cstring!(word).as_ptr()) == 0 }
    }

    /// Add a word to the in-memory dictionary with the affix of `example`.
    pub fn add_with_affix(&self, word: &str, example: &str) -> bool {
        log::trace!("add_with_affix({:?}, {:?})", word, example);
        unsafe {
            Hunspell_add_with_affix(
                self.handle,
                cstring!(word).as_ptr(),
                cstring!(example).as_ptr(),
            ) == 0
        }
    }

    /// Remove a word from the in-memory dictionary.
    pub fn remove(&self, word: &str) -> bool {
        log::trace!("remove({:?})", word);
        unsafe { Hunspell_remove(self.handle, cstring!(word).as_ptr()) == 0 }
    }
}

impl Drop for Hunspell {
    fn drop(&mut self) {
        log::trace!("destroy()");
        unsafe {
            Hunspell_destroy(self.handle);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spell() {
        let h = Hunspell::new("files/fr.aff", "files/fr.dic");
        assert!(h.spell("coucou"));
        assert!(!h.spell("cocou"));
    }

    #[test]
    fn suggest() {
        let h = Hunspell::new("files/fr.aff", "files/fr.dic");
        let suggs = h.suggest("francais");
        assert_eq!(suggs.get(0), Some(&"français".to_string()));
    }

    #[test]
    fn add_remove() {
        let h = Hunspell::new("files/fr.aff", "files/fr.dic");
        let not_a_word = "fussoire";

        assert!(!h.spell(not_a_word));

        assert!(h.add(not_a_word));
        assert!(h.spell(not_a_word));

        assert!(h.remove(not_a_word));
        assert!(!h.spell(not_a_word));
    }
}
