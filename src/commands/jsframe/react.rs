use std::process::Command;

pub fn run() {
    println!("Creating a new React project...");

    // Replace this with the actual command to create a React project
    let output = Command::new("sh")
        .arg("-c")
        .arg("npx create-react-app react-project")
        .output()
        .expect("Failed to execute command");

    if output.status.success() {
        println!("React project created successfully!");
    } else {
        eprintln!(
            "Failed to create React project: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
