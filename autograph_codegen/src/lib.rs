#![crate_type = "dylib"]
#![feature(quote, concat_idents, plugin_registrar, rustc_private, box_syntax)]
#![feature(custom_attribute)]
#![allow(unused_attributes)]
#![allow(deprecated)]

#[macro_use]
extern crate log;
#[macro_use]
extern crate rustc;
//#[macro_use] extern crate quote;
extern crate syntax;
extern crate syntax_ext;
extern crate syntax_pos;
extern crate rustc_plugin;

use syntax::codemap::{Span, DUMMY_SP};
use syntax::tokenstream::{TokenStream, TokenTree};
use syntax::ast::{Attribute, Block, Expr, FnDecl, FunctionRetTy, Ident, Item, Path, Ty};
use syntax::ext::base::{DummyResult, ExtCtxt, MacEager, MacResult};
use syntax::ext::quote;
use syntax::print::pprust;
use syntax::parse::parser::Parser;
use syntax::parse::token::{DelimToken, Token};
use syntax::parse::common::SeqSep;
use syntax::parse::PResult;
use syntax::ptr::P;
use syntax::util::small_vector::SmallVector;
use rustc_plugin::Registry;

#[plugin_registrar]
pub fn plugin_registrar(reg: &mut Registry) {
    reg.register_macro("gfx_pass", expand_gfx_pass);
}

fn mk_ident(str: &str) -> Token {
    Token::Ident(Ident::from_str(str))
}

#[derive(Debug)]
enum Declarator {
    Texture,
    Texture2D,
    Texture3D,
    Buffer,
    BufferTy(P<Ty>),
}

impl Declarator {
    // TODO: take attributes into account to determine the type exposed to `execute`
    // Return a Vec<TokenTree>
    pub fn alloc_type(&self, cx: &ExtCtxt) -> Vec<TokenTree> {
        match self {
            &Declarator::Texture | &Declarator::Texture2D | &Declarator::Texture3D => {
                quote_tokens!{cx, &'a Rc<::autograph::gfx::Texture> }
            }
            &Declarator::Buffer | &Declarator::BufferTy(_) => {
                quote_tokens! {cx, &'a Rc<::autograph::gfx::Buffer> }
            }
        }
    }

    pub fn desc_type(&self, cx: &ExtCtxt) -> Vec<TokenTree> {
        match self {
            // Those types are defined in macro_prelude.rs
            &Declarator::Texture => quote_tokens! {cx, TextureInit },
            &Declarator::Texture2D => quote_tokens! {cx, Texture2DInit },
            &Declarator::Texture3D => unimplemented!(),
            &Declarator::Buffer | &Declarator::BufferTy(_) => quote_tokens! {cx, BufferInit },
        }
    }
}

#[derive(Copy, Clone, Debug)]
enum PassInputOutputKind {
    Read,
    Write,
    Create,
}

#[derive(Debug)]
struct PassInputOutputItem {
    attr: Vec<Attribute>,
    kind: PassInputOutputKind,
    decl: Declarator,
    name: Ident,
    init: Option<Vec<TokenTree>>,
}

struct PipelineItem {
    name: Ident,
    static_name: Ident,
    init: Vec<TokenTree>,
}

struct Pass {
    name: Ident,
    read_items: Vec<PassInputOutputItem>,
    write_items: Vec<PassInputOutputItem>,
    create_items: Vec<PassInputOutputItem>,
    execute_block: P<Block>,
    pipelines: Vec<PipelineItem>,
    sig: FnDecl,
}

/// Generates the `ExecuteContext` struct that is visible in the `execute` block
/// of the pass: this contains references to all resources accessible by the pass
/*fn gen_pass_execute_context(cx: &mut ExtCtxt, pass: &Pass) -> P<Item> {
    let read_items = &pass.read_items;
    let write_items = &pass.write_items;
    let create_items = &pass.create_items;
    let sig = &pass.sig;

    let mut ctx_tokens = Vec::new();
    {
        let mut gen_ctx_item = |item: &PassInputOutputItem| {
            let name = &item.name;
            let ty = item.decl.alloc_type(cx);
            ctx_tokens.append(&mut quote_tokens!(cx, $name: $ty,));
        };
        for item in read_items.iter() {
            gen_ctx_item(item);
        }
        for item in write_items.iter() {
            gen_ctx_item(item);
        }
        for item in create_items.iter() {
            gen_ctx_item(item);
        }
    }

    let item = quote_item!(cx,
        struct ExecuteContext<'a> {
            $ctx_tokens
        }
    ).unwrap();
    //warn!("ITEM= {}", pprust::item_to_string(&item));
    item
}*/


/// Generates a struct literal that corresponds to a description of a pass I/O item
fn gen_input_output_desc_struct(cx: &mut ExtCtxt, item: &PassInputOutputItem) -> P<Expr> {
    let desc_ty_tokens = item.decl.desc_type(cx);
    let init_tokens = item.init.as_ref().unwrap();

    quote_expr!(cx, $desc_ty_tokens {
        $init_tokens
    })
}

/// Generates code that creates a framebuffer according to `#[framebuffer(name,attachement)]` attributes
fn gen_framebuffer_declarations()
{
    // TODO
}

fn gen_execute_closure(cx: &mut ExtCtxt, pass: &Pass) -> P<Expr> {
    let mut stmts = Vec::new();

    for item in pass.read_items
        .iter()
        .chain(pass.write_items.iter())
        .chain(pass.create_items.iter())
    {
        let name = &item.name;
        match &item.decl {
            &Declarator::Buffer | &Declarator::BufferTy(_) => {
                // unwrapping like it's christmas
                stmts.push(quote_stmt!(cx, let $name = alloc_as_buffer_slice(__cg.get_alloc_for_resource($name).unwrap()).unwrap();));
            },
            &Declarator::Texture | &Declarator::Texture2D | &Declarator::Texture3D => {
                stmts.push(quote_stmt!(cx, let $name = alloc_as_texture(__cg.get_alloc_for_resource($name).unwrap()).unwrap();));
            }
        };
    }

    // pipeline initialization code
    for pp in pass.pipelines.iter() {
        let name = &pp.name;
        let static_name = &pp.static_name;
        let init = &pp.init;
        stmts.push(quote_stmt!(cx, let $name = $static_name.get(|| { GraphicsPipelineInit {$init}.to_graphics_pipeline(__frame.queue().context()) });));
    }

    let execute_block = &pass.execute_block;

    quote_expr!(cx, move |__frame: &gfx::Frame, __cg: &CompiledGraph| {
        $stmts
        $execute_block
    })
}

/// Generate the pipeline initialization items
fn gen_pipeline_items(cx: &mut ExtCtxt, pass: &Pass) -> Vec<P<Item>>
{
    let mut items = Vec::new();
    for pp in pass.pipelines.iter() {
        let static_name = &pp.static_name;
        items.push(quote_item!(cx, pub static $static_name: Lazy<::std::sync::Arc<::autograph::gfx::GraphicsPipeline>> = Lazy::new(); ).unwrap());
    }

    items
}

/// Generate the constructor function `create(...)` for the pass module
fn gen_pass_ctor(cx: &mut ExtCtxt, pass: &Pass) -> P<Item> {
    let pass_name = &pass.name;
    use syntax::ext::quote::rt::ToTokens;
    let mut args_tokens = Vec::new();
    // frst argument is a mut ref to the framegraph
    args_tokens.append(&mut quote_tokens!(cx, __fg: &mut FrameGraph<'pass>,));
    // paste pass arguments
    for arg in pass.sig.inputs.iter() {
        args_tokens.append(&mut arg.to_tokens(cx));
        args_tokens.push(TokenTree::Token(DUMMY_SP, Token::Comma));
    }
    // paste inputs
    if !pass.read_items.is_empty() || !pass.write_items.is_empty() {
        args_tokens.append(&mut quote_tokens!(cx, inputs: Inputs));
    }

    // Read-write item desc
    let mut stmts = Vec::new();

    for item in pass.read_items.iter().chain(pass.write_items.iter()) {
        let name = &item.name;
        //let name_str = (&name.name.as_str() as &str).to_owned();
        //let name_ri = TokenTree::Token(DUMMY_SP, mk_ident(&(name_str + "_resource_info")));
        let desc_ty = item.decl.desc_type(cx);
        stmts.push(quote_stmt!(cx, let $name = $desc_ty::from_resource_info(__fg.get_resource_info(inputs.$name).expect("input was not a resource node")).expect("unexpected resource type");));
    }

    // Created output items desc initializers
    for item in pass.create_items.iter() {
        let name = &item.name;
        let structlit = gen_input_output_desc_struct(cx, item);
        stmts.push(quote_stmt!(cx, let mut $name = $structlit;));
    }

    // TODO put validation block


    // Create new resource nodes
    // __fg.create_resource_node($name.to_resource_info());
    for item in pass.create_items.iter() {
        let name = &item.name;
        stmts.push(quote_stmt!(cx, let $name = __fg.create_resource_node(stringify!($name).to_owned(), $name.to_resource_info());));
    }

    // Clone output nodes
    for item in pass.write_items.iter() {
        let name = &item.name;
        stmts.push(
            quote_stmt!(cx, let $name = __fg.clone_resource_node(inputs.$name);),
        );
    }

    // make read inputs visible in context
    for item in pass.read_items.iter() {
        let name = &item.name;
        stmts.push(quote_stmt!(cx, let $name = inputs.$name;));
    }


    // create execute closure
    let execute_closure = gen_execute_closure(cx, pass);
    stmts.push(quote_stmt!(cx, let __exec = Box::new($execute_closure); ));
    // create pass node
    stmts.push(quote_stmt!(cx, let __pass = __fg.create_pass_node(stringify!($pass_name).to_owned(), __exec); ));

    // link dependencies
    for item in pass.read_items.iter().chain(pass.write_items.iter()) {
        let name = &item.name;
        stmts.push(
            quote_stmt!(cx, __fg.link_input(__pass, inputs.$name, ResourceUsage::Default);),
        );
    }
    for item in pass.write_items.iter() {
        let name = &item.name;
        stmts.push(
            quote_stmt!(cx, __fg.link_output(__pass, $name, ResourceUsage::Default);),
        );
    }
    for item in pass.create_items.iter() {
        let name = &item.name;
        stmts.push(
            quote_stmt!(cx, __fg.link_output(__pass, $name, ResourceUsage::Default);),
        );
    }

    // Create outputs
    let mut output_tokens = Vec::new();
    for item in pass.write_items.iter().chain(pass.create_items.iter()) {
        let name = &item.name;
        output_tokens.push(quote_tokens!(cx, $name,));
    }


    let ctor_item = quote_item!(cx,
        pub fn create<'pass>($args_tokens) -> Outputs {
            $stmts
            Outputs {
                $output_tokens
            }
        }
    ).unwrap();
    ctor_item
}

///
/// Generates the structs `Pass::Inputs` and `Pass::Outputs` for use in ctor
fn gen_pass_input_output_structs(cx: &mut ExtCtxt, pass: &Pass) -> (P<Item>, P<Item>) {
    let mut tokens_input = Vec::new();
    let mut tokens_output = Vec::new();

    // $name: NodeIndex
    // TODO: typed inputs?
    {
        let mut gen_item = |item: &PassInputOutputItem| {
            let name = &item.name;
            tokens_input.append(&mut quote_tokens!(cx, pub $name: NodeIndex,));
        };
        for item in pass.read_items.iter() {
            gen_item(item);
        }
        for item in pass.write_items.iter() {
            gen_item(item);
        }
    }

    {
        // TODO typed outputs?
        let mut gen_item = |item: &PassInputOutputItem| {
            let name = &item.name;
            tokens_output.append(&mut quote_tokens!(cx, pub $name: NodeIndex,));
        };
        for item in pass.write_items.iter() {
            gen_item(item);
        }
        for item in pass.create_items.iter() {
            gen_item(item);
        }
    }

    let input_struct = quote_item!(cx,
        pub struct Inputs {
            $tokens_input
        }
    ).unwrap();
    let output_struct = quote_item!(cx,
        pub struct Outputs {
            $tokens_output
        }
    ).unwrap();
    (input_struct, output_struct)
}


///
/// Generates the module for a pass
fn gen_pass_module(cx: &mut ExtCtxt, pass: &Pass) -> P<Item> {
    let name = &pass.name;
    //let exec_ctx_struct_item = gen_pass_execute_context(cx, pass);
    let (input_struct_item, output_struct_item) = gen_pass_input_output_structs(cx, pass);
    let ctor_fn_item = gen_pass_ctor(cx, pass);
    let pp_items = gen_pipeline_items(cx, pass);

    let mod_item = quote_item!(cx,
        pub mod $name {
            use super::*;
            // TODO clean this up
            use std::sync::Arc;
            use ::autograph::gfx::TextureFormat::*;
            use ::autograph::framegraph::{NodeIndex,ResourceUsage,FrameGraph,CompiledGraph};
            use ::autograph::framegraph::macro_prelude::*;
            use ::autograph::lazy::Lazy;
            //$exec_ctx_struct_item
            $pp_items
            $input_struct_item
            $output_struct_item
            $ctor_fn_item
        }
    ).unwrap();
    warn!("MOD= {}", pprust::item_to_string(&mod_item));

    mod_item
}

///
/// Parse an 'input-output item': an item declaring either:
/// - a read dependency (in the `read {}` block)
/// - a write dependency (in the `write {}` block)
/// - a created resource (in the `create {}` block)
fn parse_input_output_item<'a>(
    cx: &mut ExtCtxt,
    p: &mut Parser<'a>,
    kind: PassInputOutputKind,
) -> PResult<'a, PassInputOutputItem> {
    let attr = p.parse_outer_attributes()?;

    let decl = if p.eat(&mk_ident("texture")) {
        Declarator::Texture
    } else if p.eat(&mk_ident("texture2D")) {
        Declarator::Texture2D
    } else if p.eat(&mk_ident("texture3D")) {
        Declarator::Texture3D
    } else if p.eat(&mk_ident("buffer")) {
        // optional type parameter
        if p.eat(&Token::Lt) {
            let ty = p.parse_ty()?;
            p.expect(&Token::Gt)?;
            Declarator::BufferTy(ty)
        } else {
            Declarator::Buffer
        }
    } else {
        return p.unexpected();
    };

    let name = p.parse_ident()?;

    // optional initializer (struct expression body)
    let init = if p.eat(&Token::OpenDelim(DelimToken::Brace)) {
        let tokens = p.parse_tokens();
        p.expect(&Token::CloseDelim(DelimToken::Brace))?;
        Some(tokens.trees().collect())
    } else {
        None
    };

    Ok(PassInputOutputItem {
        attr,
        kind,
        decl,
        name,
        init,
    })
}

/// Parse an 'input-output block' (`read{}`, `write{}`, or `create{}`)
fn parse_input_output_block<'a>(
    cx: &mut ExtCtxt,
    p: &mut Parser<'a>,
    kind: PassInputOutputKind,
) -> PResult<'a, Vec<PassInputOutputItem>> {
    p.expect(&Token::OpenDelim(DelimToken::Brace))?;
    let items = p.parse_seq_to_end(&Token::CloseDelim(DelimToken::Brace), SeqSep::trailing_allowed(Token::Comma),
        |p| parse_input_output_item(cx, p, kind)
    )?;

    Ok(items)
}

/// Parse a pipeline block containing info for loading a pipeline from a file
fn parse_pipeline_block<'a>(
    cx: &mut ExtCtxt,
    p: &mut Parser<'a>,
) -> PResult<'a, PipelineItem>
{
    let name = p.parse_ident()?;
    p.expect(&Token::OpenDelim(DelimToken::Brace))?;
    let init = p.parse_tokens().trees().collect();
    p.expect(&Token::CloseDelim(DelimToken::Brace))?;

    let name_str = (&name.name.as_str() as &str).to_owned();
    let static_name = Ident::from_str(&(name_str + "_static__"));

    Ok(PipelineItem {
        name,
        static_name,
        init
    })
}

fn parse_gfx_pass<'a>(cx: &mut ExtCtxt, p: &mut Parser<'a>, sp: Span) -> PResult<'a, Pass> {
    p.expect(&mk_ident("pass"))?;

    let name = p.parse_ident()?;
    // parse an argument list + return type
    let sig = p.parse_fn_decl(false)?.unwrap();
    match sig.output {
        FunctionRetTy::Default(_) => {}
        FunctionRetTy::Ty(_) => {
            p.span_err(p.prev_span, "A return type cannot be specified here");
        }
    }
    // now match a token tree
    p.expect(&Token::OpenDelim(DelimToken::Brace))?;

    let mut read_items = Vec::new();
    let mut write_items = Vec::new();
    let mut create_items = Vec::new();
    let mut execute_block = None;
    let mut pipelines = Vec::new();

    'parse: loop {
        if p.eat(&mk_ident("read")) {
            read_items.append(&mut parse_input_output_block(
                cx,
                p,
                PassInputOutputKind::Read,
            )?);
        //warn!("READ BLOCK {:#?}", read_items);
        } else if p.eat(&mk_ident("write")) {
            write_items.append(&mut parse_input_output_block(
                cx,
                p,
                PassInputOutputKind::Write,
            )?);
        //warn!("WRITE BLOCK {:#?}", write_items);
        } else if p.eat(&mk_ident("create")) {
            create_items.append(&mut parse_input_output_block(
                cx,
                p,
                PassInputOutputKind::Create,
            )?);
        //warn!("CREATE BLOCK {:#?}", create_items);
        } else if p.eat(&mk_ident("validate")) {
            //warn!("VALIDATE BLOCK");
            p.parse_block()?;
        } else if p.eat(&mk_ident("execute")) {
            // warn!("EXECUTE BLOCK");
            execute_block = Some(p.parse_block()?);
        } else if p.eat(&mk_ident("pipeline")) {
            //warn!("PIPELINE BLOCK");
            pipelines.push(parse_pipeline_block(cx, p)?);
        } else if p.eat(&Token::CloseDelim(DelimToken::Brace)) {
            //warn!("END PASS");
            break 'parse;
        } else {
            return p.unexpected();
        };

        //parse_io_block(cx, p)?;
        //p.bump();
    }

    Ok(Pass {
        read_items,
        write_items,
        create_items,
        execute_block: execute_block
            .ok_or_else(|| {
                p.span_fatal(sp, "A pipeline definition must contain an `execute` block")
            })?,
        pipelines,
        name,
        sig,
    })
}

fn expand_gfx_pass(cx: &mut ExtCtxt, sp: Span, args: &[TokenTree]) -> Box<MacResult + 'static> {
    // pass <ident> <params>
    // params:
    let mut parser = cx.new_parser_from_tts(args);
    let r = parse_gfx_pass(cx, &mut parser, sp);
    match r {
        Err(mut e) => {
            // Parse error, emit and return dummy result
            e.emit();
            DummyResult::any(sp)
        }
        Ok(ref pass) => {
            let mod_item = gen_pass_module(cx, pass);
            //warn!("EXECUTE CONTEXT {:#?}", exec_ctx_struct);
            MacEager::items(SmallVector::one(mod_item))
        }
    }
}
