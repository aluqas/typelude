use std::collections::BTreeMap;

use typelude_tooling_core::{Graph, GraphEdge, GraphNode, GraphNodeKind, NodeId};

/// A parsed typelude type expression tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeExpr {
    /// A bare identifier with no type arguments (e.g. `True`, `Nil`, `U3`).
    Name(String),
    /// A generic type (e.g. `EIf<Cond, Then, Else>`).
    Generic {
        name: String,
        args: Vec<TypeExpr>,
    },
}

/// Semantic form of a typelude type expression.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SemanticExpr {
    /// `EIf<Cond, Then, Else>`
    If {
        cond: Box<SemanticExpr>,
        then: Box<SemanticExpr>,
        else_: Box<SemanticExpr>,
    },
    /// `EWhile<Pred, Step, Init>`
    While {
        pred: Box<SemanticExpr>,
        step: Box<SemanticExpr>,
        init: Box<SemanticExpr>,
    },
    /// `EApp<Func, Arg>`
    App {
        func: Box<SemanticExpr>,
        arg: Box<SemanticExpr>,
    },
    /// `EGet<Arr, Key>`
    Get {
        arr: Box<SemanticExpr>,
        key: Box<SemanticExpr>,
    },
    /// `EMap<Func, List>`
    Map {
        func: Box<SemanticExpr>,
        list: Box<SemanticExpr>,
    },
    /// `EFilter<Pred, List>`
    Filter {
        pred: Box<SemanticExpr>,
        list: Box<SemanticExpr>,
    },
    /// `EFold<Func, Init, List>`
    Fold {
        func: Box<SemanticExpr>,
        init: Box<SemanticExpr>,
        list: Box<SemanticExpr>,
    },
    /// `ELit<T>` — a literal value lifted into Eval
    Lit(Box<SemanticExpr>),
    /// `Array<H, T>` cons cell
    Array {
        head: Box<SemanticExpr>,
        tail: Box<SemanticExpr>,
    },
    /// `Nil` — the empty list
    Nil,
    /// Anything else (typenum naturals, booleans, unknown generics)
    Primitive(String),
}

impl TypeExpr {
    /// Parse a typelude type expression string into a `TypeExpr` tree.
    ///
    /// Returns `None` if the input is empty or malformed.
    #[must_use]
    pub fn parse(input: &str) -> Option<Self> {
        let input = input.trim();
        if input.is_empty() {
            return None;
        }
        Some(parse_expr(input))
    }

    /// Lift the parsed tree into a [`SemanticExpr`].
    #[must_use]
    pub fn lift(&self) -> SemanticExpr {
        lift_to_semantic(self)
    }
}

impl SemanticExpr {
    /// Build a [`Graph`] from this semantic expression, starting node IDs at
    /// `start_id`.  Returns the graph and the root node id.
    #[must_use]
    pub fn to_graph(&self) -> Graph {
        let mut graph = Graph::default();
        let mut counter = 0_u64;
        build_graph(self, &mut graph, &mut counter);
        graph
    }
}

// ---------------------------------------------------------------------------
// Parsing
// ---------------------------------------------------------------------------

fn parse_expr(input: &str) -> TypeExpr {
    if let Some(inner) = strip_generic(input, "") {
        // Has angle brackets — generic
        let lt = input.find('<').unwrap();
        let name = input[..lt].trim().to_owned();
        let args = split_top_level(inner)
            .into_iter()
            .map(|a| parse_expr(a.trim()))
            .collect();
        TypeExpr::Generic {
            name,
            args,
        }
    } else {
        TypeExpr::Name(input.to_owned())
    }
}

/// Returns the content between the outermost `<` and `>` if the input ends
/// with `>` and contains `<`.
fn strip_generic<'a>(input: &'a str, _prefix: &str) -> Option<&'a str> {
    let lt = input.find('<')?;
    if !input.ends_with('>') {
        return None;
    }
    // Verify depth is balanced.
    let inner = &input[lt + 1..input.len() - 1];
    let mut depth = 0_i32;
    for ch in inner.chars() {
        match ch {
            '<' => depth += 1,
            '>' => {
                depth -= 1;
                if depth < 0 {
                    return None;
                }
            },
            _ => {},
        }
    }
    Some(inner)
}

fn split_top_level(input: &str) -> Vec<&str> {
    let mut result = Vec::new();
    let mut depth = 0_i32;
    let mut start = 0_usize;
    for (i, ch) in input.char_indices() {
        match ch {
            '<' => depth += 1,
            '>' => depth -= 1,
            ',' if depth == 0 => {
                result.push(input[start..i].trim());
                start = i + 1;
            },
            _ => {},
        }
    }
    result.push(input[start..].trim());
    result
}

// ---------------------------------------------------------------------------
// Semantic lifting
// ---------------------------------------------------------------------------

fn lift_to_semantic(expr: &TypeExpr) -> SemanticExpr {
    match expr {
        TypeExpr::Name(name) => {
            if name == "Nil" {
                SemanticExpr::Nil
            } else {
                SemanticExpr::Primitive(name.clone())
            }
        },
        TypeExpr::Generic {
            name,
            args,
        } => match name.as_str() {
            "EIf" if args.len() == 3 => SemanticExpr::If {
                cond: Box::new(lift_to_semantic(&args[0])),
                then: Box::new(lift_to_semantic(&args[1])),
                else_: Box::new(lift_to_semantic(&args[2])),
            },
            "EWhile" if args.len() == 3 => SemanticExpr::While {
                pred: Box::new(lift_to_semantic(&args[0])),
                step: Box::new(lift_to_semantic(&args[1])),
                init: Box::new(lift_to_semantic(&args[2])),
            },
            "EApp" if args.len() == 2 => SemanticExpr::App {
                func: Box::new(lift_to_semantic(&args[0])),
                arg: Box::new(lift_to_semantic(&args[1])),
            },
            "EGet" if args.len() == 2 => SemanticExpr::Get {
                arr: Box::new(lift_to_semantic(&args[0])),
                key: Box::new(lift_to_semantic(&args[1])),
            },
            "EMap" if args.len() == 2 => SemanticExpr::Map {
                func: Box::new(lift_to_semantic(&args[0])),
                list: Box::new(lift_to_semantic(&args[1])),
            },
            "EFilter" if args.len() == 2 => SemanticExpr::Filter {
                pred: Box::new(lift_to_semantic(&args[0])),
                list: Box::new(lift_to_semantic(&args[1])),
            },
            "EFold" if args.len() == 3 => SemanticExpr::Fold {
                func: Box::new(lift_to_semantic(&args[0])),
                init: Box::new(lift_to_semantic(&args[1])),
                list: Box::new(lift_to_semantic(&args[2])),
            },
            "ELit" if args.len() == 1 => {
                SemanticExpr::Lit(Box::new(lift_to_semantic(&args[0])))
            },
            "Array" if args.len() == 2 => SemanticExpr::Array {
                head: Box::new(lift_to_semantic(&args[0])),
                tail: Box::new(lift_to_semantic(&args[1])),
            },
            _ => {
                // Generic type we don't recognise — treat as primitive with its
                // full stringified form.
                SemanticExpr::Primitive(format_generic(name, args))
            },
        },
    }
}

fn format_generic(name: &str, args: &[TypeExpr]) -> String {
    if args.is_empty() {
        return name.to_owned();
    }
    let inner = args
        .iter()
        .map(format_type_expr)
        .collect::<Vec<_>>()
        .join(", ");
    format!("{name}<{inner}>")
}

fn format_type_expr(expr: &TypeExpr) -> String {
    match expr {
        TypeExpr::Name(n) => n.clone(),
        TypeExpr::Generic {
            name,
            args,
        } => format_generic(name, args),
    }
}

// ---------------------------------------------------------------------------
// Graph construction
// ---------------------------------------------------------------------------

fn build_graph(expr: &SemanticExpr, graph: &mut Graph, counter: &mut u64) -> NodeId {
    *counter += 1;
    let id = NodeId::new(*counter);

    match expr {
        SemanticExpr::If {
            cond,
            then,
            else_,
        } => {
            push_node(graph, id, GraphNodeKind::Expression, "EIf");
            wire_child(graph, counter, id, cond, "cond");
            wire_child(graph, counter, id, then, "then");
            wire_child(graph, counter, id, else_, "else");
        },
        SemanticExpr::While {
            pred,
            step,
            init,
        } => {
            push_node(graph, id, GraphNodeKind::Expression, "EWhile");
            wire_child(graph, counter, id, pred, "pred");
            wire_child(graph, counter, id, step, "step");
            wire_child(graph, counter, id, init, "init");
        },
        SemanticExpr::App {
            func,
            arg,
        } => {
            push_node(graph, id, GraphNodeKind::Expression, "EApp");
            wire_child(graph, counter, id, func, "func");
            wire_child(graph, counter, id, arg, "arg");
        },
        SemanticExpr::Get {
            arr,
            key,
        } => {
            push_node(graph, id, GraphNodeKind::Expression, "EGet");
            wire_child(graph, counter, id, arr, "arr");
            wire_child(graph, counter, id, key, "key");
        },
        SemanticExpr::Map {
            func,
            list,
        } => {
            push_node(graph, id, GraphNodeKind::Expression, "EMap");
            wire_child(graph, counter, id, func, "func");
            wire_child(graph, counter, id, list, "list");
        },
        SemanticExpr::Filter {
            pred,
            list,
        } => {
            push_node(graph, id, GraphNodeKind::Expression, "EFilter");
            wire_child(graph, counter, id, pred, "pred");
            wire_child(graph, counter, id, list, "list");
        },
        SemanticExpr::Fold {
            func,
            init,
            list,
        } => {
            push_node(graph, id, GraphNodeKind::Expression, "EFold");
            wire_child(graph, counter, id, func, "func");
            wire_child(graph, counter, id, init, "init");
            wire_child(graph, counter, id, list, "list");
        },
        SemanticExpr::Lit(inner) => {
            push_node(graph, id, GraphNodeKind::Expression, "ELit");
            wire_child(graph, counter, id, inner, "value");
        },
        SemanticExpr::Array {
            head,
            tail,
        } => {
            push_node(graph, id, GraphNodeKind::Semantic, "Array");
            wire_child(graph, counter, id, head, "head");
            wire_child(graph, counter, id, tail, "tail");
        },
        SemanticExpr::Nil => {
            push_node(graph, id, GraphNodeKind::Semantic, "Nil");
        },
        SemanticExpr::Primitive(label) => {
            push_node(graph, id, GraphNodeKind::Goal, label);
        },
    }

    id
}

fn push_node(graph: &mut Graph, id: NodeId, kind: GraphNodeKind, label: &str) {
    graph.nodes.push(GraphNode {
        id,
        kind,
        label: label.to_owned(),
        span_id: None,
        metadata: BTreeMap::new(),
    });
}

fn wire_child(
    graph: &mut Graph,
    counter: &mut u64,
    parent: NodeId,
    child: &SemanticExpr,
    edge_label: &str,
) {
    let child_id = build_graph(child, graph, counter);
    graph.edges.push(GraphEdge {
        from: parent,
        to: child_id,
        label: edge_label.to_owned(),
        metadata: BTreeMap::new(),
    });
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::{SemanticExpr, TypeExpr};

    #[test]
    fn parses_bare_name() {
        let expr = TypeExpr::parse("Nil").unwrap();
        assert_eq!(expr, TypeExpr::Name(String::from("Nil")));
    }

    #[test]
    fn parses_generic() {
        let expr = TypeExpr::parse("EIf<True, U1, U0>").unwrap();
        assert_eq!(
            expr,
            TypeExpr::Generic {
                name: String::from("EIf"),
                args: vec![
                    TypeExpr::Name(String::from("True")),
                    TypeExpr::Name(String::from("U1")),
                    TypeExpr::Name(String::from("U0")),
                ],
            }
        );
    }

    #[test]
    fn lifts_eif_to_semantic() {
        let expr = TypeExpr::parse("EIf<True, U1, U0>").unwrap();
        let sem = expr.lift();
        assert!(matches!(sem, SemanticExpr::If { .. }));
    }

    #[test]
    fn lifts_ewhile_to_semantic() {
        let expr = TypeExpr::parse("EWhile<Pred, Step, U0>").unwrap();
        let sem = expr.lift();
        assert!(matches!(sem, SemanticExpr::While { .. }));
    }

    #[test]
    fn lifts_nil() {
        let expr = TypeExpr::parse("Nil").unwrap();
        assert_eq!(expr.lift(), SemanticExpr::Nil);
    }

    #[test]
    fn lifts_array_cons() {
        let expr = TypeExpr::parse("Array<U1, Nil>").unwrap();
        let sem = expr.lift();
        assert!(matches!(sem, SemanticExpr::Array { .. }));
    }

    #[test]
    fn eif_graph_has_three_children() {
        let expr = TypeExpr::parse("EIf<True, U1, U0>").unwrap();
        let graph = expr.lift().to_graph();
        // root (EIf) + 3 children
        assert_eq!(graph.nodes.len(), 4);
        assert_eq!(graph.edges.len(), 3);
        let labels: Vec<&str> = graph.edges.iter().map(|e| e.label.as_str()).collect();
        assert!(labels.contains(&"cond"));
        assert!(labels.contains(&"then"));
        assert!(labels.contains(&"else"));
    }

    #[test]
    fn eapp_graph_has_two_children() {
        let expr = TypeExpr::parse("EApp<MyFn, U3>").unwrap();
        let graph = expr.lift().to_graph();
        assert_eq!(graph.nodes.len(), 3);
        assert_eq!(graph.edges.len(), 2);
    }

    #[test]
    fn nested_eif_graph_is_correct() {
        // EIf<True, EApp<F, U1>, U0>
        let expr = TypeExpr::parse("EIf<True, EApp<F, U1>, U0>").unwrap();
        let graph = expr.lift().to_graph();
        // EIf(1) + True(2) + EApp(3) + F(4) + U1(5) + U0(6)
        assert_eq!(graph.nodes.len(), 6);
    }
}
