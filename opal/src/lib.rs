use std::io::{Result, Write};

pub trait Template {
    fn size_hint(&self) -> usize;

    fn render<W>(&self, writer: &mut W) -> Result<()>
    where
        W: Write;

    fn render_as_string(&self) -> Result<String>
    where
        Self: Sized,
    {
        let mut buf = Vec::with_capacity(self.size_hint());

        self.render(&mut buf)?;

        // SAFETY: The buffer is built using `write` calls, and everything is already a Rust string
        Ok(unsafe { String::from_utf8_unchecked(buf) })
    }

    fn render_into_string(self) -> Result<String>
    where
        Self: Sized,
    {
        self.render_as_string()
    }
}

pub fn compile(bounds: Option<(&str, &str)>, typ: &str, text: &str) -> Result<String> {
    let tokens = parse(text);

    let mut buf = Vec::new();

    if let Some((bounds_impl, bounds_name)) = bounds {
        writeln!(
            &mut buf,
            "impl<{}> ::opal::Template for {}<{}> {{",
            bounds_impl, typ, bounds_name,
        )?;
    } else {
        writeln!(&mut buf, "impl ::opal::Template for {} {{", typ,)?;
    }

    writeln!(
        &mut buf,
        "#[allow(dead_code, unused_variables, clippy::if_same_then_else)]"
    )?;
    writeln!(&mut buf, "    fn size_hint(&self) -> usize {{")?;
    write!(&mut buf, "        let mut hint = 0;")?;
    write_size_hint(&mut buf, &tokens)?;
    write!(&mut buf, "        hint")?;
    writeln!(&mut buf, "    }}")?;

    writeln!(&mut buf, "#[allow(unused_imports)]")?;
    writeln!(&mut buf, "    fn render<W>(&self, writer: &mut W) -> ::std::io::Result<()>\n        where\n            W: ::std::io::Write,\n        {{")?;
    writeln!(
        &mut buf,
        "use {{::opal::Template as _, std::io::Write as _}};"
    )?;
    write_render(&mut buf, tokens)?;
    writeln!(&mut buf, "        Ok(())")?;
    writeln!(&mut buf, "    }}")?;

    writeln!(&mut buf, "}}")?;

    Ok(String::from_utf8(buf).unwrap())
}

fn write_render<W>(writer: &mut W, tokens: Vec<Stage4>) -> Result<()>
where
    W: Write,
{
    for token in tokens {
        match token {
            Stage4::Expr(expr) => writeln!(writer, "write!(writer, \"{{}}\", {})?;", expr)?,
            Stage4::ExprAssign(expr) => writeln!(writer, "{}", expr.trim())?,
            Stage4::ExprRender(expr) => writeln!(writer, "{}?;", expr.trim())?,
            Stage4::If(cond, if_tokens, else_tokens) => {
                writeln!(writer, "{} {{", cond)?;

                write_render(writer, if_tokens)?;

                if let Some(else_tokens) = else_tokens {
                    writeln!(writer, "}} else {{")?;

                    write_render(writer, else_tokens)?;
                }

                writeln!(writer, "}}")?;
            }
            Stage4::For(cond, tokens) => {
                writeln!(writer, "{} {{", cond)?;

                write_render(writer, tokens)?;

                writeln!(writer, "}}")?;
            }
            Stage4::Other(other) => writeln!(writer, "write!(writer, {:?})?;", other)?,
        }
    }

    Ok(())
}

fn write_size_hint<W>(writer: &mut W, tokens: &[Stage4]) -> Result<()>
where
    W: Write,
{
    for token in tokens {
        match token {
            Stage4::Expr(expr) => {
                if expr.trim() == "count" {
                    continue;
                }

                if !(expr.contains('+') || expr.contains('-') || expr.contains("len")) {
                    writeln!(writer, "hint += {}.len();", expr)?;
                }
            }
            Stage4::ExprAssign(expr) => writeln!(writer, "{}", expr.trim())?,
            Stage4::ExprRender(expr) => writeln!(
                writer,
                "hint += &{}.size_hint();",
                expr.trim().trim_end_matches(".render(writer)")
            )?,
            Stage4::If(cond, if_tokens, else_tokens) => {
                writeln!(writer, "{} {{", cond)?;

                write_size_hint(writer, if_tokens)?;

                if let Some(else_tokens) = else_tokens {
                    writeln!(writer, "}} else {{")?;

                    write_size_hint(writer, else_tokens)?;
                }

                writeln!(writer, "}}")?;
            }
            Stage4::For(cond, tokens) => {
                writeln!(writer, "{} {{", cond)?;

                write_size_hint(writer, tokens)?;

                writeln!(writer, "}}")?;
            }
            Stage4::Other(other) => writeln!(writer, "hint += {};", other.len())?,
        }
    }

    Ok(())
}

fn parse(text: &str) -> Vec<Stage4> {
    pass_4(pass_3(pass_2(pass_1(text))))
}

enum Stage1 {
    Open,
    Close,
    Other(char),
}

#[inline]
fn pass_1(text: &str) -> Vec<Stage1> {
    let mut iter = text.chars().peekable();

    let mut tokens = Vec::new();

    while let Some(c) = iter.next() {
        match c {
            '{' if iter.peek().map(|c| *c == '{').unwrap_or(false) => {
                let _c = iter.next();

                tokens.push(Stage1::Open);
            }
            '}' if iter.peek().map(|c| *c == '}').unwrap_or(false) => {
                let _c = iter.next();

                tokens.push(Stage1::Close);
            }
            c => tokens.push(Stage1::Other(c)),
        }
    }

    tokens
}

enum Stage2 {
    Open,
    Close,
    Other(String),
}

#[inline]
fn pass_2(stage_1: Vec<Stage1>) -> Vec<Stage2> {
    let mut tokens = Vec::new();

    for token in stage_1 {
        match token {
            Stage1::Open => tokens.push(Stage2::Open),
            Stage1::Close => tokens.push(Stage2::Close),
            Stage1::Other(c)
                if tokens
                    .last()
                    .map(|token| matches!(token, Stage2::Other(_)))
                    .unwrap_or(false) =>
            {
                if let Some(Stage2::Other(other)) = tokens.last_mut() {
                    other.push(c);
                }
            }
            Stage1::Other(c) => tokens.push(Stage2::Other(String::from(c))),
        }
    }

    tokens
}

enum Stage3 {
    Expr(String),
    Other(String),
}

#[inline]
fn pass_3(stage_2: Vec<Stage2>) -> Vec<Stage3> {
    let mut tokens = Vec::new();

    let mut take = false;

    for token in stage_2 {
        match token {
            Stage2::Open => take = true,
            Stage2::Close => take = false,
            Stage2::Other(other) => {
                if take {
                    tokens.push(Stage3::Expr(other));
                } else {
                    tokens.push(Stage3::Other(other));
                }
            }
        }
    }

    tokens
}

#[derive(Debug)]
pub enum Stage4 {
    Expr(String),
    ExprAssign(String),
    ExprRender(String),
    If(String, Vec<Stage4>, Option<Vec<Stage4>>),
    For(String, Vec<Stage4>),
    Other(String),
}

#[inline]
fn pass_4(stage_3: Vec<Stage3>) -> Vec<Stage4> {
    let mut iter = stage_3.into_iter();

    let mut tokens = Vec::new();

    while let Some(token) = iter.next() {
        match token {
            Stage3::Expr(expr) => {
                let trimmed_expr = expr.trim();

                if trimmed_expr.starts_with('#') {
                    let trimmed_expr = trimmed_expr.trim_start_matches('#');

                    if trimmed_expr.starts_with("if") {
                        let (if_exprs, else_exprs) = pass_4_if(&mut iter);

                        tokens.push(Stage4::If(trimmed_expr.to_string(), if_exprs, else_exprs));
                    } else if trimmed_expr.starts_with("for") {
                        tokens.push(Stage4::For(trimmed_expr.to_string(), pass_4_for(&mut iter)));
                    }
                } else if trimmed_expr.starts_with("let") && trimmed_expr.ends_with(';') {
                    tokens.push(Stage4::ExprAssign(expr));
                } else if trimmed_expr.ends_with(".render(writer)") {
                    tokens.push(Stage4::ExprRender(expr));
                } else {
                    tokens.push(Stage4::Expr(expr));
                }
            }
            Stage3::Other(other) => tokens.push(Stage4::Other(other)),
        }
    }

    tokens
}

#[allow(clippy::while_let_on_iterator)]
#[inline]
fn pass_4_if<I>(iter: &mut I) -> (Vec<Stage4>, Option<Vec<Stage4>>)
where
    I: Iterator<Item = Stage3>,
{
    let mut if_exprs = Vec::new();
    let mut else_exprs = Vec::new();

    let mut in_else = false;

    while let Some(token) = iter.next() {
        let mut_expr = if in_else {
            &mut else_exprs
        } else {
            &mut if_exprs
        };

        match token {
            Stage3::Expr(expr) => {
                let trimmed_expr = expr.trim();

                if trimmed_expr.starts_with('#') {
                    let trimmed_expr = trimmed_expr.trim_start_matches('#');

                    if trimmed_expr.starts_with("if") {
                        mut_expr.push({
                            let (if_exprs, else_exprs) = pass_4_if(iter);

                            Stage4::If(trimmed_expr.to_string(), if_exprs, else_exprs)
                        });
                    } else if trimmed_expr.starts_with("for") {
                        mut_expr.push(Stage4::For(trimmed_expr.to_string(), pass_4_for(iter)));
                    }
                } else if trimmed_expr == "else" {
                    in_else = true;
                } else if trimmed_expr.starts_with('/') {
                    break;
                } else if trimmed_expr.starts_with("let") && trimmed_expr.ends_with(';') {
                    mut_expr.push(Stage4::ExprAssign(expr));
                } else if trimmed_expr.ends_with(".render(writer)") {
                    mut_expr.push(Stage4::ExprRender(expr));
                } else {
                    mut_expr.push(Stage4::Expr(expr));
                }
            }
            Stage3::Other(other) => mut_expr.push(Stage4::Other(other)),
        }
    }

    (
        if_exprs,
        if else_exprs.is_empty() {
            None
        } else {
            Some(else_exprs)
        },
    )
}

#[allow(clippy::while_let_on_iterator)]
#[inline]
fn pass_4_for<I>(iter: &mut I) -> Vec<Stage4>
where
    I: Iterator<Item = Stage3>,
{
    let mut exprs = Vec::new();

    while let Some(token) = iter.next() {
        match token {
            Stage3::Expr(expr) => {
                let trimmed_expr = expr.trim();

                if trimmed_expr.starts_with('#') {
                    let trimmed_expr = trimmed_expr.trim_start_matches('#');

                    if trimmed_expr.starts_with("if") {
                        let (if_exprs, else_exprs) = pass_4_if(iter);

                        exprs.push(Stage4::If(trimmed_expr.to_string(), if_exprs, else_exprs));
                    } else if trimmed_expr.starts_with("for") {
                        exprs.push(Stage4::For(trimmed_expr.to_string(), pass_4_for(iter)));
                    }
                } else if trimmed_expr.starts_with('/') {
                    break;
                } else if trimmed_expr.starts_with("let") && trimmed_expr.ends_with(';') {
                    exprs.push(Stage4::ExprAssign(expr));
                } else if trimmed_expr.ends_with(".render(writer)") {
                    exprs.push(Stage4::ExprRender(expr));
                } else {
                    exprs.push(Stage4::Expr(expr));
                }
            }
            Stage3::Other(other) => exprs.push(Stage4::Other(other)),
        }
    }

    exprs
}
