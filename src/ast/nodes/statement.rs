use crate::{
    analysis_table::NonTerminal,
    ast::nodes::{parse_access, ExpressionKind},
    common::{
        diagnostic::{Diagnostic, DiagnosticGravity},
        localizable::Localizable,
        types::{
            file_element::{empty_file_elt, file_element_from},
            FileElement, Node, Token, Tree,
        },
    },
    parser::Lexem,
};

use super::{Assign, Ast, AstNode, Conditional, Expression, For};

pub enum Statement {
    Print(Expression),
    Println(Expression),
    Return(Expression),
    For(For),
    Conditional(Conditional),
    Assign(Assign),
    Expr(Expression),
    NotImplemented,
}

impl AstNode for Statement {
    fn get_string_repr(&self) -> String {
        String::from(match self {
            Statement::Print(_) => "Statement::Print",
            Statement::Println(_) => "Statement::Println",
            Statement::Return(_) => "Statement::Return",
            Statement::For(_) => "Statement::For",
            Statement::Conditional(_) => "Statement::Conditional",
            Statement::Assign(_) => "Statement::Assign",
            Statement::Expr(_) => "Statement::Expr",
            Statement::NotImplemented => "Statement::NotImplemented",
        })
    }
}

impl From<Ast> for Statement {
    fn from(value: Ast) -> Self {
        match value {
            Ast::Statement(s) => s,
            _ => panic!("Cannot convert {} to Statement", value),
        }
    }
}

impl From<Tree<FileElement<Lexem>>> for Statement {
    fn from(root: Tree<FileElement<Lexem>>) -> Self {
        let root_non_terminal = match root.borrow().get_value().element {
            Lexem::NonTerminal(nt) => nt,
            _ => panic!("Trying to parse STATEMENT from terminal concrete node"),
        };

        if root_non_terminal != NonTerminal::Stmt {
            panic!("Cannot parse Statement from {} node", root_non_terminal);
        }

        let left_child_elem = root.borrow().get_children()[0]
            .clone()
            .borrow()
            .get_value()
            .element;

        match left_child_elem {
            Lexem::NonTerminal(NonTerminal::SimpleStmt) => {
                return parse_simple(root.borrow().get_children()[0].clone())
            }
            Lexem::Terminal(Token::For) => return Statement::For(For::from(root)),
            Lexem::Terminal(Token::If) => return Statement::Conditional(Conditional::from(root)),
            _ => return Statement::NotImplemented,
        }
    }
}

fn parse_simple(root: Tree<FileElement<Lexem>>) -> Statement {
    let left_child_elem = root.borrow().get_children()[0]
        .clone()
        .borrow()
        .get_value()
        .element;

    match left_child_elem {
        Lexem::Terminal(Token::Print) => {
            Statement::Print(Expression::from(root.borrow().get_children()[2].clone()))
        }
        Lexem::Terminal(Token::Println) => {
            Statement::Println(Expression::from(root.borrow().get_children()[2].clone()))
        }
        Lexem::Terminal(Token::Return) => {
            Statement::Return(Expression::from(root.borrow().get_children()[1].clone()))
        }
        Lexem::Terminal(Token::Identifier(_))
        | Lexem::NonTerminal(NonTerminal::ExprNoIdentNoAccess) => parse_complex(root),
        _ => {
            Diagnostic::from_localizable(
                root.borrow().get_children()[0].borrow().get_value(),
                DiagnosticGravity::Warning,
                String::from("UnimplementedStatement"),
                format!("This statement type is not yet supported"),
            )
            .display();
            return Statement::NotImplemented;
        }
    }
}

/// Parse complex statements that starts with identifier or expr_no_indent_no_access
fn parse_complex(root: Tree<FileElement<Lexem>>) -> Statement {
    let left_child_elt = root.borrow().get_children()[0].borrow().get_value().element;

    if let Lexem::Terminal(Token::Identifier(_)) = left_child_elt {
        // Expression starting with identifier
        return parse_ident_stmt(root);
    } else if left_child_elt == Lexem::NonTerminal(NonTerminal::ExprNoIdentNoAccess) {
        // Expr not starting with identifier
        return parse_complex_stmt(root);
    }

    Diagnostic::from_localizable(
        root.borrow().get_children()[0].borrow().get_value(),
        DiagnosticGravity::Warning,
        String::from("UnimplementedStatement"),
        format!("This statement type is not yet supported"),
    )
    .display();

    return Statement::NotImplemented;
}

/// Descend into childrens with the given indexes
macro_rules! descend_children {
    ($root: expr, $n: expr) => {
        $root.borrow().get_children()[$n]
    };

    ($root: expr, $n: expr, $($children:expr),+) => {
        descend_children!(descend_children!($root, $n), $($children),+)
    }
}

/// Add a non terminal child with an empty fileelement to the given root
macro_rules! add_nonterm_child {
    ($root: expr, $nt: expr) => {
        $root
            .borrow_mut()
            .add_child(&$root, Node::new(empty_file_elt!(Lexem::NonTerminal($nt))));
    };
}

/// Parse a statement begining with an identifier
///
/// The first case is a simple id = expr assignement, with the following tree :
///
///             simple_stmt
///               /    \
///             id      simple_stmt_ident
///                        /       \
///                       =        expr
///
/// The second case is a expr1 = expr2 assignement, with expr1 starting with an id the following tree :
///
///                         simple_stmt
///                        /          \
///                     id          simple_stmt_ident
///                        ________/        |        \______________
///                       /                 |                       \
///                      /                  |                        \
///             factor_id             expr2_no_access                simple_stmt_expr ___________
///                                  /   |        \                 /    \          \___         \
///                                or  expr_and  expr2_no_access   /      \              =S        \
///                                                  ...       [ expr ]    access_suite            expr
///                                                                         /       \
///
/// In this case, we use the parse_complex_stmt function, so we need to adapt the tree to the function.                                                                          ...
/// We move create a factor tree for the left identifier, embeded in a normal expr tree to handle the ors etc,
///
///
///                             stmt
///                            /    \_________
///                           /               \
///                         expr               simple_stmt_expr
///                        /    \______             ...
///                       /            \
///                      /              \
///                   factor        expr2_no_access
///                  /      \           ...
///                id    factor_id
///
///
///
fn parse_ident_stmt(root: Tree<FileElement<Lexem>>) -> Statement {
    // Simple id = expr statement
    if descend_children!(root, 1).borrow().get_children().len() == 2 {
        let right_expr: Tree<FileElement<Lexem>> = descend_children!(root, 1, 1).clone();
        let ident_node: Tree<FileElement<Lexem>> = descend_children!(root, 0).clone();

        let left_expr: Tree<FileElement<Lexem>> = Node::new(file_element_from!(
            ident_node.borrow().get_value(),
            Lexem::NonTerminal(NonTerminal::Factor)
        ));

        left_expr.borrow_mut().add_child(&left_expr, ident_node);
        add_nonterm_child!(left_expr, NonTerminal::FactorIdent);

        let equal_node: Tree<FileElement<Lexem>> = descend_children!(root, 1, 0).clone();
        let localization = FileElement {
            element: true,
            len: equal_node.borrow().get_value().len,
            start_line: equal_node.borrow().get_value().get_start_line(),
            start_char: equal_node.borrow().get_value().get_start_char(),
            end_line: equal_node.borrow().get_value().get_end_line(),
        };

        return Statement::Assign(Assign::new(
            Expression::from(left_expr),
            Expression::from(right_expr),
            localization,
        ));
    }

    let id_node = Node::new(empty_file_elt!(Lexem::NonTerminal(NonTerminal::Factor)));

    // Move identifier
    id_node
        .borrow_mut()
        .add_child(&id_node, root.borrow().get_children()[0].clone());

    // Move factor id
    id_node
        .borrow_mut()
        .add_child(&id_node, descend_children!(root, 1, 0).clone());

    // Make fake left expr tree, without access
    let left_expr_tree: Tree<FileElement<Lexem>> =
        Node::new(empty_file_elt!(Lexem::NonTerminal(NonTerminal::Expr)));

    // Add identifier as factor
    left_expr_tree
        .borrow_mut()
        .add_child(&left_expr_tree, id_node);

    // Add the rest of the expression tree
    left_expr_tree
        .borrow_mut()
        .add_child(&left_expr_tree, descend_children!(root, 1, 1).clone());

    // Construct the fake tree for the parse_complex_stmt function

    let fake_root: Tree<FileElement<Lexem>> =
        Node::new(empty_file_elt!(Lexem::NonTerminal(NonTerminal::Stmt)));

    fake_root.borrow_mut().add_child(&fake_root, left_expr_tree);
    fake_root
        .borrow_mut()
        .add_child(&fake_root, descend_children!(root, 1, 2).clone());

    return parse_complex_stmt(fake_root);
}

///
///                         stmt
///                        /     \
///                     expr     simple_stmt_expr __________
///                                  /     \     \______    \
///                                 /       \           \    \
///                           [ expr ]    access_suite   =   expr
///
fn parse_complex_stmt(root: Tree<FileElement<Lexem>>) -> Statement {
    let left_expr = Expression::from(descend_children!(root, 0).clone());

    // Empty right children, simple expression
    if descend_children!(root, 1).borrow().get_children().len() == 0 {
        return Statement::Expr(left_expr);
    }

    // There is an access expr with an assignment

    let assign_value: Expression = Expression::from(descend_children!(root, 1, 5).clone());

    // Add the access expression to the rightmost factor

    fn insert_access(expr: Expression, access_root: Tree<FileElement<Lexem>>) -> Expression {
        match expr.kind {
            ExpressionKind::BINOP(e1, op, e2) => {
                ExpressionKind::BINOP(e1, op, Box::new(insert_access(*e2, access_root))).into()
            }
            ExpressionKind::UNOP(op, e) => {
                ExpressionKind::UNOP(op, Box::new(insert_access(*e, access_root))).into()
            }
            ExpressionKind::NotImplemented => ExpressionKind::NotImplemented.into(),
            ExpressionKind::Factor(_) => parse_access(access_root, expr),
        }
    }

    // Construct fake access tree
    let access_root: Tree<FileElement<Lexem>> =
        Node::new(empty_file_elt!(Lexem::NonTerminal(NonTerminal::ExprAccess)));

    // Move the first 4 childs of the simple_stmt_expr
    let simple_stmt_expr_children = descend_children!(root, 1).borrow().get_children();
    for i in 0..=3 {
        access_root
            .borrow_mut()
            .add_child(&access_root, simple_stmt_expr_children[i].clone());
    }

    let left_expr = insert_access(left_expr, access_root);

    let equal_node: Tree<FileElement<Lexem>> = descend_children!(root, 1, 2).clone();
    let localization = FileElement {
        element: true,
        len: equal_node.borrow().get_value().len,
        start_line: equal_node.borrow().get_value().get_start_line(),
        start_char: equal_node.borrow().get_value().get_start_char(),
        end_line: equal_node.borrow().get_value().get_end_line(),
    };

    return Statement::Assign(Assign::new(left_expr, assign_value, localization));
}

impl Into<Tree<String>> for Statement {
    fn into(self) -> Tree<String> {
        match self {
            Statement::NotImplemented => return Node::new(String::from("STMT (NI)")),
            Statement::Print(expr) => {
                let r = Node::new(String::from("PRINT"));
                r.borrow_mut().add_child(&r, expr.into());
                return r;
            }
            Statement::Println(expr) => {
                let r = Node::new(String::from("PRINTLN"));
                r.borrow_mut().add_child(&r, expr.into());
                return r;
            }
            Statement::Return(expr) => {
                let r = Node::new(String::from("RETURN"));
                r.borrow_mut().add_child(&r, expr.into());
                return r;
            }
            Statement::For(for_loop) => return for_loop.into(),
            Statement::Conditional(cdt) => return cdt.into(),
            Statement::Assign(assign) => return assign.into(),
            Statement::Expr(expression) => return expression.into(),
        };
    }
}

impl Localizable for Statement {
    fn get_len(&self) -> usize {
        match self {
            Statement::NotImplemented => 0,
            Statement::Print(expr) => expr.get_len(),
            Statement::Println(expr) => expr.get_len(),
            Statement::Return(expr) => expr.get_len(),
            Statement::For(for_loop) => for_loop.get_len(),
            Statement::Conditional(cdt) => cdt.get_len(),
            Statement::Assign(assign) => assign.get_len(),
            Statement::Expr(expression) => expression.get_len(),
        }
    }

    fn get_start_line(&self) -> usize {
        match self {
            Statement::NotImplemented => 0,
            Statement::Print(expr) => expr.get_start_line(),
            Statement::Println(expr) => expr.get_start_line(),
            Statement::Return(expr) => expr.get_start_line(),
            Statement::For(for_loop) => for_loop.get_start_line(),
            Statement::Conditional(cdt) => cdt.get_start_line(),
            Statement::Assign(assign) => assign.get_start_line(),
            Statement::Expr(expression) => expression.get_start_line(),
        }
    }

    fn get_end_line(&self) -> usize {
        match self {
            Statement::NotImplemented => 0,
            Statement::Print(expr) => expr.get_end_line(),
            Statement::Println(expr) => expr.get_end_line(),
            Statement::Return(expr) => expr.get_end_line(),
            Statement::For(for_loop) => for_loop.get_end_line(),
            Statement::Conditional(cdt) => cdt.get_end_line(),
            Statement::Assign(assign) => assign.get_end_line(),
            Statement::Expr(expression) => expression.get_end_line(),
        }
    }

    fn get_start_char(&self) -> usize {
        match self {
            Statement::NotImplemented => 0,
            Statement::Print(expr) => expr.get_start_char(),
            Statement::Println(expr) => expr.get_start_char(),
            Statement::Return(expr) => expr.get_start_char(),
            Statement::For(for_loop) => for_loop.get_start_char(),
            Statement::Conditional(cdt) => cdt.get_start_char(),
            Statement::Assign(assign) => assign.get_start_char(),
            Statement::Expr(expression) => expression.get_start_char(),
        }
    }

    fn get_end_char(&self) -> usize {
        match self {
            Statement::NotImplemented => 0,
            Statement::Print(expr) => expr.get_end_char(),
            Statement::Println(expr) => expr.get_end_char(),
            Statement::Return(expr) => expr.get_end_char(),
            Statement::For(for_loop) => for_loop.get_end_char(),
            Statement::Conditional(cdt) => cdt.get_end_char(),
            Statement::Assign(assign) => assign.get_end_char(),
            Statement::Expr(expression) => expression.get_end_char(),
        }
    }
}
