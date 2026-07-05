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
    let mut text = normalize_spaces(input);

    let replacements = [
        ("The team have completed", "The team completed"),
        ("the team have completed", "the team completed"),
        (" team have ", " team has "),
        (" it were ", " it was "),
        (" was sent to client", " was sent to the client"),
        (" sent to client", " sent to the client"),
        (" i ", " I "),
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

    text = capitalize_sentences(&text);
    ensure_terminal_punctuation(&text)
}

fn professionalize(input: &str) -> String {
    let mut text = input.to_string();
    text = text.replace("Need", "Please");
    text = text.replace("ASAP", "as soon as possible");
    text = text.replace("I want", "I would like");
    text = text.replace("can you", "could you");
    text = text.replace("Can you", "Could you");
    text
}

fn friendly(input: &str) -> String {
    let text = input.trim();
    if text.len() < 120 {
        return format!("Hi, {}", lowercase_first(text));
    }
    text.replace("Please", "Please feel free to")
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
    if fixed.len() <= 140 {
        return fixed.to_string();
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

    let mut text = input.to_string();
    for (from, to) in replacements {
        text = text.replace(from, to);
    }
    text
}

fn offline_translate_notice(input: &str, target_language: &str) -> String {
    format!(
        "Translation to {target_language} is available when an AI provider is connected. Original text: {input}"
    )
}

fn normalize_spaces(input: &str) -> String {
    input.split_whitespace().collect::<Vec<_>>().join(" ")
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

fn lowercase_first(input: &str) -> String {
    let mut chars = input.chars();
    match chars.next() {
        Some(first) => format!(
            "{}{}",
            first.to_ascii_lowercase(),
            chars.collect::<String>()
        ),
        None => String::new(),
    }
}
