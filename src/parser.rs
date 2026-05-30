use tree_sitter::{Node, Parser};

pub struct PhpParser {
    parser: Parser,
}

impl PhpParser {
    pub fn new() -> Self {
        let mut parser = Parser::new();
        let language = tree_sitter_php::LANGUAGE_PHP.into();
        parser.set_language(&language).expect("tree-sitter-php language load failed");
        Self { parser }
    }

    pub fn parse(&mut self, source: &str) -> Option<tree_sitter::Tree> {
        self.parser.parse(source, None)
    }
}

/// Collects all nodes of a given kind depth-first.
pub fn collect_nodes<'tree>(
    node: Node<'tree>,
    kind: &str,
    out: &mut Vec<Node<'tree>>,
) {
    if node.kind() == kind {
        out.push(node);
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        collect_nodes(child, kind, out);
    }
}
