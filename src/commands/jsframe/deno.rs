pub fn run() {
    println!("Creating a new Deno project...");
    let output = std::process::Command::new("sh")
        .arg("-c")
        .arg("deno init my_deno_project")
        .output()
        .expect("failed to execute process");

    if output.status.success() {
        println!("Deno project created successfully!");
    } else {
        eprintln!("Failed to create Deno project.");
    }
}
