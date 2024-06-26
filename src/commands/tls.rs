use std::env;
use std::path::PathBuf;
use std::io::{self, BufRead, BufReader, Write}; // Added BufRead here
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::fs;
use std::path::Path;

use rustls::{ServerConfig, ServerConnection};
use rustls_pemfile::{certs, pkcs8_private_keys};
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use dialoguer::Input;

// Function to load certificates from a file
fn load_certs(filename: &PathBuf) -> io::Result<Vec<CertificateDer<'static>>> {
    let certfile = fs::File::open(filename)?;
    let mut reader = BufReader::new(certfile);
    certs(&mut reader).collect()
}

// Function to load private keys from a file
fn load_keys(filename: &PathBuf) -> io::Result<PrivateKeyDer<'static>> {
    let keyfile = fs::File::open(filename)?;
    let mut reader = BufReader::new(keyfile);
    let keys = pkcs8_private_keys(&mut reader).collect::<Result<Vec<_>, _>>()?;
    keys.into_iter().next().map(|key| key.into()).ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "no keys found"))
}

// Function to handle client connections
fn handle_client(stream: TcpStream, config: Arc<ServerConfig>, project_dir: &str) -> io::Result<()> {
    let conn = ServerConnection::new(Arc::clone(&config)).unwrap();
    let mut tls = rustls::StreamOwned::new(conn, stream);
    let mut reader = BufReader::new(&mut tls);

    let mut request_line = String::new();
    reader.read_line(&mut request_line)?;

    // Parse the request (very basic parsing, you might want to use a proper HTTP parser)
    let request_parts: Vec<&str> = request_line.split_whitespace().collect();
    if request_parts.len() < 2 {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid HTTP request"));
    }

    let path = request_parts[1];
    let response = match path {
        "/" => serve_file("index.html", project_dir),
        _ => serve_file(&path[1..], project_dir), // Remove leading '/'
    };

    // Write the response
    tls.write_all(response.as_bytes())?;
    tls.flush()?;

    Ok(())
}

// Function to serve a file
fn serve_file(filename: &str, project_dir: &str) -> String {
    let path = Path::new(project_dir).join(filename);
    match fs::read_to_string(path) {
        Ok(content) => format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\n\r\n{}",
            content.len(),
            content
        ),
        Err(_) => "HTTP/1.1 404 NOT FOUND\r\n\r\n404 - Not Found".to_string(),
    }
}

pub fn run() -> io::Result<()> {
    // Get the current directory
    let current_dir = env::current_dir()?;

    // Construct paths for the certificate and key files
    let cert_path = current_dir.join("fullchain.pem");
    let key_path = current_dir.join("privkey.pem");

    // Check if the files exist
    if !cert_path.exists() || !key_path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "Certificate or key file not found. Please run the certbot program first.",
        ));
    }

    // Load certificates and private key
    let certs = load_certs(&cert_path)?;
    let key = load_keys(&key_path)?;

    // Ask for the project directory name
    let project_dir = Input::<String>::new()
        .with_prompt("Enter the name of the project directory to serve")
        .interact_text()?;

    // Check if the project directory exists
    let project_path = current_dir.join(&project_dir);
    if !project_path.exists() || !project_path.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Project directory '{}' not found", project_dir),
        ));
    }

    // Create server configuration
    let config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidInput, e))?;

    let config = Arc::new(config);

    // Create a TCP listener
    let listener = TcpListener::bind("0.0.0.0:443")?;
    println!("HTTPS server started. Listening on 0.0.0.0:443");
    println!("Serving project from directory: {}", project_dir);

    // Accept incoming connections
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("New connection accepted");
                let config = Arc::clone(&config);
                let project_dir = project_dir.clone();

                // Handle each client connection in a separate thread
                std::thread::spawn(move || {
                    if let Err(e) = handle_client(stream, config, &project_dir) {
                        eprintln!("Error in client connection: {:?}", e);
                    }
                    println!("Connection closed");
                });
            }
            Err(e) => {
                eprintln!("Error accepting connection: {}", e);
            }
        }
    }

    Ok(())
}