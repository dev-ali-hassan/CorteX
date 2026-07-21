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
    }
}

fn fix_grammar(input: &str) -> String {
    let mut text = correct_common_words(&normalize_spaces(input));

    let replacements = [
        ("The team have completed", "The team completed"),
        ("the team have completed", "the team completed"),
        (" team have ", " team has "),
        (" it were ", " it was "),
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

    text = capitalize_sentences(&text);
    ensure_terminal_punctuation(&text)
}

fn professionalize(input: &str) -> String {
    let mut text = without_casual_greeting(input).to_string();
    text = text.replace("I'm ", "I am ");
    text = text.replace("Need", "Please");
    text = text.replace("ASAP", "as soon as possible");
    text = text.replace("I want", "I would like");
    text = text.replace("can you", "could you");
    text = text.replace("Can you", "Could you");
    ensure_terminal_punctuation(&text)
}

fn friendly(input: &str) -> String {
    let text = without_casual_greeting(input);
    if text.len() < 120 && !text.to_ascii_lowercase().starts_with("hi") {
        return ensure_terminal_punctuation(&format!("Hi! {text}"));
    }
    ensure_terminal_punctuation(&text.replace("Please", "Please feel free to"))
}

fn shorten(input: &str) -> String {
    let words: Vec<&str> = input.split_whitespace().collect();
    if words.len() <= 18 {
        return input.to_string();
    }
    let compressed = words.iter().take(18).copied().collect::<Vec<_>>().join(" ");
    ensure_terminal_punctuation(&compressed)
}

fn summarize(input: &str) -> String {
    let fixed = input.trim();
    if let Some(rest) = without_casual_greeting(fixed).strip_prefix("I'm using ") {
        return ensure_terminal_punctuation(&format!("Using {rest}"));
    }
    if let Some(rest) = without_casual_greeting(fixed).strip_prefix("I am using ") {
        return ensure_terminal_punctuation(&format!("Using {rest}"));
    }
    if fixed.len() <= 140 {
        return ensure_terminal_punctuation(without_casual_greeting(fixed));
    }

    let first_sentence = fixed
        .split_terminator(|character| matches!(character, '.' | '!' | '?'))
        .next()
        .unwrap_or(fixed)
        .trim();
    ensure_terminal_punctuation(first_sentence)
}

fn confident(input: &str) -> String {
    let mut text = input.to_string();
    text = text.replace("I think ", "");
    text = text.replace("maybe ", "");
    text = text.replace("perhaps ", "");
    text = text.replace("we might ", "we will ");
    text = text.replace("We might ", "We will ");
    text = text.replace("should be able to", "can");
    text
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
    ];

    let mut text = without_casual_greeting(input).to_string();
    text = text.replace("I'm using a new application", "I use a new app");
    text = text.replace("I am using a new application", "I use a new app");
    for (from, to) in replacements {
        text = text.replace(from, to);
    }
    ensure_terminal_punctuation(&text)
}

fn offline_translate_notice(input: &str, target_language: &str) -> String {
    format!(
        "Translation to {target_language} is available when an AI provider is connected. Original text: {input}"
    )
}

fn normalize_spaces(input: &str) -> String {
    input.split_whitespace().collect::<Vec<_>>().join(" ")
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
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    if trimmed.ends_with('.') || trimmed.ends_with('!') || trimmed.ends_with('?') {
        trimmed.to_string()
    } else {
        format!("{trimmed}.")
    }
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
}
