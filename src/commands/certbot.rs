use crate::*;
use dialoguer::{Confirm, Input};
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
// this program will fetch the SSL chain spec from Let's Encrypt using the Certbot toolkit in rust-sgx
pub fn run() {
    println!("Running defe-certbot");

    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        // Set up logging
        let log_file = PathBuf::from("dfe_certbot_helper.log");
        let mut log_file = File::create(&log_file).expect("Failed to create log file");

        macro_rules! log_error {
            ($($arg:tt)*) => {
                let error_message = format!($($arg)*);
                print_certbot_error_message(&error_message);
                writeln!(log_file, "{}", error_message).expect("Failed to write to log file");
            };
        }

        // Check for the --help flag
        let args: Vec<String> = env::args().collect();
        if args.len() > 1 && args[1] == "--help" {
            print_help_message_certbot();
            return;
        }

        print_welcome_message_certbot();

        // Get the user's current directory
        let current_dir = match env::current_dir() {
            Ok(dir) => dir,
            Err(e) => {
                log_error!("Failed to get the current directory: {}", e);
                return;
            }
        };

        println!("Current directory: {}", current_dir.display());

        // Prompt the user to confirm or change the directory
        let confirm = Confirm::new()
            .with_prompt("Do you want to run Certbot in the current directory?")
            .interact()
            .unwrap();

        let mut target_dir = current_dir.clone();
        if !confirm {
            let new_dir = Input::<String>::new()
                .with_prompt("Enter the path to the enclave's directory")
                .interact_text()
                .unwrap();

            target_dir = PathBuf::from(new_dir);
            if !target_dir.exists() {
                log_error!("The specified directory does not exist.");
                return;
            }
        }

        confirm_domain_configuration_certbot();

        // Prompt for domain name
        let domain_name = Input::<String>::new()
            .with_prompt("Enter your domain name (e.g., example.com)")
            .interact_text()
            .unwrap();

        // Prompt for email address
        let email_address = Input::<String>::new()
            .with_prompt("Enter your email address (for urgent renewal and security notices)")
            .interact_text()
            .unwrap();

        // Confirm the entered information
        let confirm = Confirm::new()
            .with_prompt(format!("Domain: {}\nEmail: {}\n\nPlease confirm that your domain is properly configured and pointing to your server's IP address.\nAlso, ensure that your server is accessible from the internet on port 80 (HTTP) and 443 (HTTPS).\n\nIs this information correct and are you ready to proceed?", domain_name, email_address))
            .interact()
            .unwrap();

        if !confirm {
            println!("Exiting...");
            return;
        }

        println!("Running Certbot to obtain the SSL/TLS certificate and key files...");

        // Run Certbot to obtain the certificate and key files in the target directory
        let output = match Command::new("certbot")
            .arg("certonly")
            .arg("--standalone")
            .arg("--noninteractive")
            .arg("--agree-tos")
            .arg(format!("--email={}", email_address))
            .arg(format!("--domain={}", domain_name))
            .arg("--cert-path")
            .arg(target_dir.join("fullchain.pem"))
            .arg("--key-path")
            .arg(target_dir.join("privkey.pem"))
            .output()
        {
            Ok(output) => output,
            Err(e) => {
                log_error!("Failed to execute Certbot: {}", e);
                return;
            }
        };

        if output.status.success() {
            print_success_message_certbot(&target_dir);
        } else {
            let error_message = String::from_utf8_lossy(&output.stderr);
            print_certbot_error_message(&error_message);
        }
    });
}
