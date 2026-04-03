use super::*;
use clap::Parser;

/// Wrap ModuleArgs so we can parse it from a top-level binary name.
#[derive(Debug, clap::Parser)]
struct TestCli {
    #[command(subcommand)]
    command: ModuleCommand,
}

fn parse(args: &[&str]) -> ModuleCommand {
    TestCli::parse_from(args).command
}

#[test]
fn parse_create_minimal() {
    let cmd = parse(&["cli", "create", "--name", "Auth"]);
    match cmd {
        ModuleCommand::Create(a) => {
            assert_eq!(a.name, "Auth");
            assert!(a.description.is_none());
            assert!(a.parent.is_none());
        }
        _ => panic!("expected Create"),
    }
}

#[test]
fn parse_create_full() {
    let cmd = parse(&[
        "cli",
        "create",
        "--name",
        "Auth",
        "--description",
        "Authentication module",
        "--parent",
        "platform",
    ]);
    match cmd {
        ModuleCommand::Create(a) => {
            assert_eq!(a.name, "Auth");
            assert_eq!(a.description.as_deref(), Some("Authentication module"));
            assert_eq!(a.parent.as_deref(), Some("platform"));
        }
        _ => panic!("expected Create"),
    }
}

#[test]
fn parse_list_flat() {
    let cmd = parse(&["cli", "list"]);
    match cmd {
        ModuleCommand::List(a) => assert!(!a.tree),
        _ => panic!("expected List"),
    }
}

#[test]
fn parse_list_tree() {
    let cmd = parse(&["cli", "list", "--tree"]);
    match cmd {
        ModuleCommand::List(a) => assert!(a.tree),
        _ => panic!("expected List"),
    }
}

#[test]
fn parse_show() {
    let cmd = parse(&["cli", "show", "my-module"]);
    match cmd {
        ModuleCommand::Show(a) => assert_eq!(a.slug, "my-module"),
        _ => panic!("expected Show"),
    }
}

#[test]
fn parse_assign() {
    let cmd = parse(&["cli", "assign", "--module", "platform", "--feature", "auth"]);
    match cmd {
        ModuleCommand::Assign(a) => {
            assert_eq!(a.module, "platform");
            assert_eq!(a.feature, "auth");
        }
        _ => panic!("expected Assign"),
    }
}

#[test]
fn parse_tag() {
    let cmd = parse(&["cli", "tag", "--module", "platform", "--feature", "auth"]);
    match cmd {
        ModuleCommand::Tag(a) => {
            assert_eq!(a.module, "platform");
            assert_eq!(a.feature, "auth");
        }
        _ => panic!("expected Tag"),
    }
}

#[test]
fn parse_untag() {
    let cmd = parse(&["cli", "untag", "--module", "platform", "--feature", "auth"]);
    match cmd {
        ModuleCommand::Untag(a) => {
            assert_eq!(a.module, "platform");
            assert_eq!(a.feature, "auth");
        }
        _ => panic!("expected Untag"),
    }
}

#[test]
fn parse_delete() {
    let cmd = parse(&["cli", "delete", "old-module"]);
    match cmd {
        ModuleCommand::Delete(a) => assert_eq!(a.slug, "old-module"),
        _ => panic!("expected Delete"),
    }
}
