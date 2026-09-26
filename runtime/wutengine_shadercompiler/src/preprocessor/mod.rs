//! Shader preprocessing

use core::error::Error;
use core::range::RangeInclusive;
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::collections::HashSet;
use std::sync::Arc;

use crate::ShaderResolver;
use crate::preprocessor::parser::Expr;
use crate::preprocessor::parser::KeywordRange;
use crate::preprocessor::parser::Rule;

mod parser;

/// Start token for a preprocessor directive. Lines that start with this character will be parsed as a preprocessor directive
const DIRECTIVE_START: &str = "#";

/// An error during shader preprocessing
#[derive(Debug, derive_more::Error, derive_more::Display, derive_more::From)]
pub enum PreprocessErr {
    /// Resolver failed
    #[display("Shader resolver returned an error: {_0}")]
    #[from(skip)]
    Resolver(Box<dyn Error + Send>),

    /// Directive could not be parsed
    #[display("Error parsing a directive: {_0}")]
    DirectiveParser(Box<pest::error::Error<Rule>>),

    /// No name directive was found
    #[display("No name directive was found")]
    MissingName,

    /// Name directive was not first
    #[display("The first directive must be a name directive")]
    NameNotFirst,

    /// Branch mismatch
    #[display("if/else mismatch")]
    BranchMismatch,

    /// A keyword was declared twice
    #[display("Keyword `{_0}` is declared more than once")]
    #[from(skip)]
    DuplicateKeyword(#[error(not(source))] String),

    /// A keyword was declared with a range that contains no values
    #[display("Keyword `{_0}` has an empty range")]
    #[from(skip)]
    EmptyKeywordRange(#[error(not(source))] String),

    /// A keyword was declared in an imported file
    #[display(
        "Keyword `{_0}` is declared in an import; keywords can only be declared in the main shader"
    )]
    #[from(skip)]
    KeywordInImport(#[error(not(source))] String),

    /// An `#if` referenced a keyword that isn't declared
    #[display("Undeclared keyword `{_0}` in an #if directive")]
    #[from(skip)]
    UndeclaredKeyword(#[error(not(source))] String),
}

/// A `#keyword` declaration
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeywordDecl {
    /// The allowed values
    pub allowed: RangeInclusive<u64>,

    /// The value used when none is given: the start of the range
    pub default: u64,
}

/// The directives of a shader that apply to all its variants
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Declarations {
    /// The shader name
    pub name: String,

    /// The declared keywords
    pub keywords: BTreeMap<Arc<str>, KeywordDecl>,
}

/// Reads the `#name` and `#keyword` directives of the main shader file, without following imports
pub fn declarations(input: &str) -> Result<Declarations, PreprocessErr> {
    profiling::function_scope!();

    let mut name = None;
    let mut keywords = BTreeMap::new();

    for line in directive_lines(input) {
        let directive = parser::parse_directive(line)
            .map_err(|e| PreprocessErr::DirectiveParser(Box::new(e)))?;

        match directive {
            parser::Directive::Name(n) => {
                name.get_or_insert_with(|| n.to_owned());
            }
            _ if name.is_none() => return Err(PreprocessErr::NameNotFirst),
            parser::Directive::KeywordDecl { ident, range } => {
                let allowed = match range {
                    None => RangeInclusive { start: 0, last: 1 },
                    Some(KeywordRange::Inclusive(r)) if r.start <= r.last => r,
                    Some(KeywordRange::Exclusive(r)) if r.start < r.end => RangeInclusive {
                        start: r.start,
                        last: r.end - 1,
                    },
                    Some(_) => return Err(PreprocessErr::EmptyKeywordRange(ident.to_owned())),
                };

                let decl = KeywordDecl {
                    allowed,
                    default: allowed.start,
                };

                if keywords.insert(Arc::from(ident), decl).is_some() {
                    return Err(PreprocessErr::DuplicateKeyword(ident.to_owned()));
                }
            }
            _ => {}
        }
    }

    Ok(Declarations {
        name: name.ok_or(PreprocessErr::MissingName)?,
        keywords,
    })
}

/// Returns the lines of `input` that are directives
fn directive_lines(input: &str) -> impl Iterator<Item = &str> {
    input
        .lines()
        .filter(|line| line.trim_start().starts_with(DIRECTIVE_START))
}

/// State of a single preprocessing run
struct PreprocessorState<'a> {
    /// The value of every declared keyword
    keywords: &'a BTreeMap<Arc<str>, u64>,

    /// The names of the files imported so far
    included_files: HashSet<String>,

    /// Resolves imports
    resolver: &'a dyn ShaderResolver,
}

impl PreprocessorState<'_> {
    fn preprocess(&mut self, input: &str, is_import: bool) -> Result<String, PreprocessErr> {
        let mut output = String::new();

        let mut branch_stack: Vec<bool> = Vec::with_capacity(32);

        for line in input.lines() {
            if !line.trim_start().starts_with(DIRECTIVE_START) {
                // Source line
                if Self::branch_stack_active(&branch_stack) {
                    output.push_str(&substitute_keywords(line, self.keywords));
                    output.push('\n');
                }
                continue;
            }

            let directive = parser::parse_directive(line)
                .map_err(|e| PreprocessErr::DirectiveParser(Box::new(e)))?;

            match directive {
                parser::Directive::Name(_) => {
                    // Read by `declarations`
                }
                parser::Directive::KeywordDecl { ident, .. } => {
                    if is_import {
                        return Err(PreprocessErr::KeywordInImport(ident.to_owned()));
                    }
                }
                parser::Directive::Import(name) => {
                    if !Self::branch_stack_active(&branch_stack) {
                        continue;
                    }

                    if !self.included_files.insert(name.to_owned()) {
                        // Already imported once
                        continue;
                    }

                    log::debug!("Importing shader `{name}`");

                    let imported_content = self
                        .resolver
                        .find_by_name(name)
                        .map_err(PreprocessErr::Resolver)?;

                    let result = self.preprocess(&imported_content, true)?;

                    output.push_str(&result);
                }
                parser::Directive::If(expr) => {
                    let value = self.evaluate_expr(&expr)?;

                    branch_stack.push(Self::branch_stack_active(&branch_stack) && value != 0);
                }
                parser::Directive::Else => {
                    let prev_val = branch_stack.pop().ok_or(PreprocessErr::BranchMismatch)?;
                    branch_stack.push(!prev_val);
                }
                parser::Directive::Endif => {
                    _ = branch_stack.pop().ok_or(PreprocessErr::BranchMismatch)?;
                }
            }
        }

        if !branch_stack.is_empty() {
            return Err(PreprocessErr::BranchMismatch);
        }

        Ok(output)
    }

    fn branch_stack_active(branch_stack: &[bool]) -> bool {
        branch_stack.iter().copied().all(|x| x)
    }

    fn evaluate_expr(&self, expr: &Expr) -> Result<u64, PreprocessErr> {
        Ok(match expr {
            Expr::Ident(ident) => *self
                .keywords
                .get(*ident)
                .ok_or_else(|| PreprocessErr::UndeclaredKeyword((*ident).to_owned()))?,
            Expr::Value(val) => *val,
            Expr::Unop { op, expr: subexpr } => match op {
                parser::UnopType::Not => u64::from(self.evaluate_expr(subexpr)? == 0),
            },
            Expr::Binop { left, op, right } => {
                let left = self.evaluate_expr(left)?;

                // Short-circuit, so the right side isn't checked for undeclared keywords either
                match op {
                    parser::BinopType::And if left == 0 => return Ok(0),
                    parser::BinopType::Or if left != 0 => return Ok(left),
                    _ => {}
                }

                let right = self.evaluate_expr(right)?;

                match op {
                    parser::BinopType::Eq => u64::from(left == right),
                    parser::BinopType::Ne => u64::from(left != right),
                    parser::BinopType::Gt => u64::from(left > right),
                    parser::BinopType::Ge => u64::from(left >= right),
                    parser::BinopType::Lt => u64::from(left < right),
                    parser::BinopType::Le => u64::from(left <= right),
                    parser::BinopType::And | parser::BinopType::Or => right,
                }
            }
        })
    }
}

/// Replaces every whole-word occurrence of a keyword in a source line with its value
fn substitute_keywords<'a>(line: &'a str, keywords: &BTreeMap<Arc<str>, u64>) -> Cow<'a, str> {
    let is_ident_char = |c: char| c.is_ascii_alphanumeric() || c == '_';

    let mut output: Option<String> = None;
    let mut copied_until = 0;
    let mut rest = line.char_indices().peekable();

    while let Some((start, c)) = rest.next() {
        if !is_ident_char(c) {
            continue;
        }

        let mut end = start + c.len_utf8();

        while let Some(&(i, c)) = rest.peek()
            && is_ident_char(c)
        {
            end = i + c.len_utf8();
            rest.next();
        }

        if let Some(value) = keywords.get(&line[start..end]) {
            let out = output.get_or_insert_with(String::new);
            out.push_str(&line[copied_until..start]);
            out.push_str(&value.to_string());
            copied_until = end;
        }
    }

    match output {
        Some(mut out) => {
            out.push_str(&line[copied_until..]);
            Cow::Owned(out)
        }
        None => Cow::Borrowed(line),
    }
}

/// Preprocesses the shader source using the provided keyword values and shader resolver. `keywords` must hold a
/// value for every declared keyword.
pub fn preprocess(
    input: &str,
    keywords: &BTreeMap<Arc<str>, u64>,
    shader_resolver: &dyn ShaderResolver,
) -> Result<String, PreprocessErr> {
    profiling::function_scope!();

    log::debug!("Preprocessing shader");

    let mut state = PreprocessorState {
        keywords,
        included_files: HashSet::new(),
        resolver: shader_resolver,
    };

    state.preprocess(input, false)
}

#[cfg(test)]
mod test {
    use super::*;

    fn keywords(pairs: &[(&str, u64)]) -> BTreeMap<Arc<str>, u64> {
        pairs.iter().map(|(k, v)| (Arc::from(*k), *v)).collect()
    }

    /// Tests that substitution only replaces whole words, to prevent `HAS_COLOR` corrupting `HAS_COLOR_MAP`
    #[test]
    fn substitute_whole_words() {
        let kw = keywords(&[("HAS_COLOR", 1), ("COUNT", 7)]);

        assert_eq!(
            "let a = 1 + HAS_COLOR_MAP + 7 * 7;",
            substitute_keywords("let a = HAS_COLOR + HAS_COLOR_MAP + COUNT * COUNT;", &kw)
        );
        assert!(matches!(
            substitute_keywords("let a = MY_COUNT;", &kw),
            Cow::Borrowed(_)
        ));
    }

    /// Tests keyword declaration parsing, to prevent wrong defaults or ranges for the three declaration forms
    #[test]
    fn keyword_declarations() {
        let decls = declarations(
            "#name \"Test\"\n#keyword FLAG\n#keyword EXCL 2..5\n#keyword INCL 3..=4\nfn main() {}",
        )
        .unwrap();

        assert_eq!("Test", decls.name);
        assert_eq!(
            RangeInclusive { start: 0, last: 1 },
            decls.keywords["FLAG"].allowed
        );
        assert_eq!(
            RangeInclusive { start: 2, last: 4 },
            decls.keywords["EXCL"].allowed
        );
        assert_eq!(2, decls.keywords["EXCL"].default);
        assert_eq!(
            RangeInclusive { start: 3, last: 4 },
            decls.keywords["INCL"].allowed
        );
    }

    /// Tests declaration errors, to prevent empty ranges and duplicate keywords from being accepted
    #[test]
    fn keyword_declaration_errors() {
        assert!(matches!(
            declarations("#name \"T\"\n#keyword A 3..3"),
            Err(PreprocessErr::EmptyKeywordRange(_))
        ));
        assert!(matches!(
            declarations("#name \"T\"\n#keyword A\n#keyword A"),
            Err(PreprocessErr::DuplicateKeyword(_))
        ));
        assert!(matches!(
            declarations("#keyword A\n#name \"T\""),
            Err(PreprocessErr::NameNotFirst)
        ));
        assert!(matches!(
            declarations("fn main() {}"),
            Err(PreprocessErr::MissingName)
        ));
    }

    /// Tests branch evaluation, to prevent an `#else` inside an inactive branch from being emitted
    #[test]
    fn branches() {
        let src = "#name \"T\"\n#if A\na\n#if B\nb\n#else\nnot_b\n#endif\n#else\nnot_a\n#endif";
        let resolver = crate::UnsupportedResolver;

        let out = preprocess(src, &keywords(&[("A", 0), ("B", 0)]), &resolver).unwrap();
        assert_eq!("not_a\n", out);

        let out = preprocess(src, &keywords(&[("A", 1), ("B", 0)]), &resolver).unwrap();
        assert_eq!("a\nnot_b\n", out);

        let out = preprocess(src, &keywords(&[("A", 1), ("B", 2)]), &resolver).unwrap();
        assert_eq!("a\nb\n", out);
    }

    /// Tests expression evaluation, to prevent operator precedence or short-circuit regressions
    #[test]
    fn expressions() {
        let resolver = crate::UnsupportedResolver;
        let kw = keywords(&[("A", 2), ("B", 0)]);
        let eval = |expr: &str| {
            let src = format!("#name \"T\"\n#if {expr}\nyes\n#endif");
            preprocess(&src, &kw, &resolver).unwrap() == "yes\n"
        };

        assert!(eval("A == 2 && !B"));
        assert!(eval("B || A > 1"));
        assert!(!eval("(B || A) == 1"));
        assert!(eval("A >= 0x2 && A < 0b11"));
        assert!(!eval("B && UNDECLARED"));
        assert!(matches!(
            preprocess("#name \"T\"\n#if UNDECLARED\n#endif", &kw, &resolver),
            Err(PreprocessErr::UndeclaredKeyword(_))
        ));
    }
}
