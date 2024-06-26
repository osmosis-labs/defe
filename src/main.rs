// Import everything from the colored crate
use dotenv::dotenv;


fn main() {
    dotenv().ok(); // Load .env file if it exists
   
    dfe_lib::print_welcome_defe_message();
    dfe_lib::run_defe_menu();
}


