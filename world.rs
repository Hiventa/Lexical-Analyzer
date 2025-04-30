use std::collections::{HashMap, HashSet}; // 哈希表，哈希集合
use std::fs::{self, OpenOptions}; // 文件操作
use std::io::{Write, Result}; // 文件读写

#[derive(Debug, Clone)]
enum Token {
    Keyword(String),
    Identifier(String),
    Constant(i32),
    StringLiteral(String),
    CharLiteral(char),
    Separator(char),
    Operator(String),
    Comment(String),
}

#[derive(Debug)]
struct Lexer {
    id_table: HashMap<usize, String>,     
    const_table: HashMap<usize, i32>,     
    keywords: HashSet<String>,            
    current_id: usize,                   
    current_const: usize,                
}

impl Lexer {
    fn new() -> Self {
        let keywords = [
            "as", "async", "await", "break", "const", "continue", "crate", "dyn", 
            "else", "enum", "extern", "false", "fn", "for", "if", "impl", "in", 
            "let", "loop", "match", "mod", "move", "mut", "pub", "ref", "return", 
            "self", "Self", "static", "struct", "super", "true", "trait", "type", 
            "unsafe", "use", "where", "while"
        ].iter().cloned().map(String::from).collect();

        Lexer {
            id_table: HashMap::new(),
            const_table: HashMap::new(),
            keywords,
            current_id: 0,
            current_const: 0,
        }
    }

    fn is_keyword(&self, word: &str) -> bool {
        self.keywords.contains(word)
    }

    fn lexer(&mut self, code: &str) -> Vec<Token> {
        let mut tokens = Vec::new();
        let mut current_word = String::new();
        let mut in_comment = false;
        let mut comment_content = String::new();
        let mut chars = code.chars().peekable();

        while let Some(c) = chars.next() {
            if in_comment {
                comment_content.push(c);
                if let Some('/') = chars.peek() {
                    chars.next();
                    tokens.push(Token::Comment(comment_content.clone()));
                    comment_content.clear();
                    in_comment = false;
                }
                continue;
            }

            if c == '/' {
                if let Some('/') = chars.peek() {
                    chars.next();
                    in_comment = true;
                    comment_content.push(c);
                    while let Some(next_char) = chars.next() {
                        if next_char == '\n' {
                            tokens.push(Token::Comment(comment_content.clone()));
                            comment_content.clear();
                            in_comment = false;
                            break;
                        }
                        comment_content.push(next_char);
                    }
                } else if let Some('*') = chars.peek() {
                    chars.next();
                    in_comment = true;
                    comment_content.push(c);
                    while let Some(next_char) = chars.next() {
                        comment_content.push(next_char);
                        if next_char == '*' {
                            if let Some('/') = chars.peek() {
                                chars.next();
                                tokens.push(Token::Comment(comment_content.clone()));
                                comment_content.clear();
                                in_comment = false;
                                break;
                            }
                        }
                    }
                } else {
                    current_word.push(c);
                    tokens.push(Token::Operator(current_word.clone()));
                    current_word.clear();
                }
                continue;
            }

            if c == '"' {
                let mut string_literal = String::new();
                while let Some(ch) = chars.next() {
                    if ch == '"' {
                        tokens.push(Token::StringLiteral(string_literal.clone()));
                        break;
                    } else if ch == '\\' {
                        if let Some(next_char) = chars.next() {
                            string_literal.push(next_char);
                        }
                    } else {
                        string_literal.push(ch);
                    }
                }
                continue;
            }

            if c == '\'' {
                if let Some(ch) = chars.next() {
                    if let Some('\'') = chars.next() {
                        tokens.push(Token::CharLiteral(ch));
                    } else {
                        eprintln!("Error: Invalid character literal.");
                    }
                }
                continue;
            }

            if c.is_whitespace() {
                if !current_word.is_empty() {
                    if self.is_keyword(&current_word) {
                        tokens.push(Token::Keyword(current_word.clone()));
                    } else if current_word.chars().all(|c| c.is_digit(10)) {
                        let num = current_word.parse::<i32>().unwrap();
                        tokens.push(Token::Constant(num));
                        self.const_table.insert(self.current_const, num);
                        self.current_const += 1;
                    } else {
                        if !self.id_table.values().any(|v| v == &current_word) {
                            tokens.push(Token::Identifier(current_word.clone()));
                            self.id_table.insert(self.current_id, current_word.clone());
                            self.current_id += 1;
                        }
                    }
                    current_word.clear();
                }
            } else if self.is_operator(c) {
                current_word.push(c);
                tokens.push(Token::Operator(current_word.clone()));
                current_word.clear();
            } else if self.is_separator(c) {
                tokens.push(Token::Separator(c));
            } else {
                current_word.push(c);
            }
        }

        tokens
    }

    fn is_operator(&self, c: char) -> bool {
        matches!(c, '>' | '<' | '=' | '+' | '-' | '*' | '/' | '|' | '&' | '!' | '%' | '^')
    }

    fn is_separator(&self, c: char) -> bool {
        matches!(c, '(' | ')' | '{' | '}' | '[' | ']' | ',' | ';')
    }

    fn write_token_to_file(&self, token: &Token, file: &mut fs::File) -> Result<()> {
        match token {
            Token::Keyword(k) => {
                writeln!(file, "(关键字, \"{}\")", k)?;
            }
            Token::Identifier(i) => {
                if let Some(index) = self.id_table.iter().find(|(_, v)| v == &i).map(|(k, _)| *k) {
                    writeln!(file, "(标识符, {})", index)?;
                }
            }
            Token::Constant(c) => {
                let binary = format!("{:b}", c);
                if let Some(index) = self.const_table.iter().find(|(_, v)| v == &c).map(|(k, _)| *k) {
                    writeln!(file, "(常数, {})", binary)?;
                }
            }
            Token::StringLiteral(s) => {
                writeln!(file, "(字符串字面量, \"{}\")", s)?;
            }
            Token::CharLiteral(c) => {
                writeln!(file, "(字符字面量, '{}')", c)?;
            }
            Token::Separator(s) => {
                writeln!(file, "(分隔符, \"{}\")", s)?;
            }
            Token::Operator(o) => {
                writeln!(file, "(运算符, \"{}\")", o)?;
            }
            Token::Comment(c) => {
                writeln!(file, "(注释, \"{}\")", c)?;
            }
        }
        Ok(())
    }

    fn print_symbol_table(&self, file: &mut fs::File) -> Result<()> {
        writeln!(file, "符号表:")?;
        writeln!(file, "标识符表:")?;
        writeln!(file, "{:<6} {:<10}", "序号", "标识符")?;
        
        let mut id_vec: Vec<_> = self.id_table.iter().collect();
        id_vec.sort_by(|a, b| a.0.cmp(&b.0));

        for (index, value) in id_vec {
            writeln!(file, "{:<6} \"{}\"", index, value)?;
        }

        writeln!(file, "常数表:")?;
        writeln!(file, "{:<6} {:<10}", "序号", "常数")?;
        for (index, value) in &self.const_table {
            writeln!(file, "{:<6} \"{}\"", index, format!("{:b}", value))?;
        }

        Ok(())
    }
}

fn main() -> Result<()> {
    let code = fs::read_to_string("code.txt").expect("Unable to read file");
    let mut lexer = Lexer::new();
    let tokens = lexer.lexer(&code);

    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open("show.txt")?;

    lexer.print_symbol_table(&mut file)?;

    for token in tokens {
        lexer.write_token_to_file(&token, &mut file)?;
    }

    Ok(())
}
