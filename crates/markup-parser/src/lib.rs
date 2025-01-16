#![deny(rust_2018_idioms)]

use honeycomb::atoms;

fn parser() {
    // SYMBOLS / CHARACTERS
    let text_chars =
        atoms::if_take(|c| matches!(c, 'a'..='z' | 'A'..='Z' | '_' | '-' | ':' | '0'..='9'));

    let chevron_left_normal = atoms::seq("<");
    let chevron_left_closed = atoms::seq("</");
    let chevron_left_bang = atoms::seq("<!");
    let chevron_left_question = atoms::seq("<?");

    let chevron_right_normal = atoms::seq(">");
    let chevron_right_closed = atoms::seq("/>");
    let chevron_right_question = atoms::seq("?>");
    let chevron_right = chevron_right_normal
        .clone()
        .or(chevron_right_closed)
        .or(chevron_right_question);

    let equal = atoms::seq("=");
    let quote_double = atoms::seq("\"");
    let quote_single = atoms::seq("'");
    let quote = quote_double.or(quote_single);

    // COMMENTS
    let comment_tag_start = chevron_left_bang.and(atoms::seq("--"));
    let comment_tag_end = atoms::seq("--").and(chevron_right_normal.clone());
    let comment_normal = comment_tag_start
        .and(
            atoms::not(comment_tag_end.clone())
                .and(atoms::any())
                .repeat(..),
        )
        .and(comment_tag_end);

    // ATTRIBUTES
}
