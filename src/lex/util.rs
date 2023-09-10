use super::{Error, Token};
use logos::Lexer;
use regex::Regex;

pub(super) fn parse_duration(lexer: &mut Lexer<Token>) -> Option<u64> {
    let mut overall = 0u64;
    let mut current = 0u64;

    for c in lexer.slice().chars() {
        if c.is_ascii_digit() {
            current *= 10;
            current += c.to_digit(10)? as u64;
        } else {
            let multiplier = match c {
                'w' => 604800,
                'd' => 86400,
                'h' => 3600,
                'm' => 60,
                's' => 1,
                _ => {
                    return None;
                }
            };
            overall += current * multiplier;
            current = 0;
        }
    }

    Some(overall)
}

fn find_closing_inner<'a>(
    lexer: &'a mut Lexer<Token>,
    find_tail: bool,
) -> Option<(&'a str, char, String, String)> {
    let opening = lexer.slice().chars().last()?;
    let mut body = String::new();
    let mut tail = String::new();
    let mut escaped = false;
    let mut chars = lexer.remainder().chars();

    for char in &mut chars {
        if escaped {
            escaped = false;
        } else if char == '\\' {
            escaped = true;
        } else if char == opening {
            break;
        }
        body.push(char);
    }

    if find_tail {
        for char in chars {
            if char.is_ascii_lowercase() || char.is_ascii_uppercase() {
                tail.push(char)
            } else {
                break;
            }
        }
    }

    // `+ 1` because neither `body` nor `tail` contains the middle delim
    lexer.bump(body.len() + 1 + tail.len());

    Some((lexer.slice(), opening, body, tail))
}

pub(super) fn find_closing(lexer: &mut Lexer<Token>) -> Option<String> {
    find_closing_inner(lexer, false).map(|(_, _, body, _)| body)
}

pub(super) fn compile_regex(lexer: &mut Lexer<Token>) -> Result<Regex, Error> {
    match find_closing_inner(lexer, true) {
        Some((_, _, body, _)) => Ok(Regex::new(&body)?),
        None => Err(Error::Unfinished),
    }
}
