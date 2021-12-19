use tdn_did::Language;

mod english;
mod simplified_chinese;

pub(crate) fn load_answer(lang: &Language, index: usize) -> String {
    match lang {
        Language::SimplifiedChinese => simplified_chinese::WORDS[index].to_owned(),
        _ => english::WORDS[index].to_owned(),
    }
}
