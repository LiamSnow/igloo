use proc_macro2::Span;
use syn::Ident;

pub fn upper_camel_to_snake(s: &str) -> String {
    let mut res = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && c.is_uppercase() {
            res.push('_');
        }
        res.push(c.to_ascii_lowercase());
    }
    res
}

pub fn upper_camel_to_kebab(s: &str) -> String {
    let mut res = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && c.is_uppercase() {
            res.push('-');
        }
        res.push(c.to_ascii_lowercase());
    }
    res
}

pub fn ident(name: &str) -> Ident {
    Ident::new(name, Span::call_site())
}
