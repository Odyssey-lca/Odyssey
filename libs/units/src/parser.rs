use pest::{Parser, iterators::Pairs, pratt_parser::PrattParser};
use pest_derive::Parser;

use crate::unit::{UNITS, Unit};

#[derive(Debug)]
pub enum Expr {
    Atom { symbol: String, substance: Option<String> },
    Num(i32),
    Paren(Box<Expr>),
    BinOp {
        lhs: Box<Expr>,
        op: Op,
        rhs: Box<Expr>,
    },
}

#[derive(Debug)]
pub enum Op {
    Multiply,
    Divide,
    Exponent,
}

#[derive(Parser)]
#[grammar = "parser.pest"]
pub struct UnitParser;

lazy_static::lazy_static! {
    static ref PRATT_PARSER: PrattParser<Rule> = {
        use pest::pratt_parser::{Assoc::*, Op};
        use Rule::*;

        PrattParser::new()
            .op(Op::infix(multiply, Left) | Op::infix(divide, Left))
            .op(Op::infix(exponent, Left))
    };
}

pub fn parse_expr(pairs: Pairs<Rule>) -> Expr {
    PRATT_PARSER
        .map_primary(|primary| match primary.as_rule() {
            Rule::num_atom => Expr::Num(primary.as_str().parse::<i32>().unwrap()),
            Rule::qualified_unit => {
                let mut inner = primary.into_inner();
                let symbol = inner.next().unwrap().as_str().to_string();
                let substance = inner.next().map(|s| s.as_str().to_string());
                Expr::Atom { symbol, substance }
            }
            Rule::paren_atom => Expr::Paren(Box::new(parse_expr(primary.into_inner()))),
            Rule::expr => parse_expr(primary.into_inner()),
            rule => unreachable!("Expr::parse expected atom, found {:?}", rule),
        })
        .map_infix(|lhs, op, rhs| {
            let op = match op.as_rule() {
                Rule::multiply => Op::Multiply,
                Rule::divide => Op::Divide,
                Rule::exponent => {
                    assert!(matches!(rhs, Expr::Num(_)), "Oups");
                    Op::Exponent
                }
                rule => unreachable!("Expr::parse expected infix operation, found {:?}", rule),
            };
            Expr::BinOp {
                lhs: Box::new(lhs),
                op,
                rhs: Box::new(rhs),
            }
        })
        .parse(pairs)
}

fn unit_of_binop(lhs: Expr, op: Op, rhs: Expr, exponent: i32) -> Option<Unit> {
    match op {
        Op::Multiply => {
            let lhs = unit_of_expr(lhs, exponent);
            let rhs = unit_of_expr(rhs, exponent);
            match (lhs, rhs) {
                (Some(lhs), Some(rhs)) => Some(lhs * rhs),
                (_, _) => None,
            }
        }
        Op::Divide => {
            let lhs = unit_of_expr(lhs, exponent);
            let exponent = -exponent.abs(); // Ensures that a/b/c = a/(bc)
            let rhs = unit_of_expr(rhs, exponent);
            match (lhs, rhs) {
                (Some(lhs), Some(rhs)) => Some(lhs * rhs),
                (_, _) => None,
            }
        }
        Op::Exponent => {
            let lhs = unit_of_expr(lhs, exponent);
            match (lhs, rhs) {
                (Some(lhs), Expr::Num(exp)) => Some(lhs.pow(exp)),
                _ => unreachable!("Expr::parse expected infix operation, found"),
            }
        }
    }
}

fn unit_of_expr(expr: Expr, exponent: i32) -> Option<Unit> {
    match expr {
        Expr::Atom { symbol, substance } => {
            UNITS.get(symbol.as_str()).cloned().map(|u| {
                let mut unit = u.pow(exponent);
                unit.substance = substance; 
                unit
            })
        }
        Expr::BinOp { lhs, op, rhs } => unit_of_binop(*lhs, op, *rhs, exponent),
        Expr::Paren(expr) => unit_of_expr(*expr, 1).map(|u| u.pow(exponent)), // Ensures that a/(b/c) = ac/b
        Expr::Num(_) => unreachable!("Num should be parsed elswhere"),
    }
}

/// Parses a string slice to extract a physical unit.
///
/// # Examples
/// 
/// ```
/// use units::parser::parse_unit;
/// 
/// let input = "m/s";
/// 
/// if let Some(unit) = parse_unit(input) {
///     assert_eq!(unit.dimension.length, 1);
///     assert_eq!(unit.dimension.time, -1);
///     assert_eq!(unit.scale_to_si, 1.0);
/// }
/// ```
pub fn parse_unit(expr: &str) -> Option<Unit> {
    match UnitParser::parse(Rule::unit, expr) {
        Ok(mut pairs) => {
            let expr = parse_expr(pairs.next().unwrap().into_inner());
            unit_of_expr(expr, 1)
        }
        Err(_) => None,
    }
}
