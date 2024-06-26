use std::process::Command;

pub fn run() {
    println!("Creating a new Vue.js project...");

    // Replace this with the actual command to create a Vue.js project
    let output = Command::new("sh")
        .arg("-c")
        .arg("npx @vue/cli create vue-js-project -d")
        .output()
        .expect("Failed to execute command");

    if output.status.success() {
        println!("Vue.js project created successfully!");
    } else {
        eprintln!(
            "Failed to create Vue.js project: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
