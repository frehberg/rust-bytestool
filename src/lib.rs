#![crate_type="dylib"]
#![feature(plugin_registrar, rustc_private)]
#[allow(unused)]

extern crate syntax;
extern crate rustc_plugin;

use syntax::ext::base::ExtCtxt;
use syntax::codemap::Span;
use syntax::ext::build::AstBuilder;
use syntax::ast;
// see: https://github.com/rust-lang/rfcs/pull/566

use syntax::parse::token::*;
use syntax::tokenstream::{TokenTree, Delimited};
use syntax::ext::base::{MacResult, MacEager, DummyResult};
use rustc_plugin::Registry;
use std::rc::Rc;
use syntax::ptr::P;


#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_macro("byte_size_of", bs_expand);
    reg.register_macro("concat_bytes", concat_expand);
}


fn bs_expand(cx: &mut ExtCtxt, sp: Span, args: &[TokenTree]) -> Box<MacResult + 'static> {

    if args.len() != 1 {
        cx.span_err(sp,
                    &format!("expecting single argument but got {:?}", args.len()));
        return DummyResult::any(sp);
    }

    let result = extract_vec_from_token(cx, sp, &args[0]);
    let bytevec = match result {
        Ok(bytes) => bytes,
        Err(syntax_err) => return syntax_err,
    };

    MacEager::expr(cx.expr_usize(sp, bytevec.len()))
}


fn extract_vec_from_lit(cx: &mut ExtCtxt,
                        sp: Span,
                        l: &syntax::codemap::Spanned<syntax::ast::LitKind>)
                        -> Result<Vec<u8>, Box<MacResult + 'static>> {
    let lit = &l.node;

    match lit {
        // TODO should not clone inner value
        &ast::LitKind::ByteStr(ref str) => {
            let x: &Rc<Vec<u8>> = str;
            return Ok(x.as_ref().clone());
        }
        _ => {
            cx.span_err(sp,
                        &format!("expecting raw string b\"..\" but got {:?}", lit));
            return Err(DummyResult::any(sp));
        }
    };
}


fn extract_u8_from_lit(cx: &mut ExtCtxt,
                       sp: Span,
                       l: &syntax::codemap::Spanned<syntax::ast::LitKind>)
                       -> Result<u8, Box<MacResult + 'static>> {
    let lit = &l.node;

    match lit {
        // TODO should not clone inner value
        &ast::LitKind::Int(byte, ast::LitIntType::Unsigned(ast::UintTy::U8)) => {
            return Ok(byte as u8);
        }
        _ => {
            cx.span_err(sp, &format!("expecting u8 b\"..\" but got {:?}", lit));
            return Err(DummyResult::any(sp));
        }
    };
}


fn extract_vec_from_vec(cx: &mut ExtCtxt,
                        sp: Span,
                        expr_vec: &Vec<P<syntax::ast::Expr>>)
                        -> Result<Vec<u8>, Box<MacResult + 'static>> {
    let mut result = Vec::new();

    for expr in expr_vec {
        let kind = &expr.node;
        match kind {
            // TODO should not clone inner value
            &ast::ExprKind::Lit(ref l) => {
                match extract_u8_from_lit(cx, sp, l) {
                    Ok(byte) => result.push(byte),
                    Err(err_msg) => {
                        return Err(err_msg);
                    }
                }
            }
            _ => {
                cx.span_err(sp,
                            &format!("expecting raw string b\"..\" but got {:?}", expr_vec));
                return Err(DummyResult::any(sp));
            }
        };
    }

    return Ok(result);
}

fn extract_vec_from_ast_expr(cx: &mut ExtCtxt,
                             sp: Span,
                             e: &syntax::ast::Expr)
                             -> Result<Vec<u8>, Box<MacResult + 'static>> {
    let node = &e.node;

    match node {
        // TODO: borrow without cloning
        &ast::ExprKind::Lit(ref l) => return extract_vec_from_lit(cx, sp, &(*l).clone().unwrap()),
        &ast::ExprKind::Vec(ref v) => return extract_vec_from_vec(cx, sp, &(*v).clone()),
        _ => {
            cx.span_err(sp, &format!("expecting raw string b\"..\" but got {:?}", e));
            return Err(DummyResult::any(sp));
        }
    };
}

fn extract_vec_from_nonterminal(cx: &mut ExtCtxt,
                                sp: Span,
                                nt: &Nonterminal)
                                -> Result<Vec<u8>, Box<MacResult + 'static>> {
    match nt {
        &NtExpr(ref p) => {
            // TODO: borrow without cloning
            return extract_vec_from_ast_expr(cx, sp, &(*p).clone().unwrap());
        }
        _ => {
            cx.span_err(sp,
                        &format!("expecting raw string b\"..\" but got {:?}", nt));
            return Err(DummyResult::any(sp));
        }
    }
}


fn extract_vec_from_delimited(cx: &mut ExtCtxt,
                              sp: Span,
                              del: &Delimited)
                              -> Result<Vec<u8>, Box<MacResult + 'static>> {
    let mut res = Vec::new();

    for (idx, elem) in del.tts.iter().enumerate() {
        match elem {
            &TokenTree::Token(sp, Comma) => {
                if idx % 2 == 0 {
                    cx.span_err(sp,
                                &format!("argument {}, expecting raw string b\"..\" but found \
                                          ',' ",
                                         (idx / 2)));
                    return Err(DummyResult::any(sp));
                } else {
                    continue;
                }
            }
            &TokenTree::Token(sp, Literal(ref lit, _)) => {
                match lit {
                    &Lit::Integer(intstr) => {
                        match intstr.as_str().parse::<u8>() {
                            Ok(int) => res.push(int),
                            _ => {
                                cx.span_err(sp, &format!("expecting u8 but got {:?}", intstr));
                                return Err(DummyResult::any(sp));
                            }
                        }
                    }
                    _ => {
                        cx.span_err(sp, &format!("expecting u8 but got {:?}", elem));
                        return Err(DummyResult::any(sp));
                    }
                }
            }
            _ => {
                cx.span_err(sp, &format!("expecting [u8] but got {:?}", elem));
                return Err(DummyResult::any(sp));
            }
        }
    }

    return Ok(res);
}


fn extract_vec_from_literal(cx: &mut ExtCtxt,
                            sp: Span,
                            lit: &Lit)
                            -> Result<Vec<u8>, Box<MacResult + 'static>> {
    match lit {
        &Lit::ByteStr(str) => return Ok(str.as_str().as_bytes().to_vec()),
        _ => {
            cx.span_err(sp,
                        &format!("expecting raw string b\"..\" but got {:?}", lit));
            return Err(DummyResult::any(sp));
        }
    };
}


fn extract_vec_from_token(cx: &mut ExtCtxt,
                          sp: Span,
                          token: &TokenTree)
                          -> Result<Vec<u8>, Box<MacResult + 'static>> {
    match token {
        &TokenTree::Token(sp, Literal(ref lit, _)) => return extract_vec_from_literal(cx, sp, lit),
        &TokenTree::Token(sp, Interpolated(ref nt)) => {
            return extract_vec_from_nonterminal(cx, sp, nt)
        } // macro
        &TokenTree::Delimited(sp, ref delimited) => {
            return extract_vec_from_delimited(cx, sp, delimited.as_ref())
        }
        _ => {
            cx.span_err(sp,
                        &format!("expecting raw string b\"..\" but got {:?}", token));
            return Err(DummyResult::any(sp));
        }
    }
}


fn concat_expand(cx: &mut ExtCtxt, sp: Span, args: &[TokenTree]) -> Box<MacResult + 'static> {
    let mut con: Vec<u8> = Vec::new();

    for (idx, token) in args.iter().enumerate() {
        match token {
            &TokenTree::Token(sp, Comma) => {
                if idx % 2 == 0 {
                    cx.span_err(sp,
                                &format!("argument {}, expecting raw string b\"..\" but found \
                                          ',' ",
                                         (idx / 2)));
                    return DummyResult::any(sp);
                } else {
                    continue;
                }
            }
            _ => {
                let result = extract_vec_from_token(cx, sp, token);
                match result {
                    Ok(bytes) => con.extend(bytes),
                    Err(syntax_err) => return syntax_err,
                }
            }
        }
    }
    let rc = Rc::new(con);
    return MacEager::expr(cx.expr_lit(sp, ast::LitKind::ByteStr(rc)));
}
