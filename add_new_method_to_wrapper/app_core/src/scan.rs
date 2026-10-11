use proc_macro2::{Delimiter, TokenStream, TokenTree};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct Method {
    pub name: String,
    pub insert_offset: usize,
    pub indent: String,
}

#[derive(Debug, Clone)]
pub enum ScanError {
    Read(String),
    Tokenize(String),
    NoImpl,
}

pub fn scan_macro_core(path: &Path) -> Result<Vec<Method>, ScanError> {
    let text = std::fs::read_to_string(path).map_err(|e| ScanError::Read(e.to_string()))?;
    scan_macro_core_text(&text)
}

pub fn scan_macro_core_text(text: &str) -> Result<Vec<Method>, ScanError> {
    let tokens: TokenStream = text
        .parse()
        .map_err(|e: proc_macro2::LexError| ScanError::Tokenize(e.to_string()))?;
    let impl_body = find_impl_body(tokens).ok_or(ScanError::NoImpl)?;
    Ok(collect_methods(&impl_body, text))
}

fn find_impl_body(tokens: TokenStream) -> Option<TokenStream> {
    let toks: Vec<TokenTree> = tokens.into_iter().collect();
    find_impl_body_in(&toks)
}

fn find_impl_body_in(toks: &[TokenTree]) -> Option<TokenStream> {
    // First, look for "impl $ ident { ... }" at this level.
    let mut i = 0;
    while i + 3 < toks.len() {
        if let TokenTree::Ident(id) = &toks[i] {
            if id.to_string() == "impl" {
                let dollar = matches!(&toks[i + 1], TokenTree::Punct(p) if p.as_char() == '$');
                let ident = matches!(&toks[i + 2], TokenTree::Ident(_));
                if dollar && ident {
                    if let TokenTree::Group(g) = &toks[i + 3] {
                        if g.delimiter() == Delimiter::Brace {
                            return Some(g.stream());
                        }
                    }
                }
            }
        }
        i += 1;
    }
    // Otherwise, descend into brace groups (the macro body, then the => body).
    for tt in toks {
        if let TokenTree::Group(g) = tt {
            if g.delimiter() == Delimiter::Brace {
                let inner: Vec<TokenTree> = g.stream().into_iter().collect();
                if let Some(found) = find_impl_body_in(&inner) {
                    return Some(found);
                }
            }
        }
    }
    None
}

fn collect_methods(impl_body: &TokenStream, text: &str) -> Vec<Method> {
    let toks: Vec<TokenTree> = impl_body.clone().into_iter().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < toks.len() {
        let fn_kw = match &toks[i] {
            TokenTree::Ident(id) if id.to_string() == "fn" => id,
            _ => {
                i += 1;
                continue;
            }
        };
        let Some(TokenTree::Ident(name_id)) = toks.get(i + 1) else {
            i += 1;
            continue;
        };
        let name = name_id.to_string();

        // Walk forward to the first brace group — that's the method body.
        let mut j = i + 2;
        let mut body: Option<proc_macro2::Group> = None;
        while j < toks.len() {
            if let TokenTree::Group(g) = &toks[j] {
                if g.delimiter() == Delimiter::Brace {
                    body = Some(g.clone());
                    break;
                }
            }
            j += 1;
        }
        let Some(body) = body else { break };

        // Indent = leading whitespace of the line holding "fn". The "pub"
        // before "fn" lives on the same line, so taking the line's leading
        // spaces gives us the true indent.
        let fn_line = fn_kw.span().start().line;
        let line_text = text.lines().nth(fn_line.saturating_sub(1)).unwrap_or("");
        let indent: String = line_text.chars().take_while(|c| *c == ' ').collect();

        // insert_offset = end of the line holding the closing brace.
        let end_line = body.span().end().line.max(1);
        let mut line_start = 0usize;
        for (li, l) in text.lines().enumerate() {
            if li + 1 == end_line {
                break;
            }
            line_start += l.len() + 1;
        }
        let last_line = text
            .lines()
            .nth(end_line.saturating_sub(1))
            .unwrap_or("");
        let insert_offset = line_start + last_line.len();

        out.push(Method {
            name,
            insert_offset,
            indent,
        });
        i = j + 1;
    }
    out
}

#[derive(Debug, Clone)]
pub struct PastedMethod {
    pub name: String,
    pub input_type: Option<String>,
    pub output_type: Option<String>,
}

#[derive(Debug, Clone)]
pub enum PasteError {
    Tokenize(String),
    NotOneMethod,
    MissingSelf,
    BadInputParam(String),
    BadReturn(String),
}

pub fn parse_pasted_method(text: &str) -> Result<PastedMethod, PasteError> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Err(PasteError::NotOneMethod);
    }
    let tokens: TokenStream = trimmed
        .parse()
        .map_err(|e: proc_macro2::LexError| PasteError::Tokenize(e.to_string()))?;

    let mut iter = tokens.into_iter().peekable();

    // Skip leading attributes: #[...] before fn.
    while let Some(TokenTree::Punct(p)) = iter.peek() {
        if p.as_char() == '#' {
            iter.next();
            if let Some(TokenTree::Group(_)) = iter.peek() {
                iter.next();
            }
        } else {
            break;
        }
    }

    // Skip "pub" (and "pub(crate)" etc).
    if let Some(TokenTree::Ident(id)) = iter.peek() {
        if id.to_string() == "pub" {
            iter.next();
            if let Some(TokenTree::Group(g)) = iter.peek() {
                if g.delimiter() == Delimiter::Parenthesis {
                    iter.next();
                }
            }
        }
    }

    let Some(TokenTree::Ident(kw)) = iter.next() else {
        return Err(PasteError::NotOneMethod);
    };
    if kw.to_string() != "fn" {
        return Err(PasteError::NotOneMethod);
    }
    let Some(TokenTree::Ident(name_id)) = iter.next() else {
        return Err(PasteError::NotOneMethod);
    };
    let name = name_id.to_string();

    let Some(TokenTree::Group(args_group)) = iter.next() else {
        return Err(PasteError::NotOneMethod);
    };
    if args_group.delimiter() != Delimiter::Parenthesis {
        return Err(PasteError::NotOneMethod);
    }

    let mut args = args_group.stream().into_iter();
    let Some(TokenTree::Punct(amp)) = args.next() else {
        return Err(PasteError::MissingSelf);
    };
    if amp.as_char() != '&' {
        return Err(PasteError::MissingSelf);
    }
    let Some(TokenTree::Ident(self_id)) = args.next() else {
        return Err(PasteError::MissingSelf);
    };
    if self_id.to_string() != "self" {
        return Err(PasteError::MissingSelf);
    }

    let mut input_type: Option<String> = None;
    if args.next().is_some() {
        let Some(TokenTree::Ident(_param)) = args.next() else {
            return Err(PasteError::BadInputParam("expected a parameter name".into()));
        };
        let Some(TokenTree::Punct(colon)) = args.next() else {
            return Err(PasteError::BadInputParam("expected :".into()));
        };
        if colon.as_char() != ':' {
            return Err(PasteError::BadInputParam("expected :".into()));
        }
        let in_type = read_method_type_macro(&mut args).ok_or_else(|| {
            PasteError::BadInputParam("second parameter must be method_input_type!(IN)".into())
        })?;
        input_type = Some(in_type);
        if args.next().is_some() {
            return Err(PasteError::BadInputParam(
                "only &self and one method_input_type!(IN) parameter are allowed".into(),
            ));
        }
    }

    let mut output_type: Option<String> = None;
    if let Some(TokenTree::Punct(arrow)) = iter.peek() {
        if arrow.as_char() == '-' {
            iter.next();
            let Some(TokenTree::Punct(arrow2)) = iter.next() else {
                return Err(PasteError::BadReturn("expected ->".into()));
            };
            if arrow2.as_char() != '>' {
                return Err(PasteError::BadReturn("expected ->".into()));
            }
            let out_type = read_method_type_macro(&mut iter).ok_or_else(|| {
                PasteError::BadReturn("return type must be method_return_type!(OUT)".into())
            })?;
            output_type = Some(out_type);
        }
    }

    let Some(TokenTree::Group(body)) = iter.next() else {
        return Err(PasteError::NotOneMethod);
    };
    if body.delimiter() != Delimiter::Brace {
        return Err(PasteError::NotOneMethod);
    }
    if iter.next().is_some() {
        return Err(PasteError::NotOneMethod);
    }

    // () means no output.
    if let Some(out) = &output_type {
        if out.is_empty() {
            output_type = None;
        }
    }

    Ok(PastedMethod {
        name,
        input_type,
        output_type,
    })
}

fn read_method_type_macro(iter: &mut impl Iterator<Item = TokenTree>) -> Option<String> {
    let Some(TokenTree::Ident(id)) = iter.next() else {
        return None;
    };
    let macro_name = id.to_string();
    if macro_name != "method_input_type" && macro_name != "method_return_type" {
        return None;
    }
    let Some(TokenTree::Punct(bang)) = iter.next() else {
        return None;
    };
    if bang.as_char() != '!' {
        return None;
    }
    let Some(TokenTree::Group(g)) = iter.next() else {
        return None;
    };
    if g.delimiter() != Delimiter::Parenthesis {
        return None;
    }
    let inner: Vec<TokenTree> = g.stream().into_iter().collect();
    if inner.is_empty() {
        return Some(String::new());
    }
    // `()` — a single empty paren group — means no output.
    if inner.len() == 1 {
        if let TokenTree::Group(inner_g) = &inner[0] {
            if inner_g.delimiter() == Delimiter::Parenthesis
                && inner_g.stream().into_iter().next().is_none()
            {
                return Some(String::new());
            }
        }
    }
    let mut name: Option<String> = None;
    for tt in &inner {
        match tt {
            TokenTree::Ident(i) => name = Some(i.to_string()),
            TokenTree::Punct(p) if p.as_char() == ':' => {}
            _ => return None,
        }
    }
    name
}