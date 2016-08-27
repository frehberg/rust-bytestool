#![crate_type="dylib"]
#![feature(plugin_registrar, rustc_private)]
 #[allow(unused)]

extern crate syntax;
extern crate rustc_plugin;

use syntax::ext::base::ExtCtxt;
use syntax::codemap::Span;
use syntax::ext::build::AstBuilder;
use syntax::ast;

use syntax::parse::token::*;
use syntax::tokenstream::TokenTree;
use syntax::ext::base::{ MacResult, MacEager, DummyResult};
use rustc_plugin::Registry;
use syntax::ptr::P;

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_macro("bytesize", expand_bs);
}

fn expand_lit(cx: &mut ExtCtxt, 
    sp: Span, 
    l : &syntax::codemap::Spanned<syntax::ast::LitKind> ) 
->  Box<MacResult + 'static>
{
    let lit = &l.node;
    
    let len = match lit {
        &ast::LitKind::ByteStr(ref str) => (*str).len(),
        _ => {
            cx.span_err(sp, &format!("expecting raw string b\"..\" but got {:?}", lit));
            return DummyResult::any(sp); 
        }
    };
   
    MacEager::expr(cx.expr_usize(sp, len))    
}

fn expand_ast_expr(cx: &mut ExtCtxt, sp: Span, e: &syntax::ast::Expr) 
->  Box<MacResult + 'static>
{
    let node = &e.node;
    
    match node {
        // TODO: borrow without cloning
        &ast::ExprKind::Lit(ref l) => return expand_lit(cx, sp, &(*l).clone().unwrap() ),
         _ =>  {
             cx.span_err(sp, &format!("expecting raw string b\"..\" but got {:?}", e ));
             return DummyResult::any(sp); 
         }   
     };
}

fn expand_nonterminal(cx: &mut ExtCtxt, 
    sp: Span, 
    nt: &Nonterminal) 
->  Box<MacResult + 'static>
{
    match nt {          
        &NtExpr(ref p) => {
            // TODO: borrow without cloning
            return expand_ast_expr(cx, sp, &(*p).clone().unwrap() );
        },
        _ => {
            cx.span_err(sp, &format!("expecting raw string b\"..\" but got {:?}", nt));
            return DummyResult::any(sp); 
        }
    }
}

fn expand_literal(cx: &mut ExtCtxt, sp: Span, lit: & Lit) ->  Box<MacResult + 'static>
{
    let len = match lit {
        &Lit::ByteStr(str) => str.as_str().len(),
        _ => {
            cx.span_err(sp, &format!("expecting raw string b\"..\" but got {:?}", lit));
            return DummyResult::any(sp); 
        }
    };
   
    MacEager::expr(cx.expr_usize(sp, len))    
}


fn expand_bs(cx: &mut ExtCtxt, sp: Span, args: &[TokenTree])
        -> Box<MacResult + 'static> {
            
    let result = match &args[0] {
       &TokenTree::Token(_, Literal(ref lit,_)) => expand_literal(cx, sp, lit),
       &TokenTree::Token(_, Interpolated(ref nt)) => expand_nonterminal(cx, sp, nt), // macro
       _ => {
            cx.span_err(sp, &format!("expecting raw string b\"..\" but got {:?}", &args[0] ) );
            return DummyResult::any(sp); 
        }
    };
    
    return result;
 }