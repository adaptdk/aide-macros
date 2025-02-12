use proc_macro::TokenStream;
use quote::quote;
use std::str::FromStr;
use syn::{
    meta::{parser, ParseNestedMeta},
    parse_macro_input, Attribute, Expr, ExprLit, ItemFn, Lit, LitStr, Meta, MetaNameValue,
};

/// A procedural macro that generates a function to add API documentation to a route.
///
/// This macro processes doc comments above the function and extracts:
/// - A summary (first line of the doc comment)
/// - A description (subsequent lines of doc comments)
///
/// It also accepts an optional `tag` parameter to categorize the route.
#[proc_macro_attribute]
pub fn aide_docs(args: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);

    let doc_lines = collect_doc_comments(&input.attrs);
    let (summary, description) = split_summary_description(&doc_lines);

    let mut attrs = AideDocsAttributes::default();

    // Initialize our custom parser
    let aide_docs_parser = parser(|meta| attrs.parse(meta));

    // Parse the macro invocation's arguments, adding them to the `attrs` struct.
    parse_macro_input!(args with aide_docs_parser);

    // If a tag was provided, creates a bit of code which adds the tag to the route.
    // If not, creates an empty bit of code.
    let tag_method = match attrs.tag {
        None => tokens_from_string(String::default()),
        Some(tag) => tokens_from_string(format!(r#".tag("{}")"#, tag)),
    };

    let aide_docs_fn = tokens_from_string(format!("aide_docs_{}", input.sig.ident));

    let expanded = quote! {
        #input

        pub fn #aide_docs_fn(
        ) -> impl FnOnce(aide::transform::TransformOperation<'_>) -> aide::transform::TransformOperation<'_>
        {
            move |op| {
                op.summary(#summary).description(#description)#tag_method
            }
        }
    };

    expanded.into()
}

/// Takes a string and converts it into a [proc_macro2::TokenStream].
///
/// # Panics
/// Will panic if the input string cannot be parsed into a valid TokenStream. The function is only
/// run at compile time, so panicking is fine.
fn tokens_from_string(string: String) -> proc_macro2::TokenStream {
    proc_macro2::TokenStream::from_str(&string).unwrap()
}

#[derive(Default)]
struct AideDocsAttributes {
    tag: Option<String>,
}

impl AideDocsAttributes {
    /// Parses macro arguments into [AideDocsAttributes]
    fn parse(&mut self, meta: ParseNestedMeta) -> syn::parse::Result<()> {
        if meta.path.is_ident("tag") {
            self.tag = Some(meta.value()?.parse::<LitStr>()?.value());
            Ok(())
        } else {
            let ident = meta
                .path
                .get_ident()
                .map(|i| i.to_string())
                .unwrap_or_default();

            Err(meta.error(format!("unsupported property '{ident}'",)))
        }
    }
}

/// Takes a slice of [Attribute]s, finds those that are non-empty doc comments, and returns them as
/// [String]s.
fn collect_doc_comments(attrs: &[Attribute]) -> Vec<String> {
    let mut lines = Vec::new();
    for attr in attrs {
        // Skip attributes that aren't doc comments
        if !attr.path().is_ident("doc") {
            continue;
        }

        // Unpack literal string from doc comment
        if let Meta::NameValue(MetaNameValue {
            value:
                Expr::Lit(ExprLit {
                    lit: Lit::Str(lit_str),
                    ..
                }),
            ..
        }) = &attr.meta
        {
            // If the doc comment isn't empty, trim it and add it to `lines`
            if !lit_str.value().is_empty() {
                lines.push(lit_str.value().trim().into())
            }
        }
    }

    lines
}

/// Splits a slice of strings into a summary and description. The first string in the slice
/// becomes the summary, and the remaining strings are joined with newlines to become the description.
fn split_summary_description(lines: &[String]) -> (String, String) {
    match lines.split_first() {
        Some((summary, desc)) => (summary.clone(), desc.join("\n")),
        None => ("".into(), "".into()),
    }
}
