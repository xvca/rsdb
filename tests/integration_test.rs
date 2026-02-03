use std::process::{Command, Stdio};
use std::io::Write;

fn run_script(commands: Vec<&str>) -> Vec<String> {
    use std::time::{SystemTime, UNIX_EPOCH};

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let db_file = format!("test_{}.db", timestamp);

    let mut child = Command::new("cargo")
        .args(&["run", "--quiet", "--", &db_file])
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

#[test]
fn test_persistence_single_session() {
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let db_file = format!("test_persist_{}.db", timestamp);

    // session 1: insert data
    let result1 = run_script_with_file(
        vec![
            "insert 1 alice alice@example.com",
            "insert 2 bob bob@example.com",
            ".exit",
        ],
        &db_file,
    );
    assert!(result1.iter().any(|line| line == "executed."));

    // session 2: read data back
    let result2 = run_script_with_file(vec!["select", ".exit"], &db_file);
    assert!(result2.iter().any(|line| line.contains("alice")));
    assert!(result2.iter().any(|line| line.contains("bob")));

    // cleanup
    let _ = fs::remove_file(&db_file);
}

#[test]
fn test_persistence_multiple_sessions() {
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let db_file = format!("test_multi_{}.db", timestamp);

    // session 1: insert 2 rows
    run_script_with_file(
        vec![
            "insert 1 user1 user1@example.com",
            "insert 2 user2 user2@example.com",
            ".exit",
        ],
        &db_file,
    );

    // session 2: insert 1 more row
    run_script_with_file(
        vec!["insert 3 user3 user3@example.com", ".exit"],
        &db_file,
    );

    // session 3: verify all 3 rows exist
    let result = run_script_with_file(vec!["select", ".exit"], &db_file);
    assert!(result.iter().any(|line| line.contains("user1")));
    assert!(result.iter().any(|line| line.contains("user2")));
    assert!(result.iter().any(|line| line.contains("user3")));

    // cleanup
    let _ = fs::remove_file(&db_file);
}

fn run_script_with_file(commands: Vec<&str>, db_file: &str) -> Vec<String> {
    let mut child = Command::new("cargo")
        .args(&["run", "--quiet", "--", db_file])
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
