use mechanise::anthropic_types::StreamEvent;
use regex::Regex;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

async fn read_and_process_events(file_path: &Path) {
    let file = File::open(file_path).unwrap();
    let reader = io::BufReader::new(file);

    for line in reader.lines() {
        let line = line.unwrap();
        // Process the line to extract and deserialize events
        process_line(&line);
    }
}

fn process_line(line: &str) {
    // Assuming your events are split by "event:" and need to be wrapped with braces
    let events = line.split("event: ").filter(|e| !e.is_empty());

    for event_str in events {
        // Since your events seem to be JSON objects without needing additional wrapping
        let event_json = format!(r#"{{"{}"#, correct_malformed_json(event_str)); // Adjust based on your actual event format

        // Attempt to deserialize the event JSON into the StreamEvent enum
        let event: Result<StreamEvent, _> = serde_json::from_str(&event_json);

        match event {
            Ok(_event) => {
                // Process the event
                println!("Successfully deserialized an event");
                // Here, add your logic to handle each deserialized event
            }
            Err(e) => {
                println!(
                    "Error deserializing event: {}\nFrom event string: {}",
                    e, event_json
                );
            }
        }
    }
}

fn correct_malformed_json(sanitised_line: &str) -> String {
    println!("LINEIN: {}", sanitised_line);
    // Create a regex pattern to match the incorrect keys (word characters followed by a colon without quotes)
    let re = Regex::new(r#""(\w+):"#).unwrap();

    // Replace occurrences with the correct format by wrapping the key in quotes and keeping the colon
    let corrected_line = re.replace_all(sanitised_line, r#""$1": "#).to_string();

    println!("LINEOUT: {}", corrected_line);
    corrected_line
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Adjust the path according to where your file is located
    let path = Path::new("./chunks.jsonl");
    tokio::runtime::Runtime::new()?.block_on(read_and_process_events(path));

    Ok(())
}
