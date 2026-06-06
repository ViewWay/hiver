//! SpEL expression parser — tokenizer + recursive descent.
//! SpEL 表达式解析器 — 词法分析 + 递归下降。

use std::fmt;

use thiserror::Error;

/// Parse / evaluation errors.
/// 解析/求值错误。
#[derive(Debug, Error)]
pub enum SpelError
{
    /// Parse error.
    /// 解析错误。
    #[error("parse error: {0}")]
    Parse(String),
    /// Evaluation error.
    /// 求值错误。
    #[error("evaluation error: {0}")]
    Evaluation(String),
}

/// Comparison operator.
/// 比较运算符。
#[derive(Debug, Clone, Copy)]
pub enum CmpOp
{
    Eq,
    NotEq,
    Gt,
    Lt,
    GtEq,
    LtEq,
}

/// Parsed SpEL expression AST.
/// 解析后的 SpEL 表达式 AST。
#[derive(Debug, Clone)]
pub enum SpelExpr
{
    /// `hasRole('X')` check.
    HasRole(String),
    /// `hasAuthority('X')` check.
    HasAuthority(String),
    /// `hasAnyRole('X', 'Y', ...)` check.
    HasAnyRole(Vec<String>),
    /// Always true.
    PermitAll,
    /// Always false.
    DenyAll,
    /// Logical AND.
    And(Box<SpelExpr>, Box<SpelExpr>),
    /// Logical OR.
    Or(Box<SpelExpr>, Box<SpelExpr>),
    /// Logical NOT.
    Not(Box<SpelExpr>),
    /// Comparison with operator.
    Compare(Box<SpelExpr>, CmpOp, Box<SpelExpr>),
    /// Variable reference (`#name`).
    Variable(String),
    /// Boolean literal.
    LiteralBool(bool),
    /// Number literal.
    LiteralNumber(f64),
    /// String literal.
    LiteralString(String),
}

impl fmt::Display for SpelExpr
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
    {
        match self
        {
            Self::HasRole(r) => write!(f, "hasRole('{r}')"),
            Self::HasAuthority(a) => write!(f, "hasAuthority('{a}')"),
            Self::HasAnyRole(rs) =>
            {
                let args: Vec<String> = rs.iter().map(|r| format!("'{r}'")).collect();
                write!(f, "hasAnyRole({})", args.join(", "))
            },
            Self::PermitAll => write!(f, "permitAll"),
            Self::DenyAll => write!(f, "denyAll"),
            Self::And(a, b) => write!(f, "({a} and {b})"),
            Self::Or(a, b) => write!(f, "({a} or {b})"),
            Self::Not(e) => write!(f, "not {e}"),
            Self::Compare(l, op, r) =>
            {
                let s = match op
                {
                    CmpOp::Eq => "==",
                    CmpOp::NotEq => "!=",
                    CmpOp::Gt => ">",
                    CmpOp::Lt => "<",
                    CmpOp::GtEq => ">=",
                    CmpOp::LtEq => "<=",
                };
                write!(f, "{l} {s} {r}")
            },
            Self::Variable(n) => write!(f, "#{n}"),
            Self::LiteralBool(b) => write!(f, "{b}"),
            Self::LiteralNumber(n) => write!(f, "{n}"),
            Self::LiteralString(s) => write!(f, "'{s}'"),
        }
    }
}

// ============================================================
// Tokenizer / 词法分析
// ============================================================

#[derive(Debug, Clone)]
enum Token
{
    Ident(String),
    StringLit(String),
    Number(f64),
    Variable(String),
    LParen,
    RParen,
    Comma,
    Eq,
    NotEq,
    Gt,
    Lt,
    GtEq,
    LtEq,
    Bang,
}

#[allow(clippy::indexing_slicing)]
fn tokenize(input: &str) -> Result<Vec<Token>, SpelError>
{
    let chars: Vec<char> = input.chars().collect();
    let mut tokens = Vec::new();
    let mut pos = 0;

    while pos < chars.len()
    {
        match chars[pos]
        {
            ' ' | '\t' | '\n' | '\r' => pos += 1,
            '(' =>
            {
                tokens.push(Token::LParen);
                pos += 1;
            },
            ')' =>
            {
                tokens.push(Token::RParen);
                pos += 1;
            },
            ',' =>
            {
                tokens.push(Token::Comma);
                pos += 1;
            },
            '#' =>
            {
                pos += 1;
                let mut name = String::new();
                while pos < chars.len() && (chars[pos].is_alphanumeric() || chars[pos] == '_')
                {
                    name.push(chars[pos]);
                    pos += 1;
                }
                if name.is_empty()
                {
                    return Err(SpelError::Parse("expected variable name after '#'".into()));
                }
                tokens.push(Token::Variable(name));
            },
            '\'' =>
            {
                pos += 1;
                let mut s = String::new();
                while pos < chars.len() && chars[pos] != '\''
                {
                    s.push(chars[pos]);
                    pos += 1;
                }
                if pos >= chars.len()
                {
                    return Err(SpelError::Parse("unterminated string literal".into()));
                }
                pos += 1;
                tokens.push(Token::StringLit(s));
            },
            '=' if pos + 1 < chars.len() && chars[pos + 1] == '=' =>
            {
                tokens.push(Token::Eq);
                pos += 2;
            },
            '!' if pos + 1 < chars.len() && chars[pos + 1] == '=' =>
            {
                tokens.push(Token::NotEq);
                pos += 2;
            },
            '!' =>
            {
                tokens.push(Token::Bang);
                pos += 1;
            },
            '>' if pos + 1 < chars.len() && chars[pos + 1] == '=' =>
            {
                tokens.push(Token::GtEq);
                pos += 2;
            },
            '>' =>
            {
                tokens.push(Token::Gt);
                pos += 1;
            },
            '<' if pos + 1 < chars.len() && chars[pos + 1] == '=' =>
            {
                tokens.push(Token::LtEq);
                pos += 2;
            },
            '<' =>
            {
                tokens.push(Token::Lt);
                pos += 1;
            },
            c if c.is_ascii_digit() =>
            {
                let start = pos;
                while pos < chars.len() && (chars[pos].is_ascii_digit() || chars[pos] == '.')
                {
                    pos += 1;
                }
                let s: String = chars[start..pos].iter().collect();
                let n: f64 = s
                    .parse()
                    .map_err(|_| SpelError::Parse(format!("invalid number: {s}")))?;
                tokens.push(Token::Number(n));
            },
            c if c.is_alphabetic() || c == '_' =>
            {
                let mut s = String::new();
                while pos < chars.len() && (chars[pos].is_alphanumeric() || chars[pos] == '_')
                {
                    s.push(chars[pos]);
                    pos += 1;
                }
                tokens.push(Token::Ident(s));
            },
            c => return Err(SpelError::Parse(format!("unexpected character: '{c}'"))),
        }
    }

    Ok(tokens)
}

// ============================================================
// Recursive descent parser / 递归下降解析器
// ============================================================

struct Parser
{
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser
{
    fn new(tokens: Vec<Token>) -> Self
    {
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> Option<&Token>
    {
        self.tokens.get(self.pos)
    }

    fn advance(&mut self)
    {
        self.pos += 1;
    }

    fn parse(&mut self) -> Result<SpelExpr, SpelError>
    {
        let expr = self.parse_or()?;
        if self.pos < self.tokens.len()
        {
            return Err(SpelError::Parse("unexpected tokens after expression".into()));
        }
        Ok(expr)
    }

    // or = and ("or" and)*
    fn parse_or(&mut self) -> Result<SpelExpr, SpelError>
    {
        let mut left = self.parse_and()?;
        while matches!(self.peek(), Some(Token::Ident(s)) if s == "or")
        {
            self.advance();
            let right = self.parse_and()?;
            left = SpelExpr::Or(Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    // and = cmp ("and" cmp)*
    fn parse_and(&mut self) -> Result<SpelExpr, SpelError>
    {
        let mut left = self.parse_cmp()?;
        while matches!(self.peek(), Some(Token::Ident(s)) if s == "and")
        {
            self.advance();
            let right = self.parse_cmp()?;
            left = SpelExpr::And(Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    // cmp = not (comp_op not)?
    fn parse_cmp(&mut self) -> Result<SpelExpr, SpelError>
    {
        let left = self.parse_not()?;
        let op = match self.peek()
        {
            Some(Token::Eq) => Some(CmpOp::Eq),
            Some(Token::NotEq) => Some(CmpOp::NotEq),
            Some(Token::Gt) => Some(CmpOp::Gt),
            Some(Token::Lt) => Some(CmpOp::Lt),
            Some(Token::GtEq) => Some(CmpOp::GtEq),
            Some(Token::LtEq) => Some(CmpOp::LtEq),
            _ => None,
        };
        if let Some(op) = op
        {
            self.advance();
            let right = self.parse_not()?;
            Ok(SpelExpr::Compare(Box::new(left), op, Box::new(right)))
        }
        else
        {
            Ok(left)
        }
    }

    // not = "not" not | "!" not | primary
    fn parse_not(&mut self) -> Result<SpelExpr, SpelError>
    {
        match self.peek()
        {
            Some(Token::Bang) =>
            {
                self.advance();
                let expr = self.parse_not()?;
                Ok(SpelExpr::Not(Box::new(expr)))
            },
            Some(Token::Ident(s)) if s == "not" =>
            {
                self.advance();
                let expr = self.parse_not()?;
                Ok(SpelExpr::Not(Box::new(expr)))
            },
            _ => self.parse_primary(),
        }
    }

    // primary = "(" expr ")" | function_call | variable | literal
    fn parse_primary(&mut self) -> Result<SpelExpr, SpelError>
    {
        match self.peek().cloned()
        {
            Some(Token::LParen) =>
            {
                self.advance();
                let expr = self.parse_or()?;
                self.expect_rparen()?;
                Ok(expr)
            },
            Some(Token::Variable(name)) =>
            {
                self.advance();
                Ok(SpelExpr::Variable(name))
            },
            Some(Token::StringLit(s)) =>
            {
                self.advance();
                Ok(SpelExpr::LiteralString(s))
            },
            Some(Token::Number(n)) =>
            {
                self.advance();
                Ok(SpelExpr::LiteralNumber(n))
            },
            Some(Token::Ident(ident)) =>
            {
                self.advance();
                match ident.as_str()
                {
                    "hasRole" => self.parse_single_arg_fn(SpelExpr::HasRole),
                    "hasAuthority" => self.parse_single_arg_fn(SpelExpr::HasAuthority),
                    "hasAnyRole" => self.parse_has_any_role(),
                    "permitAll" => Ok(SpelExpr::PermitAll),
                    "denyAll" => Ok(SpelExpr::DenyAll),
                    "true" => Ok(SpelExpr::LiteralBool(true)),
                    "false" => Ok(SpelExpr::LiteralBool(false)),
                    other => Err(SpelError::Parse(format!("unknown identifier: '{other}'"))),
                }
            },
            _ => Err(SpelError::Parse("unexpected end of expression".into())),
        }
    }

    fn parse_single_arg_fn(
        &mut self,
        ctor: impl Fn(String) -> SpelExpr,
    ) -> Result<SpelExpr, SpelError>
    {
        self.expect_lparen()?;
        let arg = self.expect_string_lit()?;
        self.expect_rparen()?;
        Ok(ctor(arg))
    }

    fn parse_has_any_role(&mut self) -> Result<SpelExpr, SpelError>
    {
        self.expect_lparen()?;
        let mut roles = vec![self.expect_string_lit()?];
        while matches!(self.peek(), Some(Token::Comma))
        {
            self.advance();
            roles.push(self.expect_string_lit()?);
        }
        self.expect_rparen()?;
        Ok(SpelExpr::HasAnyRole(roles))
    }

    fn expect_lparen(&mut self) -> Result<(), SpelError>
    {
        match self.peek()
        {
            Some(Token::LParen) =>
            {
                self.advance();
                Ok(())
            },
            _ => Err(SpelError::Parse("expected '('".into())),
        }
    }

    fn expect_rparen(&mut self) -> Result<(), SpelError>
    {
        match self.peek()
        {
            Some(Token::RParen) =>
            {
                self.advance();
                Ok(())
            },
            _ => Err(SpelError::Parse("expected ')'".into())),
        }
    }

    fn expect_string_lit(&mut self) -> Result<String, SpelError>
    {
        match self.peek().cloned()
        {
            Some(Token::StringLit(s)) =>
            {
                self.advance();
                Ok(s)
            },
            _ => Err(SpelError::Parse("expected string literal".into())),
        }
    }
}

/// Parse a SpEL expression string into an AST.
/// 将 SpEL 表达式字符串解析为 AST。
pub(crate) fn parse(input: &str) -> Result<SpelExpr, SpelError>
{
    let tokens = tokenize(input)?;
    let mut parser = Parser::new(tokens);
    parser.parse()
}
