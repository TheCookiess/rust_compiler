use crate::lexer::*;
const LOG_DEBUG_INFO: bool = true;

#[derive(Debug, PartialEq)]
pub struct NodeProg {
    pub stmts: Vec<NodeStmt>,
}

#[derive(Debug, PartialEq)]
pub struct NodeScope {
    pub stmts: Vec<NodeStmt>,
    pub inherits_stms: bool,
}

// #[derive(Debug, PartialEq)]
// pub enum NodeScope {
//     Exclusive(Vec<NodeStmt>),
//     Inclusive(Vec<NodeStmt>),
// }

#[derive(Debug, PartialEq)]
pub enum NodeStmt {
    Exit(NodeExpr),
    Let(Token, NodeExpr),
    Scope(NodeScope),
    If(NodeExpr, NodeScope),
}

#[derive(Debug, PartialEq)]
pub enum NodeExpr {
    Term(Box<NodeTerm>),
    BinExpr(Box<NodeBinExpr>),
    BoolExpr(Box<NodeBoolExpr>),
}

#[derive(Debug, PartialEq)]
pub enum NodeTerm {
    Ident(Token),
    IntLit(Token),
    Paren(NodeExpr),
}

#[derive(Debug, PartialEq)]
pub enum NodeBinExpr {
    Divide(NodeExpr, NodeExpr),
    Multiply(NodeExpr, NodeExpr),
    Subtract(NodeExpr, NodeExpr),
    Add(NodeExpr, NodeExpr),
}

#[derive(Debug, PartialEq)]
pub enum NodeBoolExpr {
    Equal(NodeExpr, NodeExpr),
}

pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

impl Parser {
    pub fn new(input: Vec<Token>) -> Parser {
        let parser = Parser {
            tokens: input,
            position: 0,
        };
        return parser;
    }

    pub fn parse_prog(&mut self) -> Result<NodeProg, &'static str> {
        let mut prog = NodeProg { stmts: vec![] };
        while self.peek(0).is_some() {
            prog.stmts.push(self.parse_stmt()?);
        }

        return Ok(prog);
    }

    fn parse_stmt(&mut self) -> Result<NodeStmt, &'static str> {
        if LOG_DEBUG_INFO {
            println!("\nparsing statement: {:?}", self.peek(0).unwrap());
        }

        let stmt = match self.peek(0).unwrap().kind {
            TokenKind::KeywordExit => {
                self.try_consume(TokenKind::KeywordExit)?;
                self.try_consume(TokenKind::OpenParen)?;
                let stmt = NodeStmt::Exit(self.parse_expr(0)?);
                self.try_consume(TokenKind::CloseParen)?;

                stmt
            }
            TokenKind::KeywordLet => {
                self.try_consume(TokenKind::KeywordLet)?;
                let ident = self.try_consume(TokenKind::Ident)?;
                self.try_consume(TokenKind::Assign)?;

                NodeStmt::Let(ident, self.parse_expr(0)?)
            }
            TokenKind::KeywordIf => {
                self.try_consume(TokenKind::KeywordIf)?;
                let expr = self.parse_expr(0)?;
                let scope = self.parse_scope()?;

                NodeStmt::If(expr, scope)
            }
            TokenKind::OpenSquirly => NodeStmt::Scope(self.parse_scope()?),
            _ => return Err("Unable to parse statement"),
        };

        // statments that do/don't require a ';' to end.
        match stmt {
            NodeStmt::Scope(_) => false,
            _ => self.try_consume(TokenKind::SemiColon).is_ok(),
        };
        return Ok(stmt);
    }

    fn parse_scope(&mut self) -> Result<NodeScope, &'static str> {
        self.try_consume(TokenKind::OpenSquirly)?;
        let mut stmts = Vec::new();
        // while not end of scope, will shit itself in parse_stmt if no CloseSquirly
        while self.token_equals(TokenKind::CloseSquirly, 0).is_err() {
            stmts.push(self.parse_stmt()?);
        }
        self.try_consume(TokenKind::CloseSquirly)?;
        return Ok(NodeScope {
            stmts,
            inherits_stms: true,
        });
    }

    // parse binary expression
    // if next token is boolean operator
    // .. create NodeExpr::BoolExpr
    // .. parse rhs
    // else
    // .. create BinExpr
    fn parse_expr(&mut self, min_prec: i32) -> Result<NodeExpr, &'static str> {
        let term = self.parse_term()?;
        if self.peek(0).is_none() {
            return Err("No expression to parse");
        }

        let lhs = NodeExpr::Term(Box::new(term));
        self.parse_bin_expr(lhs, min_prec)?;

        return match &self.peek(0).unwrap().kind {
            // TokenKind::Divide | TokenKind::Multiply | TokenKind::Subtract | TokenKind::Add => {
            // }
            TokenKind::Equal
            | TokenKind::GreaterThan
            | TokenKind::LessThan
            | TokenKind::GreaterEqual
            | TokenKind::LessEqual => self.parse_bool_comp(lhs),
            _ => Ok(lhs),
        };
    }

    fn parse_bin_expr(
        &mut self,
        mut lhs: NodeExpr,
        min_prec: i32,
    ) -> Result<NodeExpr, &'static str> {
        loop {
            let prec = match self.peek(0).unwrap().kind {
                TokenKind::Divide | TokenKind::Multiply => 1,
                TokenKind::Subtract | TokenKind::Add => 0,
                _ => break,
            };
            if prec < min_prec {
                break;
            }

            let next_prec = prec + 1;
            let op = self.consume(); // consume operand, checked in match so don't check again
            let rhs = self.parse_expr(next_prec)?;

            let bin_expr = match op.kind {
                TokenKind::Divide => NodeBinExpr::Divide(lhs, rhs),
                TokenKind::Multiply => NodeBinExpr::Multiply(lhs, rhs),
                TokenKind::Subtract => NodeBinExpr::Subtract(lhs, rhs),
                TokenKind::Add => NodeBinExpr::Add(lhs, rhs),
                _ => break,
            };

            lhs = NodeExpr::BinExpr(Box::new(bin_expr)); // on a separate line for clarity..
        }
        Ok(lhs)
    }

    fn parse_bool_comp(&mut self, mut lhs: NodeExpr) -> Result<NodeExpr, &'static str> {
        println!("lhs {:#?}", lhs);
        todo!("");
    }

    fn parse_term(&mut self) -> Result<NodeTerm, &'static str> {
        if self.peek(0).is_none() {
            return Err("No term to parse.");
        }

        if LOG_DEBUG_INFO {
            println!("\nparsing term: {:?}", self.peek(0).unwrap());
        }

        return match self.peek(0).unwrap().kind {
            TokenKind::IntLit => Ok(NodeTerm::IntLit(self.consume())),
            TokenKind::Ident => Ok(NodeTerm::Ident(self.consume())),
            TokenKind::OpenParen => {
                self.try_consume(TokenKind::OpenParen)?;
                let term = NodeTerm::Paren(self.parse_expr(0)?);
                self.try_consume(TokenKind::CloseParen)?;
                Ok(term)
            }
            _ => {
                if LOG_DEBUG_INFO {
                    println!("term: '{:?}'", self.peek(0).unwrap());
                }
                Err("Unable to parse expression")
            }
        };
    }

    fn token_equals(&self, kind: TokenKind, offset: usize) -> Result<bool, &'static str> {
        if self.peek(offset).is_none() {
            return Err("no token to evaluate");
        }
        if self.peek(offset).unwrap().kind != kind {
            if LOG_DEBUG_INFO {
                println!(
                    "[COMPILER] Expected '{:?}', found '{:?}'",
                    kind,
                    self.peek(offset).unwrap()
                );
            }
            return Err("token evaluation was false");
        }
        return Ok(true);
    }

    fn peek(&self, offset: usize) -> Option<&Token> {
        return self.tokens.get(self.position + offset);
    }

    // remove item from vec? << no clone.
    fn consume(&mut self) -> Token {
        if LOG_DEBUG_INFO {
            println!("consuming: {:?}", self.peek(0).unwrap());
        }
        let i = self.position;
        self.position += 1;
        return self.tokens[i].clone(); // this works aswell, but clones, ew.
    }

    fn try_consume(&mut self, kind: TokenKind) -> Result<Token, &'static str> {
        self.token_equals(kind, 0)?;
        return Ok(self.consume());
    }
}
