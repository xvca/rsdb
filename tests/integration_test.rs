use std::process::{Command, Stdio};
use std::io::Write;

fn run_script(commands: Vec<&str>) -> Vec<String> {
    let mut child = Command::new("cargo")
        .args(&["run", "--quiet", "--"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to spawn child process");

    {
        let stdin = child.stdin.as_mut().expect("failed to open stdin");
        for cmd in commands {
            writeln!(stdin, "{}", cmd).expect("failed to write to stdin");
        }
    }

    let output = child.wait_with_output().expect("failed to wait on child");
    let stdout = String::from_utf8_lossy(&output.stdout);

    stdout
        .lines()
        .map(|line| {
            if line.starts_with("db > ") {
                &line[5..]
            } else {
                line
            }
        })
        .filter(|line| !line.trim().is_empty())
        .map(|s| s.to_string())
        .collect()
}

#[test]
fn test_insert_and_retrieve_row() {
    let result = run_script(vec![
        "insert 1 user1 person1@example.com",
        "select",
        ".exit",
    ]);

    assert_eq!(result[0], "executed.");
    assert_eq!(result[1], "(1, user1, person1@example.com)");
    assert_eq!(result[2], "executed.");
}

#[test]
fn test_table_full_error() {
    let mut commands = vec![];
    for i in 1..=1401 {
        commands.push(format!("insert {} user{} person{}@example.com", i, i, i));
    }
    commands.push(".exit".to_string());

    let script: Vec<&str> = commands.iter().map(|s| s.as_str()).collect();
    let result = run_script(script);

    assert!(result.iter().any(|line| line.contains("Table full")));
}

#[test]
fn test_max_length_strings() {
    let username = "a".repeat(32);
    let email = "a".repeat(255);
    let result = run_script(vec![
        &format!("insert 1 {} {}", username, email),
        "select",
        ".exit",
    ]);

    assert_eq!(result[0], "executed.");
    assert!(result[1].contains(&username));
    assert!(result[1].contains(&email));
}

#[test]
fn test_string_too_long() {
    let long_username = "a".repeat(33);
    let result = run_script(vec![
        &format!("insert 1 {} test@example.com", long_username),
        ".exit",
    ]);

    assert!(result.iter().any(|line| line.contains("string is too long")));

    let long_email = "a".repeat(256);
    let result = run_script(vec![
        &format!("insert 1 user {}", long_email),
        ".exit",
    ]);

    assert!(result.iter().any(|line| line.contains("string is too long")));
}

#[test]
fn test_negative_id() {
    let result = run_script(vec![
        "insert -1 user test@example.com",
        ".exit",
    ]);

    assert!(result.iter().any(|line| line.contains("syntax error") || line.contains("could not parse")));
}
