#[test]
fn test_parenthesis_0() {
    assert_eq!(
        chert::lex::lex("("),
        Ok(Vec::from([(chert::lex::Token::ParenthesisOpen, 0..1)]))
    );
}

#[test]
fn test_parenthesis_1() {
    assert_eq!(
        chert::lex::lex(")"),
        Ok(Vec::from([(chert::lex::Token::ParenthesisClose, 0..1)]))
    );
}

#[test]
fn test_duration_0() {
    assert_eq!(
        chert::lex::lex("1w"),
        Ok(Vec::from([(chert::lex::Token::Duration(604800), 0..2)]))
    );
}

#[test]
fn test_duration_1() {
    assert_eq!(
        chert::lex::lex("1d"),
        Ok(Vec::from([(chert::lex::Token::Duration(86400), 0..2)]))
    );
}

#[test]
fn test_duration_2() {
    assert_eq!(
        chert::lex::lex("1h"),
        Ok(Vec::from([(chert::lex::Token::Duration(3600), 0..2)]))
    );
}

#[test]
fn test_duration_3() {
    assert_eq!(
        chert::lex::lex("1m"),
        Ok(Vec::from([(chert::lex::Token::Duration(60), 0..2)]))
    );
}

#[test]
fn test_duration_4() {
    assert_eq!(
        chert::lex::lex("1s"),
        Ok(Vec::from([(chert::lex::Token::Duration(1), 0..2)]))
    );
}

#[test]
fn test_duration_5() {
    assert_eq!(
        chert::lex::lex("1w2d3h4m5s"),
        Ok(Vec::from([(chert::lex::Token::Duration(788645), 0..10)]))
    );
}

#[test]
fn test_lex_string_0() {
    assert_eq!(
        chert::lex::lex("\"asd\""),
        Ok(Vec::from([(
            chert::lex::Token::String("asd".to_owned()),
            0..5
        )]))
    );
}
#[test]
fn test_lex_string_1() {
    assert_eq!(
        chert::lex::lex("'asd'"),
        Ok(Vec::from([(
            chert::lex::Token::String("asd".to_owned()),
            0..5
        )]))
    );
}
