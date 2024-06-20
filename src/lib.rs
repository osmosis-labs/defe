use dialoguer::Input;
use colored::*; 
use dialoguer::{theme::ColorfulTheme, Select};
pub mod commands;


pub const HELP_MESSAGE: &str = r#"
DFE Certbot Helper

This program assists in obtaining SSL/TLS certificates using Certbot.

Usage:
  dfe-certbot-helper [--help]

Options:
  --help    Display this help message.

For more information, visit https://example.com/dfe-certbot-helper
"#;

pub fn print_welcome_message_certbot() {
    println!("{}", "Welcome to the DFE Certbot Helper!".bright_green());
}

pub fn print_help_message_certbot() {
    println!("{}", HELP_MESSAGE.bright_yellow());
}

pub fn print_navigation_help_certbot(target_dir: &std::path::PathBuf) {
    println!("\n{}", "Next steps:".bright_blue());
    println!("1. Ensure that the 'fullchain.pem' and 'privkey.pem' files are in your enclave's directory.");
    println!("2. Configure your enclave to use the generated certificate and key files.");
    println!("3. Build and run your enclave.");
    println!("\nFor more information and detailed instructions, visit:");
    println!("{}", "https://example.com/enclave-setup".underline());
    println!("\n{}", format!("Certificate and key files saved in: {}", target_dir.display()).bright_green());
}

pub fn print_success_message_certbot(target_dir: &std::path::PathBuf) {
    println!("{}", "SSL/TLS certificate and key files generated successfully!".bright_green());
    print_navigation_help_certbot(target_dir);
}

pub fn confirm_domain_configuration_certbot() {
    println!("Before proceeding, please ensure that you have completed the following steps:");
    println!("1. Log in to your domain registrar's control panel (e.g., GoDaddy, Namecheap, etc.).");
    println!("2. Navigate to the DNS management section for your domain.");
    println!("3. Create an A record that points your domain name to your server's IP address.");
    println!("   - If you want to obtain a certificate for a subdomain (e.g., www.example.com), create an A record for the subdomain as well.");
    println!("4. Save the DNS changes and wait for the changes to propagate. This can take some time (usually a few minutes to a few hours).");
    println!("5. Verify that your domain is properly pointing to your server's IP address by running the following command in your terminal:");
    println!("   ping your_domain_name");

    let _domain_config_confirm = Input::<String>::new()
        .with_prompt("Please type 'I have' to confirm that you have completed the above steps")
        .validate_with(|input: &String| -> Result<(), &str> {
            if input.trim().to_lowercase() == "i have" {
                Ok(())
            } else {
                Err("Please type 'I have' to confirm")
            }
        })
        .interact_text()
        .unwrap();
}


pub fn print_certbot_error_message(error_message: &str) {
    eprintln!("{}", format!("Error running Certbot: {}", error_message).bright_red());
    eprintln!("{}", "Please check the error message and ensure that your domain is properly configured and accessible.".bright_red());
}

pub fn print_welcome_defe_message() {
    let art = r#"
            _____                    _____                    _____                    _____          
           /\    \                  /\    \                  /\    \                  /\    \         
          /::\    \                /::\    \                /::\    \                /::\    \        
         /::::\    \              /::::\    \              /::::\    \              /::::\    \       
        /::::::\    \            /::::::\    \            /::::::\    \            /::::::\    \      
       /:::/\:::\    \          /:::/\:::\    \          /:::/\:::\    \          /:::/\:::\    \     
      /:::/  \:::\    \        /:::/__\:::\    \        /:::/__\:::\    \        /:::/__\:::\    \    
     /:::/    \:::\    \      /::::\   \:::\    \      /::::\   \:::\    \      /::::\   \:::\    \   
    /:::/    / \:::\    \    /::::::\   \:::\    \    /::::::\   \:::\    \    /::::::\   \:::\    \  
   /:::/    /   \:::\ ___\  /:::/\:::\   \:::\    \  /:::/\:::\   \:::\    \  /:::/\:::\   \:::\    \ 
  /:::/____/     \:::|    |/:::/__\:::\   \:::\____\/:::/  \:::\   \:::\____\/:::/__\:::\   \:::\____\
  \:::\    \     /:::|____|\:::\   \:::\   \::/    /\::/    \:::\   \::/    /\:::\   \:::\   \::/    /
   \:::\    \   /:::/    /  \:::\   \:::\   \/____/  \/____/ \:::\   \/____/  \:::\   \:::\   \/____/ 
    \:::\    \ /:::/    /    \:::\   \:::\    \               \:::\    \       \:::\   \:::\    \     
     \:::\    /:::/    /      \:::\   \:::\____\               \:::\____\       \:::\   \:::\____\    
      \:::\  /:::/    /        \:::\   \::/    /                \::/    /        \:::\   \::/    /    
       \:::\/:::/    /          \:::\   \/____/                  \/____/          \:::\   \/____/     
        \::::::/    /            \:::\    \                                        \:::\    \         
         \::::/    /              \:::\____\                                        \:::\____\        
          \::/____/                \::/    /                                         \::/    /        
           ~~                       \/____/                                           \/____/         
                                                                                                      
"#;

    let colored_art = art
        .replace("_", &"_".truecolor(30, 144, 255).to_string())  // Dodger Blue
        .replace("/", &"/".truecolor(0, 255, 127).to_string())   // Spring Green
        .replace("\\", &"\\".truecolor(255, 20, 147).to_string()) // Deep Pink
        .replace("|", &"|".truecolor(255, 105, 180).to_string()) // Hot Pink
        .replace("~", &"~".truecolor(255, 69, 0).to_string())    // Orange Red
        .replace(" ", &" ".truecolor(128, 0, 128).to_string());  // Purple

    println!("{}", colored_art);
    println!();
}


pub fn run_defe_menu() {
    loop {
        let selections = vec![
            "Run defe-certbot",
            "Run defe-fetcher",
            "Run defe-tls",
            "Run defe-server",
            "Create new project",
            "Exit",
        ];

        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Please select a command:")
            .default(0)
            .items(&selections)
            .interact()
            .unwrap();

        match selection {
            0 => commands::certbot::run(),
            1 => commands::fetcher::run(),
            2 => commands::tls::run(),
            3 => commands::server::run(),
            4 => handle_new_defe_project(),
            5 => {
                println!("Exiting...");
                break;
            }
            _ => {}
        }
    }
}

pub fn handle_new_defe_project() {
    loop {
        let selections = vec![
            "Create new Deno project",
            "Create new Node.js project",
            "Create new React project",
            "Create new Vue.js project",
            "Back",
        ];

        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select a project type to create:")
            .default(0)
            .items(&selections)
            .interact()
            .unwrap();

        match selection {
            0 => commands::jsframe::deno::run(),
            1 => commands::jsframe::node::run(),
            2 => commands::jsframe::react::run(),
            3 => commands::jsframe::vue::run(),
            4 => break,
            _ => {}
        }
    }
}