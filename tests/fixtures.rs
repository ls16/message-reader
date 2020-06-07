#[allow(dead_code)]
pub fn reg_exp() -> String {
  "
    space         \\s+
    digit         [0-9]       DEF
    letter        _|[A-Za-z]  DEF
    id            {letter}({letter}|{digit})*
    number        {digit}({digit}|{digit})*
    plus          \\+ {set_name('+')}
    mul           \\* {set_name('*')}
    lbracket      \\( {set_name('(')}
    rbracket      \\) {set_name(')')}
  ".to_string()
}

#[allow(dead_code)]
pub fn regular_definition_text1() -> String {
  "
    http HTTP\\u2764
    method_name OPTIONS|GET|HEAD|POST|PUT|DELETE|TRACE|CONNECT
  ".to_string()
}

#[allow(dead_code)]
pub fn lex_text() -> String {
  "name123 456 (75+86*7)".to_string()
}

#[allow(dead_code)]
pub fn grammar() -> String {
  "
    E1: E;
    E: E '+' T | T;
    T: T '*' F | F;
    F: '(' E ')' | 'id' | 'number';
  ".to_string()
}