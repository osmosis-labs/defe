use aes_gcm::{aead::{Aead, KeyInit}, Aes256Gcm, Nonce};
use base64::{engine::general_purpose, Engine as _};
use crossterm::{
    cursor, execute, queue,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor, Stylize},
    terminal::{self, Clear, ClearType},
    event::{self, Event, KeyCode},
};
use hmac::Hmac;
use pbkdf2::pbkdf2;
use rand::prelude::SliceRandom;
use rand::{thread_rng, Rng};
use rpassword;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::{
    collections::HashMap,
    error::Error,
    fs::File,
    io::{self, Read, Write},
    thread,
    time::Duration,
};
use zxcvbn::{zxcvbn, Score};

type Result<T> = std::result::Result<T, Box<dyn Error>>;

const TERMINAL_WIDTH: u16 = 80;
const CONFIG_FILE: &str = ".enigma_config";
const COLORS: [&str; 4] = ["Red", "Green", "Blue", "Yellow"];
const SYMBOLS: [&str; 4] = ["♦", "♥", "♠", "♣"];
const DIRECTIONS: [&str; 4] = ["Left", "Up", "Right", "Down"];
const SALT_LENGTH: usize = 16;
const NONCE_LENGTH: usize = 12;

const PRIMARY_COLOR: Color = Color::Cyan;
const SECONDARY_COLOR: Color = Color::Yellow;
const SUCCESS_COLOR: Color = Color::Green;
const ERROR_COLOR: Color = Color::Red;
const WARNING_COLOR: Color = Color::DarkYellow;

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Config {
    color_to_symbol: HashMap<String, String>,
    direction_to_symbol: HashMap<String, String>,
    encrypted_password: String,
    salt: String,
    nonce: String,
}

impl Config {
    fn new() -> Self {
        Self {
            color_to_symbol: HashMap::new(),
            direction_to_symbol: HashMap::new(),
            encrypted_password: String::new(),
            salt: general_purpose::STANDARD.encode(thread_rng().gen::<[u8; SALT_LENGTH]>()),
            nonce: general_purpose::STANDARD.encode(thread_rng().gen::<[u8; NONCE_LENGTH]>()),
        }
    }

    fn load_or_create() -> Result<Self> {
        match File::open(CONFIG_FILE) {
            Ok(mut file) => {
                let mut contents = String::new();
                file.read_to_string(&mut contents)?;
                let config: Config = serde_json::from_str(&contents)?;
                Ok(config)
            }
            Err(_) => {
                let config = Self::new();
                config.save()?;
                Ok(config)
            }
        }
    }

    fn save(&self) -> Result<()> {
        let config_str = serde_json::to_string(self)?;
        let mut file = File::create(CONFIG_FILE)?;
        file.write_all(config_str.as_bytes())?;
        Ok(())
    }

    fn setup_color_to_symbol(&mut self) -> Result<()> {
        clear_screen()?;
        print_title("Color to Symbol Mapping Setup")?;
        println!("Assign a symbol to each color:\n");

        for color in COLORS.iter() {
            println!("{}", format!("For {}:", color).bold());
            for (i, symbol) in SYMBOLS.iter().enumerate() {
                println!("  {}. {}", i + 1, symbol);
            }
            let choice: usize =
                get_user_input(&format!("Enter your choice for {} (1-4): ", color))?.parse()?;
            self.color_to_symbol
                .insert(color.to_string(), SYMBOLS[choice - 1].to_string());
            println!();
        }
        Ok(())
    }

    fn setup_direction_to_symbol(&mut self) -> Result<()> {
        clear_screen()?;
        print_title("Direction to Symbol Mapping Setup")?;
        println!("Assign a symbol to each direction:\n");

        for direction in DIRECTIONS.iter() {
            println!("{}", format!("For {}:", direction).bold());
            for (i, symbol) in SYMBOLS.iter().enumerate() {
                println!("  {}. {}", i + 1, symbol);
            }
            let choice: usize =
                get_user_input(&format!("Enter your choice for {} (1-4): ", direction))?.parse()?;
            self.direction_to_symbol
                .insert(direction.to_string(), SYMBOLS[choice - 1].to_string());
            println!();
        }
        Ok(())
    }

    fn set_password(&mut self, password: &str) -> Result<()> {
        if self.salt.is_empty() {
            self.salt = general_purpose::STANDARD.encode(thread_rng().gen::<[u8; SALT_LENGTH]>());
        }
        let salt = general_purpose::STANDARD.decode(&self.salt)?;
        let key = derive_fixed_key(&salt)?;
        let cipher = Aes256Gcm::new(&key.into());
        let nonce = thread_rng().gen::<[u8; NONCE_LENGTH]>();
        self.nonce = general_purpose::STANDARD.encode(nonce);
        let encrypted_password = cipher
            .encrypt(&Nonce::from_slice(&nonce), password.as_bytes())
            .map_err(|e| format!("Encryption error: {}", e))?;
        self.encrypted_password = general_purpose::STANDARD.encode(encrypted_password);
        Ok(())
    }
}


fn derive_fixed_key(salt: &[u8]) -> Result<[u8; 32]> {
    let mut key = [0u8; 32];
    pbkdf2::<Hmac<Sha256>>("RosarioEnigmaFixedKey".as_bytes(), salt, 10000, &mut key)
        .map_err(|e| format!("Key derivation error: {}", e))?;
    Ok(key)
}

fn clear_screen() -> Result<()> {
    execute!(io::stdout(), Clear(ClearType::All), cursor::MoveTo(0, 0))?;
    Ok(())
}

fn print_centered(text: &str, color: Option<Color>) -> Result<()> {
    let text_length = text.len();
    let padding = if text_length > TERMINAL_WIDTH as usize {
        0
    } else {
        (TERMINAL_WIDTH as usize - text_length) / 2
    };
    let styled_text = match color {
        Some(c) => text.with(c).to_string(),
        None => text.to_string(),
    };
    execute!(
        io::stdout(),
        cursor::MoveToColumn(padding as u16),
        Print(styled_text),
        cursor::MoveToNextLine(1)
    )?;
    Ok(())
}

fn print_title(text: &str) -> Result<()> {
    let styled_text = text.bold().with(PRIMARY_COLOR);
    print_centered(&styled_text.to_string(), None)?;
    print_separator()?;
    Ok(())
}

fn print_separator() -> Result<()> {
    print_centered(&"═".repeat(TERMINAL_WIDTH as usize - 4), Some(PRIMARY_COLOR))?;
    Ok(())
}

fn generate_alphanumeric_grid(password: &str, index: usize) -> (Vec<Vec<char>>, char) {
    let mut chars: Vec<char> = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789"
        .chars()
        .collect();
    let mut grid = vec![vec![' '; 4]; 8];
    let mut rng = thread_rng();

    let password_char = password.chars().nth(index).unwrap();
    chars.retain(|&c| c != password_char);
    chars.shuffle(&mut rng);

    // Fill the grid with random characters
    for row in grid.iter_mut() {
        for cell in row.iter_mut() {
            *cell = chars.pop().unwrap_or_else(|| rng.sample(rand::distributions::Alphanumeric) as char);
        }
    }

    // Place the password character in the correct color zone
    let (row_start, col_start) = match index % 4 {
        0 => (0, 0), // Red
        1 => (4, 0), // Blue
        2 => (4, 2), // Yellow
        3 => (0, 2), // Green
        _ => unreachable!(),
    };
    let row = row_start + rng.gen_range(0..2);
    let col = col_start + rng.gen_range(0..2);
    grid[row][col] = password_char;

    (grid, password_char)
}


fn display_grid(grid: &Vec<Vec<char>>) -> Result<()> {
    let mut stdout = io::stdout();
    let top_padding = 2;
    let left_padding = (TERMINAL_WIDTH as usize - 19) / 2;

    execute!(stdout, Clear(ClearType::All))?;

    queue!(stdout, cursor::MoveTo(left_padding as u16, top_padding as u16))?;
    queue!(stdout, Print("╔═══╦═══╦═══╦═══╗"))?;

    for (i, row) in grid.iter().enumerate() {
        queue!(stdout, cursor::MoveToNextLine(1), cursor::MoveToColumn(left_padding as u16))?;
        queue!(stdout, Print("║"))?;
        for (j, &cell) in row.iter().enumerate() {
            let color = match (i / 2, j) {
                (0, 0) | (0, 1) | (1, 0) | (1, 1) => Color::Red,
                (0, 2) | (0, 3) | (1, 2) | (1, 3) => Color::Green,
                (2, 0) | (2, 1) | (3, 0) | (3, 1) => Color::Blue,
                _ => Color::Yellow,
            };
            queue!(
                stdout,
                SetBackgroundColor(color),
                SetForegroundColor(Color::Black),
                Print(format!(" {} ", cell)),
                ResetColor,
                Print("║")
            )?;
        }
        if i < grid.len() - 1 {
            queue!(stdout, cursor::MoveToNextLine(1), cursor::MoveToColumn(left_padding as u16))?;
            queue!(stdout, Print("╠═══╬═══╬═══╬═══╣"))?;
        }
    }
    queue!(stdout, cursor::MoveToNextLine(1), cursor::MoveToColumn(left_padding as u16))?;
    queue!(stdout, Print("╚═══╩═══╩═══╩═══╝"))?;

    stdout.flush()?;
    Ok(())
}

fn get_arrow_input() -> Result<KeyCode> {
    loop {
        if let Event::Key(key_event) = event::read()? {
            match key_event.code {
                KeyCode::Left | KeyCode::Up | KeyCode::Right | KeyCode::Down => {
                    return Ok(key_event.code)
                }
                KeyCode::Esc => return Err("User interrupted".into()),
                _ => continue,
            }
        }
    }
}

fn show_loading_animation() -> Result<()> {
    let spinner = ['◐', '◓', '◑', '◒'];
    for i in 0..10 {
        print!("\r{} Generating grid...", spinner[i % 4].to_string().blue());
        io::stdout().flush()?;
        thread::sleep(Duration::from_millis(50));
    }
    print!("\r                      \r");
    Ok(())
}

fn validate_password(config: &Config) -> Result<bool> {
    let mut stdout = io::stdout();
    execute!(stdout, terminal::EnterAlternateScreen, cursor::Hide)?;
    terminal::enable_raw_mode()?;

    let result = (|| -> Result<bool> {
        let salt = general_purpose::STANDARD.decode(&config.salt)
            .map_err(|e| format!("Failed to decode salt: {}", e))?;
        let key = derive_fixed_key(&salt)?;
        let cipher = Aes256Gcm::new(&key.into());
        let nonce_bytes = general_purpose::STANDARD.decode(&config.nonce)
            .map_err(|e| format!("Failed to decode nonce: {}", e))?;
        let nonce = Nonce::from_slice(&nonce_bytes);
        let encrypted_password = general_purpose::STANDARD.decode(&config.encrypted_password)
            .map_err(|e| format!("Failed to decode encrypted password: {}", e))?;
        let decrypted_password = cipher
            .decrypt(nonce, encrypted_password.as_slice())
            .map_err(|e| format!("Decryption error: {}", e))?;
        let decrypted_password = String::from_utf8(decrypted_password)
            .map_err(|e| format!("Failed to convert decrypted password to string: {}", e))?;

        let password_length = decrypted_password.len();
        let mut correct_selections = 0;

        for index in 0..password_length {
            clear_screen()?;
            print_title("Password Validation")?;
            print_centered("Enter your password using arrow keys:", None)?;
            print_centered("← : Left, ↑ : Up, → : Right, ↓ : Down", None)?;
            print_centered("Press ESC to cancel at any time.", None)?;
            print_separator()?;

            show_loading_animation()?;
            let (grid, _) = generate_alphanumeric_grid(&decrypted_password, index);
            display_grid(&grid)?;

            let grid_bottom = 21;
            queue!(
                stdout,
                cursor::MoveTo(0, grid_bottom),
                Print(&format!("Enter direction for character {}: ", index + 1))
            )?;
            stdout.flush()?;

            let entered_direction = match get_arrow_input()? {
                KeyCode::Left => "Left",
                KeyCode::Up => "Up",
                KeyCode::Right => "Right",
                KeyCode::Down => "Down",
                _ => {
                    print_centered("Password validation cancelled.", Some(WARNING_COLOR))?;
                    thread::sleep(Duration::from_secs(2));
                    return Ok(false);
                }
            };

            let expected_color = match index % 4 {
                0 => "Red",
                1 => "Blue",
                2 => "Yellow",
                3 => "Green",
                _ => unreachable!(),
            };

            let entered_symbol = &config.direction_to_symbol[entered_direction];
            let entered_color = config.color_to_symbol
                .iter()
                .find(|(_, symbol)| *symbol == entered_symbol)
                .map(|(color, _)| color)
                .ok_or_else(|| "Invalid symbol-color mapping".to_string())?;

            if entered_color == expected_color {
                correct_selections += 1;
            }
        }

        clear_screen()?;
        print_title("Password Validation Result")?;
        
        let is_valid = correct_selections == password_length;
        if is_valid {
            print_centered("Password validated successfully!", Some(SUCCESS_COLOR))?;
        } else {
            print_centered("Password validation failed.", Some(ERROR_COLOR))?;
        }
        print_separator()?;
        print_centered(&format!("Correct selections: {}/{}", correct_selections, password_length), None)?;
        print_centered("Press Enter to continue...", None)?;
        
        // Wait for Enter key
        while event::read()? != Event::Key(KeyCode::Enter.into()) {}

        Ok(is_valid)
    })();

    terminal::disable_raw_mode()?;
    execute!(stdout, terminal::LeaveAlternateScreen, cursor::Show)?;

    result
}

fn get_user_input(prompt: &str) -> Result<String> {
    print!("{}", prompt.blue());
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

fn get_secure_password() -> Result<String> {
    loop {
        clear_screen()?;
        print_title("Password Setup")?;
        print_centered("Enter a secure password (alphanumeric only):", None)?;
        let password = rpassword::prompt_password("")?;
        if !password.chars().all(char::is_alphanumeric) {
            print_centered("Error: Password must contain only letters and numbers.", Some(ERROR_COLOR))?;
            thread::sleep(Duration::from_secs(1));
            continue;
        }
        let entropy = zxcvbn(&password, &[]);

        if entropy.score() < Score::Three {
            clear_screen()?;
            print_title("Weak Password Warning")?;
            print_centered("The password you entered is considered weak.", Some(WARNING_COLOR))?;
            print_centered("Suggestions for a strong password:", None)?;
            print_centered("1. Use a mix of uppercase and lowercase letters.", None)?;
            print_centered("2. Include numbers.", None)?;
            print_centered("3. Make it at least 8 characters long.", None)?;
            print_centered("4. Avoid common words or phrases.", None)?;
            print_centered("5. Use a unique password for each account.", None)?;
            print_separator()?;
            let choice = get_user_input("Do you want to use this password anyway? (y/n): ")?;
            if choice.to_lowercase() == "y" {
                return Ok(password);
            }
        } else {
            return Ok(password);
        }
    }
}

fn display_menu() -> Result<()> {
    clear_screen()?;
    print_title("Rosario's Enigma Encryption Tool - Main Menu")?;
    print_centered("1. Generate new grid and validate password", Some(SECONDARY_COLOR))?;
    print_centered("2. View current configuration", Some(SECONDARY_COLOR))?;
    print_centered("3. Reset configuration", Some(SECONDARY_COLOR))?;
    print_centered("4. Exit", Some(SECONDARY_COLOR))?;
    print_separator()?;
    Ok(())
}

pub fn run() -> Result<()> {
    let mut stdout = io::stdout();
    execute!(stdout, terminal::EnterAlternateScreen)?;
    let result = (|| -> Result<()> {
        clear_screen()?;
        print_title("Rosario's Password Configuration Ceremony")?;
        thread::sleep(Duration::from_secs(1));

        let mut config = Config::load_or_create()?;

        if config.color_to_symbol.is_empty()
            || config.direction_to_symbol.is_empty()
            || config.encrypted_password.is_empty()
        {
            clear_screen()?;
            print_title("Initial Configuration Setup")?;
            print_centered("Let's set up your personalized configuration.", None)?;
            thread::sleep(Duration::from_secs(2));

            config.setup_color_to_symbol()?;
            config.setup_direction_to_symbol()?;

            let password = get_secure_password()?;
            config.set_password(&password)?;

            config.save()?;
            clear_screen()?;
            print_centered("Configuration saved successfully!", Some(SUCCESS_COLOR))?;
            thread::sleep(Duration::from_secs(1));
        }

        loop {
            display_menu()?;
            let choice = get_user_input("Enter your choice (1-4): ")?;

            match choice.as_str() {
                "1" => {
                    match validate_password(&config) {
                        Ok(true) => {
                            print_centered("Password validated successfully!", Some(SUCCESS_COLOR))?;
                            thread::sleep(Duration::from_secs(2));
                            break;
                        }
                        Ok(false) => print_centered("Password validation failed.", Some(ERROR_COLOR))?,
                        Err(e) => print_centered(&format!("An error occurred: {}", e), Some(ERROR_COLOR))?,
                    }
                    thread::sleep(Duration::from_secs(2));
                }
                "2" => {
                    clear_screen()?;
                    print_title("Current Configuration")?;
                    print_centered("Color to Symbol mapping:", None)?;
                    for (color, symbol) in &config.color_to_symbol {
                        print_centered(&format!("  {} -> {}", color, symbol), None)?;
                    }
                    print_centered("\nDirection to Symbol mapping:", None)?;
                    for (direction, symbol) in &config.direction_to_symbol {
                        print_centered(&format!("  {} -> {}", direction, symbol), None)?;
                    }
                    print_centered(&format!("\nEncrypted Password: {}", config.encrypted_password), None)?;
                    print_separator()?;
                    get_user_input("Press Enter to continue...")?;
                }
                "3" => {
                    clear_screen()?;
                    print_title("Reset Configuration")?;
                    print_centered("Warning: This will reset your entire configuration.", Some(WARNING_COLOR))?;
                    let confirm = get_user_input("Are you sure you want to proceed? (y/n): ")?;
                    if confirm.to_lowercase() == "y" {
                        let second_confirm = get_user_input("This action is irreversible. Type 'RESET' to confirm: ")?;
                        if second_confirm == "RESET" {
                            config = Config::new();
                            config.save()?;
                            print_centered("Configuration reset. Please restart the program to set up new configuration.", Some(SUCCESS_COLOR))?;
                            thread::sleep(Duration::from_secs(2));
                            return Ok(());
                        } else {
                            print_centered("Reset cancelled.", Some(WARNING_COLOR))?;
                            thread::sleep(Duration::from_secs(2));
                        }
                    }
                }
                "4" => {
                    clear_screen()?;
                    print_title("Goodbye")?;
                    thread::sleep(Duration::from_secs(2));
                    break;
                }
                _ => {
                    print_centered("Invalid choice. Please try again.", Some(ERROR_COLOR))?;
                    thread::sleep(Duration::from_secs(2));
                }
            }
        }

        Ok(())
    })();

    execute!(stdout, terminal::LeaveAlternateScreen)?;

    if let Err(ref e) = result {
        eprintln!("An error occurred: {}", e);
    }

    result
}