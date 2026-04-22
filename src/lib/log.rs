
use tokio::io::AsyncWriteExt;

pub async fn logln(msg: &str) {
    let mut file = tokio::fs::OpenOptions::new()
        .append(true)
        .open("log.txt")
        .await
        .expect("failed to open log file");
        
    // push a newline
    let mut msg_string = msg.to_string();
    msg_string.push('\n');
    let _ = file.write_all(msg_string.as_bytes()).await;
    
}

