extern crate chrono;
extern crate mbedtls;
extern crate core; // Add this line

// this program will run a TLS server on port 7878, built and signed by the rust-sgx 
use chrono::prelude::*;
use mbedtls::hash::Type::Sha256;
use mbedtls::pk::Pk;
use mbedtls::rng::{CtrDrbg, EntropyCallback};
use mbedtls::ssl::config::{Endpoint, Preset, Transport};
use mbedtls::ssl::{Config, Context};
use mbedtls::x509::certificate::{Builder, Certificate};
use mbedtls::x509::Time;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use std::os::raw::{c_uchar, c_int, c_void};
use std::mem::size_of;
use core::arch::x86_64;

const RSA_KEY_SIZE: u32 = 3072;
const RSA_KEY_EXP: u32 = 0x10001;
const DAYS_TO_SECS: u64 = 86400;
const CERT_VAL_SECS: u64 = 365 * DAYS_TO_SECS;

trait ToTime {
    fn to_time(&self) -> Time;
}

impl ToTime for chrono::DateTime<Utc> {
    fn to_time(&self) -> Time {
        Time::new(
            self.year() as _,
            self.month() as _,
            self.day() as _,
            self.hour() as _,
            self.minute() as _,
            self.second() as _,
        )
        .unwrap()
    }
}

fn get_validity() -> (Time, Time) {
    let start = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let end = start + CERT_VAL_SECS;
    let not_before = Utc.timestamp_opt(start as _, 0).unwrap();
    let not_after = Utc.timestamp_opt(end as _, 0).unwrap();
    (not_before.to_time(), not_after.to_time())
}

struct Entropy;

impl EntropyCallback for Entropy {
    unsafe extern "C" fn call(_: *mut c_void, data: *mut c_uchar, len: usize) -> c_int {
        let mut outbuf = std::slice::from_raw_parts_mut(data, len);
        write_rng_to_slice(&mut outbuf, rdseed)
    }

    fn data_ptr(&self) -> *mut c_void {
        std::ptr::null_mut()
    }
}

fn write_rng_to_slice(outbuf: &mut [u8], rng: fn() -> Option<usize>) -> c_int {
    let stepsize = size_of::<usize>();

    for chunk in outbuf.chunks_mut(stepsize) {
        if let Some(val) = rng() {
            let buf = val.to_ne_bytes();
            let ptr = &buf[..chunk.len()];
            chunk.copy_from_slice(ptr);
        } else {
            return -1; // Error code
        }
    }
    0
}



fn rdseed() -> Option<usize> {
    let mut value = 0;
    for _ in 0..10 {
        if unsafe { x86_64::_rdseed64_step(&mut value) } == 1 {
            return Some(value as usize);
        }
    }
    None
}

fn get_key_and_cert() -> (Arc<Pk>, mbedtls::alloc::Box<Certificate>) {
    let entropy = Arc::new(Entropy);
    let mut ctr_drbg = CtrDrbg::new(entropy, None).unwrap();
    let key = Arc::new(Pk::generate_rsa(&mut ctr_drbg, RSA_KEY_SIZE, RSA_KEY_EXP).unwrap());
    let mut key_for_issuer = Pk::generate_rsa(&mut ctr_drbg, RSA_KEY_SIZE, RSA_KEY_EXP).unwrap();
    let (not_before, not_after) = get_validity();

    let mut key_for_subject = Pk::generate_rsa(&mut ctr_drbg, RSA_KEY_SIZE, RSA_KEY_EXP).unwrap();

    let cert = Certificate::from_der(
        &Builder::new()
            .subject_key(&mut key_for_subject)
            .subject_with_nul("CN=mbedtls-server.example\0")
            .unwrap()
            .issuer_key(&mut key_for_issuer)
            .issuer_with_nul("CN=mbedtls-server.example\0")
            .unwrap()
            .validity(not_before, not_after)
            .unwrap()
            .serial(&[5])
            .unwrap()
            .signature_hash(Sha256)
            .write_der_vec(&mut ctr_drbg)
            .unwrap(),
    )
    .unwrap();
    (key, cert)
}

#[allow(dead_code)]
#[derive(Debug)]
enum MyError {
    IoError(io::Error),
    TlsError(mbedtls::Error),
}

impl From<io::Error> for MyError {
    fn from(err: io::Error) -> MyError {
        MyError::IoError(err)
    }
}

impl From<mbedtls::Error> for MyError {
    fn from(err: mbedtls::Error) -> MyError {
        MyError::TlsError(err)
    }
}

fn serve(
    mut conn: TcpStream,
    key: Arc<Pk>,
    cert: &mbedtls::alloc::Box<Certificate>,
) -> Result<(), MyError> {
    let entropy = Arc::new(Entropy);
    let ctr_drbg = CtrDrbg::new(entropy, None).map_err(MyError::from)?;

    let mut config = Config::new(Endpoint::Server, Transport::Stream, Preset::Default);
    config.set_rng(Arc::new(ctr_drbg));
    let mut cert_list = mbedtls::alloc::List::new();
    cert_list.push(cert.clone());
    config
        .push_cert(Arc::new(cert_list), key.clone())
        .map_err(MyError::from)?;

    let mut ctx = Context::new(Arc::new(config));
    ctx.establish(&mut conn, None).map_err(MyError::from)?;

    struct SessionWrapper<'a, T: Read + Write + 'a>(&'a mut Context<T>);

    impl<'a, T: Read + Write> Read for SessionWrapper<'a, T> {
        fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
            self.0
                .read(buf)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
        }
    }

    impl<'a, T: Read + Write> Write for SessionWrapper<'a, T> {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.0
                .write(buf)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
        }

        fn flush(&mut self) -> io::Result<()> {
            self.0
                .flush()
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
        }
    }

    let mut session = SessionWrapper(&mut ctx);
    let mut reader = BufReader::new(&mut session);
    let mut read_buf = String::new();
    let mut write_buf = Vec::new();

    while let Ok(size) = reader.read_line(&mut read_buf) {
        if size == 0 {
            break;
        }
        write_buf.extend_from_slice(read_buf.as_bytes());
        read_buf.clear();
    }

    session.write_all(&write_buf).map_err(MyError::from)?;

    Ok(())
}

pub fn run() {
    let (key, cert) = get_key_and_cert();
    let listener = TcpListener::bind("0.0.0.0:7878").unwrap();

    println!("TLS server started. Listening on 0.0.0.0:7878");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("New connection accepted");
                let _ = serve(stream, key.clone(), &cert);
                println!("Connection closed");
            }
            Err(e) => {
                eprintln!("Error accepting connection: {}", e);
            }
        }
    }
}