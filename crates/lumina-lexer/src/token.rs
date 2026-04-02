use logos::Logos;
use serde::{Serialize, Deserialize};

#[derive(Logos, Debug, Clone, PartialEq)]
pub enum Token {
    // ── Keywords ──────────────────────────────────────────
    #[token("entity")]   KwEntity,
    #[token("let")]      KwLet,
    #[token("rule")]     KwRule,
    #[token("when")]     KwWhen,
    #[token("then")]     KwThen,
    #[token("becomes")]  KwBecomes,
    #[token("for")]      KwFor,
    #[token("every")]    KwEvery,
    #[token("external")] KwExternal,
    #[token("sync")]     KwSync,
    #[token("on")]       KwOn,
    #[token("show")]     KwShow,
    #[token("update")]   KwUpdate,
    #[token("to")]       KwTo,
    #[token("create")]   KwCreate,
    #[token("delete")]   KwDelete,
    #[token("if")]       KwIf,
    #[token("else")]     KwElse,
    #[token("and")]      KwAnd,
    #[token("or")]       KwOr,
    #[token("not")]      KwNot,
    #[token("true")]     KwTrue,
    #[token("false")]    KwFalse,
    #[token("Text")]     KwTypeText,
    #[token("Number")]   KwTypeNumber,
    #[token("Boolean")]  KwTypeBoolean,
    #[token("fn")]       KwFn,
    #[token("import")]   Import,
    #[token("prev")]     KwPrev,
    #[token("any")]      KwAny,
    #[token("all")]      KwAll,
    #[token("alert")]    Alert,
    #[token("severity")] Severity,
    #[token("aggregate")] Aggregate,
    #[token("over")]      Over,
    #[token("cooldown")]   Cooldown,
    #[token("clear")]      Clear,
    #[token("poll_interval")] KwPollInterval,
    #[token("sync_on")]      KwSyncOn,
    #[token("ref")]        KwRef,
    #[token("times")]      KwTimes,
    #[token("within")]     KwWithin,
    #[token("write")]      KwWrite,
    #[token("Timestamp")]  KwTypeTimestamp,
    #[token("now")]        KwNow,

    // ── Operators & punctuation ────────────────────────────
    #[token(":=")]  ColonEq,
    #[token(":")]   Colon,
    #[token("==")]  EqEq,
    #[token("=")]   Eq,
    #[token("!=")]  BangEq,
    #[token(">=")]  GtEq,
    #[token("<=")]  LtEq,
    #[token(">")]   Gt,
    #[token("<")]   Lt,
    #[token("+")]   Plus,
    #[token("-")]   Minus,
    #[token("*")]   Star,
    #[token("/")]   Slash,
    #[token("{")]   LBrace,
    #[token("}")]   RBrace,
    #[token("(")]   LParen,
    #[token(")")]   RParen,
    #[token(",")]   Comma,
    #[token(".")]   Dot,
    #[token("@")]   At,
    #[token("->")]  Arrow,
    #[token("[")]   LBracket,
    #[token("]")]   RBracket,

    // ── Literals ───────────────────────────────────────────
    #[regex(r"[0-9]+(\.[0-9]+)?", |lex| lex.slice().parse::<f64>().ok())]
    Number(f64),

    #[regex(r#""([^"\\]|\\.)*""#, |lex| {
        let s = lex.slice();
        Some(s[1..s.len()-1].to_string())
    })]
    Text(String),

    // ── Interpolation (post-processed) ─────────────────────
    InterpStringStart,
    InterpPart(String),
    InterpExprStart,
    InterpExprEnd,
    InterpStringEnd,

    // ── Identifiers ────────────────────────────────────────
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Ident(String),

    // ── Whitespace & comments ──────────────────────────────
    #[regex(r"--[^\n]*", logos::skip)]
    #[regex(r"[ \t\r]+", logos::skip)]

    #[token("\n")]
    Newline,
}

impl Token {
    pub fn to_human_string(&self) -> String {
        match self {
            Token::KwEntity => "keyword 'entity'".to_string(),
            Token::KwLet => "keyword 'let'".to_string(),
            Token::KwRule => "keyword 'rule'".to_string(),
            Token::KwWhen => "keyword 'when'".to_string(),
            Token::KwThen => "keyword 'then'".to_string(),
            Token::KwBecomes => "keyword 'becomes'".to_string(),
            Token::KwFor => "keyword 'for'".to_string(),
            Token::KwEvery => "keyword 'every'".to_string(),
            Token::KwExternal => "keyword 'external'".to_string(),
            Token::KwSync => "keyword 'sync'".to_string(),
            Token::KwOn => "keyword 'on'".to_string(),
            Token::KwShow => "keyword 'show'".to_string(),
            Token::KwUpdate => "keyword 'update'".to_string(),
            Token::KwTo => "keyword 'to'".to_string(),
            Token::KwCreate => "keyword 'create'".to_string(),
            Token::KwDelete => "keyword 'delete'".to_string(),
            Token::KwIf => "keyword 'if'".to_string(),
            Token::KwElse => "keyword 'else'".to_string(),
            Token::KwAnd => "keyword 'and'".to_string(),
            Token::KwOr => "keyword 'or'".to_string(),
            Token::KwNot => "keyword 'not'".to_string(),
            Token::KwTrue => "boolean 'true'".to_string(),
            Token::KwFalse => "boolean 'false'".to_string(),
            Token::KwTypeText => "type 'Text'".to_string(),
            Token::KwTypeNumber => "type 'Number'".to_string(),
            Token::KwTypeBoolean => "type 'Boolean'".to_string(),
            Token::KwTypeTimestamp => "type 'Timestamp'".to_string(),
            Token::KwFn => "keyword 'fn'".to_string(),
            Token::Import => "keyword 'import'".to_string(),
            Token::KwPrev => "keyword 'prev'".to_string(),
            Token::KwAny => "keyword 'any'".to_string(),
            Token::KwAll => "keyword 'all'".to_string(),
            Token::Alert => "keyword 'alert'".to_string(),
            Token::Severity => "keyword 'severity'".to_string(),
            Token::Aggregate => "keyword 'aggregate'".to_string(),
            Token::Over => "keyword 'over'".to_string(),
            Token::Cooldown => "keyword 'cooldown'".to_string(),
            Token::Clear => "keyword 'clear'".to_string(),
            Token::KwPollInterval => "keyword 'poll_interval'".to_string(),
            Token::KwSyncOn => "keyword 'sync_on'".to_string(),
            Token::KwRef => "keyword 'ref'".to_string(),
            Token::KwTimes => "keyword 'times'".to_string(),
            Token::KwWithin => "keyword 'within'".to_string(),
            Token::KwWrite => "keyword 'write'".to_string(),
            Token::KwNow => "keyword 'now'".to_string(),
            
            Token::ColonEq => "':='".to_string(),
            Token::Colon => "':'".to_string(),
            Token::EqEq => "'=='".to_string(),
            Token::Eq => "'='".to_string(),
            Token::BangEq => "'!='".to_string(),
            Token::GtEq => "'>='".to_string(),
            Token::LtEq => "'<='".to_string(),
            Token::Gt => "'>'".to_string(),
            Token::Lt => "'<'".to_string(),
            Token::Plus => "'+'".to_string(),
            Token::Minus => "'-'".to_string(),
            Token::Star => "'*'".to_string(),
            Token::Slash => "'/'".to_string(),
            Token::LBrace => "'{'".to_string(),
            Token::RBrace => "'}'".to_string(),
            Token::LParen => "'('".to_string(),
            Token::RParen => "')'".to_string(),
            Token::Comma => "','".to_string(),
            Token::Dot => "'.'".to_string(),
            Token::At => "'@'".to_string(),
            Token::Arrow => "'->'".to_string(),
            Token::LBracket => "'['".to_string(),
            Token::RBracket => "']'".to_string(),
            
            Token::Number(n) => format!("number '{}'", n),
            Token::Text(s) => format!("text \"{}\"", s),
            Token::Ident(s) => format!("identifier '{}'", s),
            Token::Newline => "newline".to_string(),
            
            Token::InterpStringStart => "start of string".to_string(),
            Token::InterpPart(s) => format!("string segment '{}'", s),
            Token::InterpExprStart => "'${'".to_string(),
            Token::InterpExprEnd => "'}'".to_string(),
            Token::InterpStringEnd => "end of string".to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct Span {
    pub start: u32,
    pub end:   u32,
    pub line:  u32,
    pub col:   u32,
}

#[derive(Debug, Clone)]
pub struct SpannedToken {
    pub token: Token,
    pub span:  Span,
}
