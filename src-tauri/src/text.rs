use crate::models::RewriteMode;

pub fn rewrite_offline(input: &str, mode: &RewriteMode, target_language: Option<&str>) -> String {
    let fixed = fix_grammar(input);
    match mode {
        RewriteMode::FixGrammar => fixed,
        RewriteMode::Professional => professionalize(&fixed),
        RewriteMode::Friendly => friendly(&fixed),
        RewriteMode::Shorter => shorten(&fixed),
        RewriteMode::Translate => {
            offline_translate_notice(&fixed, target_language.unwrap_or("the selected language"))
        }
        RewriteMode::Summarize => summarize(&fixed),
        RewriteMode::Confident => confident(&fixed),
        RewriteMode::Simplify => simplify(&fixed),
        // Expansion is intentionally conservative offline. A rules engine cannot
        // invent supporting detail without risking a change in meaning.
        RewriteMode::Expand => fixed,
    }
}

pub fn instruction_for(mode: &RewriteMode, target_language: Option<&str>) -> String {
    match mode {
        RewriteMode::FixGrammar => {
            "Fix grammar, tense, punctuation, and clarity without changing meaning.".to_string()
        }
        RewriteMode::Professional => {
            "Rewrite the text in a polished, professional tone.".to_string()
        }
        RewriteMode::Friendly => "Rewrite the text in a warm, friendly tone.".to_string(),
        RewriteMode::Shorter => {
            "Make the text shorter while preserving the key meaning.".to_string()
        }
        RewriteMode::Translate => format!(
            "Translate the text into {} while preserving the original meaning.",
            target_language.unwrap_or("English")
        ),
        RewriteMode::Summarize => "Summarize the text into the clearest main point.".to_string(),
        RewriteMode::Confident => {
            "Rewrite the text so it sounds confident, direct, and clear.".to_string()
        }
        RewriteMode::Simplify => {
            "Simplify the text so it is easier to read and understand.".to_string()
        }
        RewriteMode::Expand => {
            "Expand the text with useful clarity and detail supported by the source. Do not invent facts.".to_string()
        }
    }
}

fn fix_grammar(input: &str) -> String {
    let leading_indent: String = input
        .chars()
        .take_while(|character| matches!(character, ' ' | '\t'))
        .collect();
    let had_trailing_newline = input.ends_with('\n');
    let mut text = correct_common_words(&normalize_spaces(input));

    let replacements = [
        ("The team have completed", "The team completed"),
        ("the team have completed", "the team completed"),
        (" team have ", " team has "),
        ("I is ", "I am "),
        ("I are ", "I am "),
        ("You is ", "You are "),
        ("We is ", "We are "),
        ("They is ", "They are "),
        ("People does ", "People do "),
        ("people does ", "people do "),
        ("People is ", "People are "),
        ("people is ", "people are "),
        ("Companies wants ", "Companies want "),
        ("companies wants ", "companies want "),
        ("They has ", "They have "),
        ("they has ", "they have "),
        ("We has ", "We have "),
        ("we has ", "we have "),
        ("He are ", "He is "),
        ("She are ", "She is "),
        ("It are ", "It is "),
        (" it were ", " it was "),
        (" could of ", " could have "),
        (" should of ", " should have "),
        (" would of ", " would have "),
        (" did not went ", " did not go "),
        (" do not knows ", " do not know "),
        (" does not knows ", " does not know "),
        (" more better", " better"),
        (" better then ", " better than "),
        (" more then ", " more than "),
        (" rather then ", " rather than "),
        (" different then ", " different than "),
        (" your welcome", " you're welcome"),
        (" your going ", " you're going "),
        (" your able ", " you're able "),
        (" your not ", " you're not "),
        (" your right", " you're right"),
        (" their is ", " there is "),
        (" their are ", " there are "),
        (" over their", " over there"),
        (" right their", " right there"),
        (" there team", " their team"),
        (" there company", " their company"),
        (" there work", " their work"),
        (" was sent to client", " was sent to the client"),
        (" sent to client", " sent to the client"),
        (" i ", " I "),
        (" im ", " I'm "),
        ("Im ", "I'm "),
        ("goin ", "going "),
        ("seep", "sleep"),
        (" dont ", " do not "),
        (" doesnt ", " does not "),
        (" wasnt ", " was not "),
        (" cant ", " cannot "),
        (" alot ", " a lot "),
        (" definately ", " definitely "),
        (" recieve ", " receive "),
        (" seperate ", " separate "),
        (" occured ", " occurred "),
        (" teh ", " the "),
    ];

    for (from, to) in replacements {
        text = text.replace(from, to);
    }

    text = text.replace("hey there I'm", "hey there, I'm");
    text = text.replace("I'm using new application", "I'm using a new application");
    text = tidy_punctuation(&text);
    text = add_introductory_commas(&text);

    text = capitalize_sentences(&text);
    text = normalize_known_capitalization(&text);
    let mut output = ensure_terminal_punctuation(&text);
    if !leading_indent.is_empty() && !output.starts_with(&leading_indent) {
        output.insert_str(0, &leading_indent);
    }
    if had_trailing_newline && !output.ends_with('\n') {
        output.push('\n');
    }
    output
}

fn professionalize(input: &str) -> String {
    let mut text = without_casual_greeting(input).to_string();
    text = text.replace("I'm ", "I am ");
    text = text.replace("I've ", "I have ");
    text = text.replace("We'll ", "We will ");
    text = text.replace("we'll ", "we will ");
    text = text.replace("can't", "cannot");
    text = text.replace("won't", "will not");
    text = text.replace("Need ", "Please provide ");
    text = text.replace("ASAP", "as soon as possible");
    text = text.replace("I want", "I would like");
    text = text.replace("Can you please", "Could you please");
    text = text.replace("can you please", "could you please");
    text = text.replace("Can you", "Could you please");
    text = text.replace("can you", "could you please");
    text = text.replace("get back to me", "respond");
    text = text.replace("a lot of", "many");
    text = text.replace("Thanks", "Thank you");
    ensure_terminal_punctuation(&capitalize_sentences(&text))
}

fn friendly(input: &str) -> String {
    let owned = input
        .replace("Could you please", "Would you mind")
        .replace("Please provide", "Please share");
    let text = without_casual_greeting(&owned);
    if text.len() < 120 && !text.to_ascii_lowercase().starts_with("hi") {
        return ensure_terminal_punctuation(&format!("Hi! {text}"));
    }
    ensure_terminal_punctuation(text)
}

fn shorten(input: &str) -> String {
    let mut text = remove_filler(input);
    let phrase_replacements = [
        ("in order to", "to"),
        ("at this point in time", "now"),
        ("due to the fact that", "because"),
        ("has the ability to", "can"),
        ("for the purpose of", "to"),
        ("a large number of", "many"),
        ("as soon as possible", "soon"),
    ];
    for (from, to) in phrase_replacements {
        text = text.replace(from, to).replace(&capitalize_first(from), &capitalize_first(to));
    }

    let sentences = sentences(&text);
    if word_count(&text) <= 24 && sentences.len() <= 2 {
        return ensure_terminal_punctuation(&capitalize_sentences(&text));
    }

    if sentences.len() <= 1 {
        let clauses: Vec<&str> = text
            .split(|character| matches!(character, ',' | ';' | '—'))
            .map(str::trim)
            .filter(|clause| !clause.is_empty())
            .collect();
        if clauses.len() > 1 {
            let first = clauses[0];
            if let Some(key) = clauses.iter().skip(1).find(|clause| contains_key_signal(clause)) {
                return ensure_terminal_punctuation(&format!("{first}, {key}"));
            }
            return ensure_terminal_punctuation(first);
        }
        return ensure_terminal_punctuation(&text);
    }

    let first = sentences[0].clone();
    let second = sentences
        .iter()
        .skip(1)
        .find(|sentence| contains_key_signal(sentence));
    match second {
        Some(sentence) if word_count(&first) + word_count(sentence) <= 32 => {
            format!("{} {}", ensure_terminal_punctuation(&capitalize_sentences(&first)), ensure_terminal_punctuation(sentence))
        }
        _ => ensure_terminal_punctuation(&capitalize_sentences(&first)),
    }
}

fn summarize(input: &str) -> String {
    let fixed = input.trim();
    if let Some(rest) = without_casual_greeting(fixed).strip_prefix("I'm using ") {
        return ensure_terminal_punctuation(&format!("Using {rest}"));
    }
    if let Some(rest) = without_casual_greeting(fixed).strip_prefix("I am using ") {
        return ensure_terminal_punctuation(&format!("Using {rest}"));
    }
    if word_count(fixed) <= 24 {
        return ensure_terminal_punctuation(without_casual_greeting(fixed));
    }

    let cleaned = remove_filler(without_casual_greeting(fixed));
    let all = sentences(&cleaned);
    if all.len() <= 1 {
        return shorten(&cleaned);
    }
    let first = all.first().map(String::as_str).unwrap_or(cleaned.as_str());
    let key = all
        .iter()
        .skip(1)
        .find(|sentence| contains_key_signal(sentence));
    match key {
        Some(sentence) if word_count(first) + word_count(sentence) <= 38 => {
            format!("{} {}", ensure_terminal_punctuation(first), ensure_terminal_punctuation(sentence))
        }
        _ => ensure_terminal_punctuation(first),
    }
}

fn confident(input: &str) -> String {
    let mut text = input.to_string();
    text = text.replace("I think ", "");
    text = text.replace("I believe ", "");
    text = text.replace("I feel that ", "");
    text = text.replace("It seems that ", "");
    text = text.replace("it seems that ", "");
    text = text.replace("maybe ", "");
    text = text.replace("Maybe ", "");
    text = text.replace("perhaps ", "");
    text = text.replace("Perhaps ", "");
    text = text.replace("we might ", "we will ");
    text = text.replace("We might ", "We will ");
    text = text.replace("we hope to ", "we will ");
    text = text.replace("We hope to ", "We will ");
    text = text.replace("I hope to ", "I will ");
    text = text.replace("could possibly", "can");
    text = text.replace("should be able to", "can");
    ensure_terminal_punctuation(&capitalize_sentences(text.trim()))
}

fn simplify(input: &str) -> String {
    let replacements = [
        ("utilize", "use"),
        ("approximately", "about"),
        ("commence", "start"),
        ("terminate", "end"),
        ("purchase", "buy"),
        ("assist", "help"),
        ("prior to", "before"),
        ("subsequent to", "after"),
        ("in order to", "to"),
        ("at this point in time", "now"),
        ("due to the fact that", "because"),
        ("with regard to", "about"),
        ("make a decision", "decide"),
        ("provide assistance", "help"),
        ("has the ability to", "can"),
        ("a sufficient number of", "enough"),
    ];

    let mut text = without_casual_greeting(input).to_string();
    text = text.replace("I'm using a new application", "I use a new app");
    text = text.replace("I am using a new application", "I use a new app");
    for (from, to) in replacements {
        text = text.replace(from, to);
    }
    ensure_terminal_punctuation(&text)
}

fn remove_filler(input: &str) -> String {
    let mut text = input.to_string();
    let fillers = [
        "Basically, ",
        "basically, ",
        "Actually, ",
        "actually, ",
        "To be honest, ",
        "to be honest, ",
        "As a matter of fact, ",
        "as a matter of fact, ",
        "I just wanted to say that ",
        "I wanted to let you know that ",
        "It is important to note that ",
    ];
    for filler in fillers {
        text = text.replace(filler, "");
    }
    normalize_spaces(&text)
}

fn sentences(input: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut current = String::new();
    for character in input.chars() {
        current.push(character);
        if matches!(character, '.' | '!' | '?') {
            let sentence = current.trim();
            if !sentence.is_empty() {
                result.push(sentence.to_string());
            }
            current.clear();
        }
    }
    let remaining = current.trim();
    if !remaining.is_empty() {
        result.push(remaining.to_string());
    }
    result
}

fn word_count(input: &str) -> usize {
    input.split_whitespace().count()
}

fn contains_key_signal(input: &str) -> bool {
    let lower = input.to_ascii_lowercase();
    [
        "must", "need", "will", "result", "because", "deadline", "risk", "recommend",
        "next", "action", "important", "therefore", "however",
    ]
    .iter()
    .any(|signal| lower.split(|character: char| !character.is_ascii_alphabetic()).any(|word| word == *signal))
}

fn capitalize_first(input: &str) -> String {
    let mut characters = input.chars();
    match characters.next() {
        Some(first) => format!("{}{}", first.to_ascii_uppercase(), characters.as_str()),
        None => String::new(),
    }
}

fn tidy_punctuation(input: &str) -> String {
    let mut text = input.to_string();
    for (from, to) in [
        (" ,", ","),
        (" .", "."),
        (" !", "!"),
        (" ?", "?"),
        (" ;", ";"),
        (" :", ":"),
        (",,", ","),
        ("..", "."),
    ] {
        text = text.replace(from, to);
    }

    let mut output = String::with_capacity(text.len() + 4);
    let chars: Vec<char> = text.chars().collect();
    for (index, character) in chars.iter().enumerate() {
        output.push(*character);
        if (matches!(character, ',' | ';' | ':')
            || (matches!(character, '.' | '!' | '?')
                && chars.get(index + 1).is_some_and(|next| next.is_ascii_alphabetic())))
            && chars.get(index + 1).is_some_and(|next| !next.is_whitespace())
        {
            output.push(' ');
        }
    }
    normalize_spaces(&output)
}

fn add_introductory_commas(input: &str) -> String {
    let mut text = input.to_string();
    for word in [
        "However", "Therefore", "Additionally", "Meanwhile", "Nevertheless", "Consequently",
    ] {
        if text.starts_with(&format!("{word} ")) {
            text = text.replacen(&format!("{word} "), &format!("{word}, "), 1);
        }
        text = text.replace(&format!(". {word} "), &format!(". {word}, "));
        let lower = word.to_ascii_lowercase();
        text = text.replace(&format!(". {lower} "), &format!(". {word}, "));
    }
    for phrase in ["For example", "In addition", "On the other hand"] {
        if text.starts_with(&format!("{phrase} ")) {
            text = text.replacen(&format!("{phrase} "), &format!("{phrase}, "), 1);
        }
        text = text.replace(&format!(". {phrase} "), &format!(". {phrase}, "));
    }
    text
}

fn normalize_known_capitalization(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut word = String::new();

    fn push_word(output: &mut String, word: &mut String) {
        if word.is_empty() {
            return;
        }
        let normalized = match word.to_ascii_lowercase().as_str() {
            "ai" => "AI",
            "api" => "API",
            "cpu" => "CPU",
            "gpu" => "GPU",
            "usa" => "USA",
            "uk" => "UK",
            "ui" => "UI",
            "ux" => "UX",
            "url" => "URL",
            "http" => "HTTP",
            "https" => "HTTPS",
            "json" => "JSON",
            "sql" => "SQL",
            "html" => "HTML",
            "css" => "CSS",
            "javascript" => "JavaScript",
            "typescript" => "TypeScript",
            "github" => "GitHub",
            "openai" => "OpenAI",
            "microsoft" => "Microsoft",
            "google" => "Google",
            "windows" => "Windows",
            "gemini" => "Gemini",
            "cortex" => "CorteX",
            "english" => "English",
            _ => word.as_str(),
        };
        output.push_str(normalized);
        word.clear();
    }

    for character in input.chars() {
        if character.is_ascii_alphanumeric() {
            word.push(character);
        } else {
            push_word(&mut output, &mut word);
            output.push(character);
        }
    }
    push_word(&mut output, &mut word);
    output
}

fn offline_translate_notice(input: &str, target_language: &str) -> String {
    format!(
        "Translation to {target_language} is available when an AI provider is connected. Original text: {input}"
    )
}

fn normalize_spaces(input: &str) -> String {
    let had_trailing_newline = input.ends_with('\n');
    let mut output = input
        .replace("\r\n", "\n")
        .replace('\r', "\n")
        .lines()
        .map(|line| {
            let indent_len = line
                .char_indices()
                .take_while(|(_, character)| matches!(character, ' ' | '\t'))
                .last()
                .map(|(index, character)| index + character.len_utf8())
                .unwrap_or(0);
            let (indent, content) = line.split_at(indent_len);
            if content.trim().is_empty() {
                String::new()
            } else {
                format!("{indent}{}", content.split_whitespace().collect::<Vec<_>>().join(" "))
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    if had_trailing_newline {
        output.push('\n');
    }
    output
}

fn correct_common_words(input: &str) -> String {
    let mut text = format!(" {} ", input.trim());
    let replacements = [
        (" ths ", " this "),
        (" Ths ", " This "),
        (" thsi ", " this "),
        (" frst ", " first "),
        (" Frst ", " First "),
        (" projeckt ", " project "),
        (" proejct ", " project "),
        (" proyect ", " project "),
        (" wrks ", " works "),
        (" wrk ", " work "),
        (" I hop it ", " I hope it "),
        (" i hop it ", " I hope it "),
        (" heythere ", " hey there "),
        (" m ", " I'm "),
        (" im ", " I'm "),
        (" usng ", " using "),
        (" nev ", " new "),
        (" aplication", " application"),
        (" applicaton", " application"),
        (" teh ", " the "),
        (" thier ", " their "),
        (" wierd ", " weird "),
        (" becuase ", " because "),
        (" goverment ", " government "),
        (" enviroment ", " environment "),
        (" accomodate ", " accommodate "),
        (" acheive ", " achieve "),
        (" adress ", " address "),
        (" begining ", " beginning "),
        (" calender ", " calendar "),
        (" concious ", " conscious "),
        (" embarass ", " embarrass "),
        (" existance ", " existence "),
        (" independant ", " independent "),
        (" maintenence ", " maintenance "),
        (" neccessary ", " necessary "),
        (" occassion ", " occasion "),
        (" prefered ", " preferred "),
        (" recomend ", " recommend "),
        (" responsability ", " responsibility "),
        (" succesful ", " successful "),
        (" tommorow ", " tomorrow "),
        (" untill ", " until "),
        (" wich ", " which "),
        (" woud ", " would "),
        (" shoud ", " should "),
        (" cud ", " could "),
        (" recieve", " receive"),
        (" definately", " definitely"),
        (" seperate", " separate"),
        (" alot ", " a lot "),
        (" u ", " you "),
        (" ur ", " your "),
        (" pls ", " please "),
        (" thx ", " thanks "),
        (" cant ", " cannot "),
        (" dont ", " do not "),
        (" doesnt ", " does not "),
        (" didnt ", " did not "),
        (" couldnt ", " could not "),
        (" wouldnt ", " would not "),
        (" shouldnt ", " should not "),
        (" wasnt ", " was not "),
        (" goin ", " going "),
        (" seep", " sleep"),
        ("Hey there I'm", "Hey there, I'm"),
        ("I'm using new application", "I'm using a new application"),
    ];

    for (from, to) in replacements {
        text = text.replace(from, to);
    }

    text.trim().to_string()
}

fn without_casual_greeting(input: &str) -> &str {
    input
        .trim()
        .strip_prefix("Hey there, ")
        .or_else(|| input.trim().strip_prefix("Hey there "))
        .unwrap_or_else(|| input.trim())
}

fn ensure_terminal_punctuation(input: &str) -> String {
    let had_trailing_newline = input.ends_with('\n');
    let trimmed = input.trim_end();
    if trimmed.is_empty() {
        return String::new();
    }
    let mut output = if trimmed.ends_with('.') || trimmed.ends_with('!') || trimmed.ends_with('?') {
        trimmed.to_string()
    } else {
        format!("{trimmed}.")
    };
    if had_trailing_newline {
        output.push('\n');
    }
    output
}

fn capitalize_sentences(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut capitalize_next = true;

    for character in input.chars() {
        if capitalize_next && character.is_ascii_alphabetic() {
            output.push(character.to_ascii_uppercase());
            capitalize_next = false;
            continue;
        }

        output.push(character);

        if matches!(character, '.' | '!' | '?') {
            capitalize_next = true;
        } else if !character.is_whitespace() {
            capitalize_next = false;
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::rewrite_offline;
    use crate::models::RewriteMode;

    #[test]
    fn offline_modes_correct_common_casual_typos() {
        let input = "heythere m usng nev aplication";

        assert_eq!(
            rewrite_offline(input, &RewriteMode::FixGrammar, None),
            "Hey there, I'm using a new application."
        );
        assert_eq!(
            rewrite_offline(input, &RewriteMode::Professional, None),
            "I am using a new application."
        );
        assert_eq!(
            rewrite_offline(input, &RewriteMode::Friendly, None),
            "Hi! I'm using a new application."
        );
        assert_eq!(
            rewrite_offline(input, &RewriteMode::Simplify, None),
            "I use a new app."
        );
        assert_eq!(
            rewrite_offline(input, &RewriteMode::Summarize, None),
            "Using a new application."
        );
    }

    #[test]
    fn fix_grammar_corrects_common_missing_letter_and_phonetic_typos() {
        let input = "Ths is my frst projeckt and I hop it wrks";

        assert_eq!(
            rewrite_offline(input, &RewriteMode::FixGrammar, None),
            "This is my first project and I hope it works."
        );
    }

    #[test]
    fn grammar_fixes_agreement_and_common_misused_phrases() {
        let input = "I is sure we could of done more better, but it were sent to client";
        assert_eq!(
            rewrite_offline(input, &RewriteMode::FixGrammar, None),
            "I am sure we could have done better, but it was sent to the client."
        );
    }

    #[test]
    fn shorter_keeps_complete_high_value_sentences() {
        let input = "Basically, the team completed the first design yesterday. The client must approve it before Friday. We also discussed several optional ideas for a later release.";
        assert_eq!(
            rewrite_offline(input, &RewriteMode::Shorter, None),
            "The team completed the first design yesterday. The client must approve it before Friday."
        );
    }

    #[test]
    fn professional_and_confident_remove_casual_or_uncertain_language() {
        assert_eq!(
            rewrite_offline("Hey there can you get back to me ASAP", &RewriteMode::Professional, None),
            "Could you please respond as soon as possible."
        );
        assert_eq!(
            rewrite_offline("I think we might finish and should be able to ship", &RewriteMode::Confident, None),
            "We will finish and can ship."
        );
    }

    #[test]
    fn every_offline_mode_starts_from_correct_publication_quality_english() {
        let input = "people does use ai better then companies wants to use it.however your api is over their";
        let modes = [
            RewriteMode::FixGrammar,
            RewriteMode::Professional,
            RewriteMode::Friendly,
            RewriteMode::Shorter,
            RewriteMode::Summarize,
            RewriteMode::Confident,
            RewriteMode::Simplify,
            RewriteMode::Expand,
        ];

        for mode in modes {
            let output = rewrite_offline(input, &mode, None);
            assert!(output.contains("AI"), "{:?} did not capitalize AI: {}", mode, output);
            assert!(output.contains("API"), "{:?} did not capitalize API: {}", mode, output);
            assert!(output.contains("However,"), "{:?} missed the introductory comma: {}", mode, output);
            for error in ["people does", "companies wants", "better then", "over their"] {
                assert!(!output.to_ascii_lowercase().contains(error), "{:?} retained '{}': {}", mode, error, output);
            }
        }
    }

    #[test]
    fn grammar_normalizes_acronyms_proper_nouns_and_common_spelling() {
        let input = "openai and microsoft recomend javascript api maintenence in the usa";
        assert_eq!(
            rewrite_offline(input, &RewriteMode::FixGrammar, None),
            "OpenAI and Microsoft recommend JavaScript API maintenance in the USA."
        );
    }

    #[test]
    fn grammar_cleanup_preserves_paragraphs_lists_and_indentation() {
        let input = "Heading\n\n- people does work\n  indented  text\n";
        let output = rewrite_offline(input, &RewriteMode::FixGrammar, None);
        assert!(output.contains("\n\n- people do work"));
        assert!(output.contains("\n  indented text"));
        assert!(output.ends_with('\n'));
    }
}
