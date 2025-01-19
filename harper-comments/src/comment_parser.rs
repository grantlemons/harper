use std::path::Path;

use comment_parsers::{Go, JavaDoc, JsDoc, Unit};
use harper_core::parsers::{self, MarkdownOptions, Parser};
use harper_core::{FullDictionary, Token};
use harper_tree_sitter::TreeSitterMasker;
use tree_sitter::Node;

use crate::comment_parsers;

pub struct CommentParser {
    inner: parsers::Mask<TreeSitterMasker, Box<dyn Parser>>,
}

impl CommentParser {
    pub fn create_ident_dict(&self, source: &[char]) -> Option<FullDictionary> {
        self.inner.masker.create_ident_dict(source)
    }

    pub fn new_from_language_id(
        language_id: &str,
        markdown_options: MarkdownOptions,
    ) -> Option<Self> {
        let language = match language_id {
            "rust" => tree_sitter_rust::language(),
            "typescriptreact" => tree_sitter_typescript::language_tsx(),
            "typescript" => tree_sitter_typescript::language_typescript(),
            "python" => tree_sitter_python::language(),
            "nix" => tree_sitter_nix::language(),
            "javascript" => tree_sitter_javascript::language(),
            "javascriptreact" => tree_sitter_typescript::language_tsx(),
            "go" => tree_sitter_go::language(),
            "c" => tree_sitter_c::language(),
            "cpp" => tree_sitter_cpp::language(),
            "cmake" => tree_sitter_cmake::language(),
            "ruby" => tree_sitter_ruby::language(),
            "swift" => tree_sitter_swift::language(),
            "csharp" => tree_sitter_c_sharp::language(),
            "toml" => tree_sitter_toml::language(),
            "lua" => tree_sitter_lua::language(),
            "shellscript" => tree_sitter_bash::language(),
            "java" => tree_sitter_java::language(),
            "haskell" => tree_sitter_haskell::language(),
            _ => return None,
        };

        let comment_parser: Box<dyn Parser> = match language_id {
            "javascriptreact" | "typescript" | "typescriptreact" | "javascript" => {
                Box::new(JsDoc::new_markdown(markdown_options))
            }
            "java" => Box::new(JavaDoc::default()),
            "go" => Box::new(Go::new_markdown(markdown_options)),
            _ => Box::new(Unit::new_markdown(markdown_options)),
        };

        Some(Self {
            inner: parsers::Mask::new(
                TreeSitterMasker::new(language, Self::node_condition),
                comment_parser,
            ),
        })
    }

    /// Infer the programming language from a provided filename.
    pub fn new_from_filename(filename: &Path, markdown_options: MarkdownOptions) -> Option<Self> {
        Self::new_from_language_id(Self::filename_to_filetype(filename)?, markdown_options)
    }

    /// Convert a provided path to a corresponding Language Server Protocol file
    /// type.
    ///
    /// Note to contributors: try to keep this in sync with
    /// [`Self::new_from_language_id`]
    fn filename_to_filetype(path: &Path) -> Option<&'static str> {
        Some(match path.extension()?.to_str()? {
            "py" => "python",
            "nix" => "nix",
            "rs" => "rust",
            "ts" => "typescript",
            "tsx" => "typescriptreact",
            "js" => "javascript",
            "jsx" => "javascriptreact",
            "go" => "go",
            "c" => "c",
            "cpp" => "cpp",
            "cmake" => "cmake",
            "h" => "cpp",
            "rb" => "ruby",
            "swift" => "swift",
            "cs" => "csharp",
            "toml" => "toml",
            "lua" => "lua",
            "sh" => "shellscript",
            "bash" => "shellscript",
            "java" => "java",
            "hs" => "haskell",
            _ => return None,
        })
    }

    fn node_condition(n: &Node) -> bool {
        n.kind().contains("comment")
    }
}

impl Parser for CommentParser {
    fn parse(&self, source: &[char]) -> Vec<Token> {
        self.inner.parse(source)
    }
}

#[cfg(test)]
mod tests {
    use harper_core::{
        parsers::{MarkdownOptions, StrParser},
        TokenKind,
    };

    use super::CommentParser;

    #[test]
    fn ignores_shebangs() {
        let source = "
#!/bin/python3

# This is a comment
a = 1
";
        let python_parser =
            CommentParser::new_from_language_id("python", MarkdownOptions::default()).unwrap();
        let tokens = python_parser.parse_str(source);
        let token_kinds: Vec<_> = tokens.iter().map(|t| t.kind).collect();

        dbg!(&token_kinds);
        assert!(matches!(
            token_kinds.as_slice(),
            &[
                TokenKind::Word(_), // This
                TokenKind::Space(_),
                TokenKind::Word(_), // is
                TokenKind::Space(_),
                TokenKind::Word(_), // a
                TokenKind::Space(_),
                TokenKind::Word(_), // comment
            ]
        ));
    }
}
