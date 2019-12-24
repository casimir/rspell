use rspell::Spell;

fn main() {
    env_logger::init();
    let spell = Spell::new("fr_FR").unwrap();
    for bad in spell.check("Coment est votre blaquette ?") {
        println!(
            "{} (offset: {}): possible corrections: {:?}",
            bad.word, bad.offset, bad.suggestions
        );
    }
}
