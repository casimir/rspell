use rspell::{LangProvider, SpellError};
use structopt::StructOpt;

#[derive(StructOpt)]
enum Cli {
    /// Prints information about the given language
    Info { lang: String },
    /// Ensures the files for the given language are available
    Ensure { lang: String },
    /// Removes the files for the given language
    Remove { lang: String },
}

fn main() -> Result<(), SpellError> {
    env_logger::init();
    let config = rspell::load_config()?;
    match Cli::from_args() {
        Cli::Info { lang } => {
            let provider = LangProvider::new(&lang, &config);
            let source = config.dictionaries.sources.get(&lang);
            println!("language code     : {}", lang);
            println!("aff file exists   : {}", provider.aff().exists());
            println!("dic file exists   : {}", provider.dic().exists());
            println!(
                "aff file url      : {}",
                match source {
                    Some(s) => &s.aff,
                    None => "",
                }
            );
            println!(
                "dic file url      : {}",
                match source {
                    Some(s) => &s.dic,
                    None => "",
                }
            );
            let aff_on_disk = provider.aff_on_disk();
            if aff_on_disk.len() > 0 {
                println!(
                    "aff files on disk : {}",
                    aff_on_disk.get(0).unwrap().display()
                );
                for d in aff_on_disk.iter().skip(1) {
                    println!("                  : {}", d.display());
                }
            }
            let dic_on_disk = provider.dic_on_disk();
            if dic_on_disk.len() > 0 {
                println!(
                    "dic files on disk : {}",
                    dic_on_disk.get(0).unwrap().display()
                );
                for d in dic_on_disk.iter().skip(1) {
                    println!("                  : {}", d.display());
                }
            }
            Ok(())
        }
        Cli::Ensure { lang } => {
            let provider = LangProvider::new(&lang, &config);
            provider.ensure_data()
        }
        Cli::Remove { lang } => {
            let provider = LangProvider::new(&lang, &config);
            provider.remove_data()
        }
    }
}
