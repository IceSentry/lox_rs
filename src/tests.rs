use crate::{
    logger::{LoggerImpl, TestLogger},
    lox::Lox,
};
use std::{cell::RefCell, rc::Rc};

fn lox_run(source: &str) -> Vec<u8> {
    let mut output = Vec::new();
    let logger = TestLogger::new(&mut output);
    let logger = Rc::new(RefCell::new(LoggerImpl::from(logger)));
    let mut lox = Lox::new(&logger, false, false);
    let result = lox.run(source);
    assert!(result.is_ok());
    output.clone()
}

fn assert_output(source: &str, expected: &str) {
    let output = lox_run(source);
    assert_eq!(
        String::from_utf8(output).expect("Not UTF-8").trim(),
        expected,
    );
}

fn assert_output_list(source: &str, expected: &[&str]) {
    let output = lox_run(source);
    let output = String::from_utf8(output).expect("Not UTF-8");
    for (i, result) in output.split('\n').enumerate() {
        if !result.is_empty() {
            assert_eq!(result, expected[i]);
        }
    }
}

#[test]
fn test_shadowing() {
    let source = r#"
        let a = 1;
        {
            let a = a + 2;
            print a; // 3
        }
    "#;
    assert_output(source, "3");
}

#[test]
fn test_block() {
    let source = r#"
        let a = "global a";
        let b = "global b";
        let c = "global c";
        {
            let a = "outer a";
            let b = "outer b";
            let d = "outer d";
            {
                let a = "inner a";
                d = "inner d";
                print a;
                print b;
                print c;
            }
            print a;
            print b;
            print c;
            print d;
        }
        print a;
        print b;
        print c;
    "#;

    assert_output_list(
        source,
        &[
            "inner a", "outer b", "global c", "outer a", "outer b", "global c", "inner d",
            "global a", "global b", "global c",
        ],
    )
}

#[test]
fn test_operator_precedence() {
    let source = r#"
        print 2 + 3 * 4 * 5 - 6;
    "#;
    assert_output(source, "56");
}

#[test]
fn test_if() {
    let source = r#"
        if true {
            print "then_branch"; // <--
        } else if false {
            print "else_if_branch";
        } else {
            print "else_branch";
        }
    "#;
    assert_output(source, "then_branch");

    let source = r#"
        if false {
            print "then_branch";
        } else if true {
            print "else_if_branch"; // <--
        } else {
            print "else_branch";
        }
    "#;
    assert_output(source, "else_if_branch");

    let source = r#"
        if false {
            print "then_branch";
        } else if false {
            print "else_if_branch";
        } else {
            print "else_branch"; // <--
        }
    "#;
    assert_output(source, "else_branch");
}

#[test]
fn test_logical_operator() {
    let source = r#"
        print "hi" or 2; // "hi".
        print nil or "yes"; // "yes".
    "#;

    assert_output_list(source, &["hi", "yes"]);
}

#[test]
fn test_while() {
    let source = r#"
        let i = 0;
        while (i < 5) {
            print i;
            i = i + 1;
        }
    "#;

    assert_output_list(source, &["0", "1", "2", "3", "4"]);
}

#[test]

fn test_for_continue_break() {
    let source = r#"
        for (var i = 0; i <= 10; i = i + 1) {
            if i == 2 or i == 3 {
                continue;
            }
            print i;
            if i >= 5 {
                break;
            }
        }
    "#;

    assert_output_list(source, &["0", "1", "4", "5"]);
}
