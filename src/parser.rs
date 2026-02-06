use log::error;
use rootcause::prelude::*;
use std::path::PathBuf;
use typst_syntax::{
    SyntaxKind, SyntaxNode,
    ast::{Arg, AstNode, Conditional, ContentBlock, EnumItem, FuncCall, Markup, Raw},
};

// All the code in here is absolutely horrible. Many terrible things have been done where in the
// name of "getting it working ASAP." At some point in the future, it can be refactored. Of course,
// nothing is more permanent than a temporary solution so we shall see.
//
// Outstanding TODOs:
// - FuncCall -> argument spreading contents
// - Check if there are any other `SyntaxNode` types that can actually be recursed into
pub fn search_ast_tree(root: &SyntaxNode) -> Result<Vec<(PathBuf, String)>, Report> {
    let mut results = vec![];

    match root.kind() {
        SyntaxKind::Raw => {
            let casted: Raw = root
                .cast()
                .ok_or(report!("SyntaxKind didn't match true type"))?;

            let parse_result = parse_raw(&casted);
            if let Some(parse_result) = parse_result {
                results.push(parse_result);
            }
        }
        SyntaxKind::Markup => {
            let casted: Markup = root
                .cast()
                .ok_or(report!("SyntaxKind didn't match true type"))?;

            for expr in casted.exprs() {
                let mut sub_results = search_ast_tree(expr.to_untyped())?;
                results.append(&mut sub_results);
            }
        }
        SyntaxKind::FuncCall => {
            let casted: FuncCall = root
                .cast()
                .ok_or(report!("SyntaxKind didn't match true type"))?;

            for arg in casted.args().items() {
                match arg {
                    Arg::Pos(expr) => {
                        let mut sub_results = search_ast_tree(expr.to_untyped())?;
                        results.append(&mut sub_results);
                    }
                    Arg::Named(named) => {
                        let mut sub_results = search_ast_tree(named.expr().to_untyped())?;
                        results.append(&mut sub_results);
                    }

                    // TODO: handle argument spreading
                    _ => {}
                }
            }
        }
        SyntaxKind::ContentBlock => {
            let casted: ContentBlock = root
                .cast()
                .ok_or(report!("SyntaxKind didn't match true type"))?;

            let mut sub_results = search_ast_tree(casted.body().to_untyped())?;
            results.append(&mut sub_results);
        }
        SyntaxKind::EnumItem => {
            let casted: EnumItem = root
                .cast()
                .ok_or(report!("SyntaxKind didn't match true type"))?;

            let mut sub_results = search_ast_tree(casted.body().to_untyped())?;
            results.append(&mut sub_results);
        }
        SyntaxKind::Conditional => {
            let casted: Conditional = root
                .cast()
                .ok_or(report!("SyntaxKind didn't match true type"))?;

            let mut sub_results = search_ast_tree(casted.condition().to_untyped())?;
            let mut sub2_results = search_ast_tree(casted.if_body().to_untyped())?;
            let mut sub3_results = match casted.else_body() {
                Some(else_body) => search_ast_tree(else_body.to_untyped())?,
                None => vec![],
            };

            results.append(&mut sub_results);
            results.append(&mut sub2_results);
            results.append(&mut sub3_results);
        }
        _ => {}
    }

    Ok(results)
}

fn parse_raw(node: &Raw) -> Option<(PathBuf, String)> {
    if node.lang().unwrap_or_default().get().as_str() != "plantuml" {
        return None;
    }

    let first = node.lines().next().unwrap_or_default().get().as_str();
    if !first.starts_with("' ") {
        error!("Inline plantuml must specify a file name!");
        return None;
    }

    let filename = PathBuf::from(first.split_at(2).1);
    let mut string_buf = String::new();

    for line in node.lines().skip(1) {
        string_buf.push_str(line.get().as_str());
        string_buf.push('\n');
    }

    Some((filename, string_buf))
}
