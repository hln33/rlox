use std::{cell::RefCell, fmt::Arguments, rc::Rc, vec};

use rlox::{logger::Logger, run_file};

const TEST_FILE_DIR: &str = "test_files";

struct MockLogger {
    logs: Rc<RefCell<Vec<String>>>,
}
impl MockLogger {
    fn new() -> MockLogger {
        MockLogger {
            logs: Rc::new(RefCell::new(vec![])),
        }
    }
}
impl Logger for MockLogger {
    fn print(&mut self, value: Arguments) {
        self.logs.borrow_mut().push(value.to_string());
    }
}

fn assert_prints(file_name: &str, expected_prints: &[String]) {
    let logger = Box::new(MockLogger::new());
    let logs = logger.logs.clone();
    run_file(&format!("{TEST_FILE_DIR}/{file_name}"), Some(logger));

    assert_eq!(expected_prints.len(), logs.borrow().len());
    for (index, log) in logs.borrow().iter().enumerate() {
        assert_eq!(log.to_owned(), expected_prints[index]);
    }
}

#[test]
fn variable_declaration_and_assignment() {
    assert_prints(
        "declaration_and_assignment.lox",
        &[String::from("1"), String::from("25.1")],
    )
}

#[test]
fn expression_evaluation() {
    assert_prints("expression_eval.lox", &[String::from("15")])
}

#[test]
fn variable_scoping() {
    assert_prints(
        "variable_scoping.lox",
        &[
            String::from("inner a"),
            String::from("outer b"),
            String::from("global c"),
            String::from("outer a"),
            String::from("outer b"),
            String::from("global c"),
            String::from("global a"),
            String::from("global b"),
            String::from("global c"),
        ],
    )
}

#[test]
fn loops() {
    assert_prints(
        "loops.lox",
        &[
            String::from("0"),
            String::from("1"),
            String::from("2"),
            String::from("3"),
            String::from("4"),
        ],
    )
}

#[test]
fn function_calls() {
    assert_prints("function_calls.lox", &[String::from("Hi, Dear Reader!")])
}

#[test]
fn recursive_functions() {
    assert_prints(
        "recursive_functions.lox",
        &[
            String::from("0"),
            String::from("1"),
            String::from("1"),
            String::from("2"),
            String::from("3"),
            String::from("5"),
            String::from("8"),
            String::from("13"),
            String::from("21"),
            String::from("34"),
            String::from("55"),
            String::from("89"),
            String::from("144"),
            String::from("233"),
            String::from("377"),
            String::from("610"),
            String::from("987"),
            String::from("1597"),
            String::from("2584"),
            String::from("4181"),
        ],
    )
}

#[test]
fn closures() {
    assert_prints("closures.lox", &[String::from("1")])
}

#[test]
fn print_class_name() {
    assert_prints("print_class_name.lox", &[String::from("DevonshireCream")])
}

#[test]
fn print_class_instance() {
    assert_prints(
        "print_class_instance.lox",
        &[String::from("Bagel instance")],
    )
}

#[test]
fn basic_method() {
    assert_prints("basic_method.lox", &[String::from("inside method")]);
}

#[test]
fn bound_methods() {
    assert_prints(
        "bound_methods.lox",
        &[
            String::from("chocolate"),
            String::from("chocolate"),
            String::from("vanilla"),
            String::from("vanilla"),
            String::from("strawberry"),
            String::from("strawberry"),
        ],
    )
}

#[test]
fn init_class() {
    assert_prints(
        "init_class.lox",
        &[
            String::from("hello!"),
            String::from("hello!"),
            String::from("foo instance"),
        ],
    )
}
