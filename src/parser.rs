extern crate tree_sitter;

use tree_sitter::{Language, Parser, Tree, Point, Node};

extern "C" {fn tree_sitter_php() -> Language; }

pub struct MyParser {
    tree: Tree,
    source_code: String,
}

impl MyParser {
    pub fn new(source_code: &str) -> Self {
        let language = unsafe {tree_sitter_php()};

        let mut parser = Parser::new();
        parser.set_language(language).unwrap();

        let tree = parser.parse(&source_code, None).unwrap();

        return MyParser { tree, source_code: source_code.to_string()};
    }

    pub fn get_node_at_point(&self, point: &Point) ->Option<Node> {
        return self.tree.root_node().descendant_for_point_range(*point, *point)
    }

    fn get_node_text(&self, node: Node) ->String {
        return node.utf8_text(self.source_code.as_bytes()).unwrap().to_string();
    }

    fn get_parent_function_node<'a>(&'a self, node: &'a Node) ->Option<Node> {
        let mut cur_node = *node;
        loop {
            match cur_node.parent() {
                Some(parent) => {
                    if parent.kind() == "function_call_expression" {
                        return Some(parent.to_owned());
                    }
                    cur_node = parent;
                }
                None => break,
            }
        }

        None
    }

    fn get_function_name_from_node(&self, node: Node) -> Option<String> {
        match node.child_by_field_name("function") {
            Some(node) => Some(self.get_node_text(node)),
            None => None,
        }
    }

    pub fn get_view_path_from_node(&self, node: Node) -> Option<String> {
        if let Some(func_node) = self.get_parent_function_node(&node) {
            if let Some(func_name) = self.get_function_name_from_node(func_node) {
                if "view" == func_name {
                    let view_name = self.get_node_text(node);
                    let parts: Vec<&str> = view_name.split(".").collect();
                    let view_path = format!("resources/views/{}.blade.php", parts.join("/"));

                    return Some(view_path);
                }
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
use tree_sitter::Point;

use super::MyParser;

    #[test]
    fn test_get_node() {
        let source_code = "<?php view('some.view');";
        let my_parser = MyParser::new(source_code);

        let point = Point {row: 0, column: 12};
        let node = my_parser.get_node_at_point(&point).unwrap();
        assert_eq!("string_value", node.kind());
        assert_eq!("some.view", my_parser.get_node_text(node));
        let func_node = my_parser.get_parent_function_node(&node);

        assert_eq!("function_call_expression", func_node.unwrap().kind());
        dbg!(func_node.unwrap().named_child(0));
        dbg!(func_node.unwrap().child_by_field_name("function"));
        assert_eq!("view", my_parser.get_function_name_from_node(func_node.unwrap()).unwrap());

        let view_name = my_parser.get_node_text(node);
        let parts: Vec<&str> = view_name.split(".").collect();
        let view_path = format!("resources/views/{}.blade.php", parts.join("/"));

        assert_eq!("resources/views/some/view.blade.php", view_path);
    }

    #[test]
    fn get() {
        let source_code = "<?php view('some.view');";
        let my_parser = MyParser::new(source_code);
        let point = Point {row: 0, column: 12};
        let node = my_parser.get_node_at_point(&point).unwrap();

        assert_eq!("resources/views/some/view.blade.php", my_parser.get_view_path_from_node(node).unwrap());
    }
}
