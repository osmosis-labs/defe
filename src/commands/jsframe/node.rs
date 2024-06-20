pub fn run() {
    println!("Creating a new Node.js project with Express...");
    let output = std::process::Command::new("sh")
        .arg("-c")
        .arg("npx express-generator my_node_project && cd my_node_project && npm install")
        .output()
        .expect("failed to execute process");

    if output.status.success() {
        println!("Node.js project created successfully!");
    } else {
        eprintln!("Failed to create Node.js project.");
    }
}
