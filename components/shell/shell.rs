use std::time::Duration;
use shell_exec::{Execution, Shell};

let execution = Execution::builder()
    .shell(Shell::Bash)
    .cmd(
        r#"
        INPUT=`cat -`;
        echo "hello $INPUT"
        "#
        .to_string(),
    )
    .timeout(Duration::from_millis(10000))
    .build();

let data = execution.execute(b"world").await.unwrap();

assert_eq!(b"hello world"[..], data);