use std::env::current_dir;

fn main() {
    let basedir = current_dir()
        .expect("get cwd")
        .join("hunspell")
        .join("src")
        .join("hunspell");

    cc::Build::new()
        .cpp(true)
        .file(basedir.join("affentry.cxx"))
        .file(basedir.join("affixmgr.cxx"))
        .file(basedir.join("csutil.cxx"))
        .file(basedir.join("filemgr.cxx"))
        .file(basedir.join("hashmgr.cxx"))
        .file(basedir.join("hunspell.cxx"))
        .file(basedir.join("hunzip.cxx"))
        .file(basedir.join("phonet.cxx"))
        .file(basedir.join("replist.cxx"))
        .file(basedir.join("suggestmgr.cxx"))
        .define("BUILDING_LIBHUNSPELL", None)
        .compile("hunspell");
}
