use std::io::{Result, Write};

use analysis;
use analysis::upcasts::Upcasts;
use env::Env;
use super::function_body::Builder;
use super::general::tabs;
use super::parameter::ToParameter;
use super::return_value::ToReturnValue;
use super::translate_from_glib::TranslateFromGlib;
use super::translate_to_glib::TranslateToGlib;

pub fn generate<W: Write>(w: &mut W, env: &Env, analysis: &analysis::functions::Info,
    in_trait: bool, only_declaration: bool, indent: i32) -> Result<()> {

    let comment_prefix = if analysis.comented { "//" } else { "" };
    let pub_prefix = if in_trait { "" } else { "pub " };
    let declaration = declaration(env, analysis);
    let suffix = if only_declaration { ";" } else { " {" };

    try!(writeln!(w, "{}{}{}{}{}", tabs(indent),
        comment_prefix, pub_prefix, declaration, suffix));

    if !only_declaration {
        if analysis.comented {
            try!(writeln!(w, "{}//{}unsafe {{ TODO: call ffi:{}() }}",
                tabs(indent), tabs(1), analysis.glib_name));
            try!(writeln!(w, "{}//}}", tabs(indent)));
        }
        else {
            let body = body(env, analysis, in_trait);
            for s in body {
                try!(writeln!(w, "{}{}", tabs(indent + 1), s));
            }
            try!(writeln!(w, "{}}}", tabs(indent)));
        }
    }

    Ok(())
}

pub fn declaration(env: &Env, analysis: &analysis::functions::Info) -> String {
    let return_str = analysis.ret.to_return_value(env, analysis);
    let mut param_str = String::with_capacity(100);

    let upcasts = upcasts(&analysis.upcasts);

    for (pos, par) in analysis.parameters.iter().enumerate() {
        if pos > 0 { param_str.push_str(", ") }
        let s = par.to_parameter(env, &analysis.upcasts);
        param_str.push_str(&s);
    }

    format!("fn {}{}({}){}", analysis.name, upcasts, param_str, return_str)
}

fn upcasts(upcasts: &Upcasts) -> String {
    if upcasts.is_empty() { return String::new() }
    let strs: Vec<String> = upcasts.iter()
        .map(|upcast| { format!("{}: Upcast<{}>", upcast.1, upcast.2)})
        .collect();
    format!("<{}>", strs.connect(", "))
}

pub fn body(env: &Env, analysis: &analysis::functions::Info,
    in_trait: bool) -> Vec<String> {
    let mut builder = Builder::new();
    builder.glib_name(&analysis.glib_name)
        .from_glib(analysis.ret.translate_from_glib_as_function(env, &analysis));

    //TODO: change to map on parameters with pass Vec<String> to builder
    for par in &analysis.parameters {
        let s = par.translate_to_glib(&env.library, in_trait);
        builder.parameter(s);
    }

    builder.generate()
}