use std::cell::RefCell;
use std::collections::HashMap;
use std::convert::Infallible;
use std::fs;
use std::io::{BufRead, BufReader};
use std::net::SocketAddr;

use hyper::{
    header::HeaderValue,
    service::{make_service_fn, service_fn},
    Body, Request, Response, Server, StatusCode,
};
use lazy_static::lazy_static;
use phpbb_pwhash::{check_hash, CheckHashResult};

thread_local! {
    static LOGINS: RefCell<HashMap<String, String>> = RefCell::new(HashMap::with_capacity(0));
}

lazy_static! {
    static ref BASIC_AUTH_REALM: String =
        std::env::var("BASIC_AUTH_REALM").unwrap_or_else(|_| "Login".to_string());
}

/// Handle a single request.
async fn handle(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    const BASIC_AUTH_PREFIX: &[u8] = b"Basic ";
    match req
        .headers()
        .get("authorization")
        .map(HeaderValue::as_bytes)
    {
        Some(auth) if auth.starts_with(BASIC_AUTH_PREFIX) => {
            handle_auth(&auth[BASIC_AUTH_PREFIX.len()..]).await
        }
        Some(_) => handle_noauth().await,
        None => handle_noauth().await,
    }
}

/// Handle a request with an authentication header
async fn handle_auth(auth_value: &[u8]) -> Result<Response<Body>, Infallible> {
    fn invalid(reason: impl AsRef<str>) -> Result<Response<Body>, Infallible> {
        println!("[handle_auth  ] Invalid ({})", reason.as_ref());
        Ok(Response::builder()
            .status(StatusCode::FORBIDDEN)
            .body(Body::from("Authentication failed"))
            .unwrap())
    }

    // Decode base64 credentials
    let decoded = match base64::decode(auth_value)
        .ok()
        .and_then(|d| String::from_utf8(d).ok())
    {
        Some(d) => d,
        None => return invalid("Base64 or UTF-8 decode failed"),
    };
    let parts: Vec<&str> = decoded.splitn(2, ':').collect();
    if parts.len() != 2 {
        return invalid("Header value splitting failed");
    }
    let username = parts[0];
    let password = parts[1];

    // Lookup credentials
    let hash: String = match LOGINS.with(|map| map.borrow().get(username).cloned()) {
        Some(hash) => hash,
        None => {
            return invalid("User not found");
        }
    };

    // Validate credentials
    match check_hash(&hash, password) {
        CheckHashResult::Valid => {
            println!("[handle_auth  ] OK");
            Ok(Response::new(Body::from("Login successful")))
        }
        CheckHashResult::Invalid => return invalid(format!("Login failed for user {}", username)),
        _ => return invalid("Invalid hash"),
    }
}

/// Handle a request with no or an invalid authentication header
async fn handle_noauth() -> Result<Response<Body>, Infallible> {
    let resp = Response::builder()
        .status(StatusCode::UNAUTHORIZED)
        .header(
            "www-authenticate",
            format!("Basic realm=\"{}\", charset=\"UTF-8\"", *BASIC_AUTH_REALM),
        )
        .body(Body::from("Basic auth missing"))
        .unwrap();
    println!("[handle_noauth] Basic auth missing");
    Ok(resp)
}

/// Read hashes from the specified file path, return a HashMap with username-hash pairs.
fn read_hashes(filepath: &str) -> std::io::Result<HashMap<String, String>> {
    let reader = BufReader::new(fs::File::open(filepath)?);
    let mut map = HashMap::new();
    for (i, line) in reader.lines().enumerate() {
        if let Ok(line) = line {
            let parts: Vec<_> = line.splitn(2, ';').collect();
            if parts.len() != 2 {
                eprintln!("Invalid entry: No semicolon found on line {}", i + 1);
                continue;
            }
            map.insert(parts[0].to_string(), parts[1].to_string());
        }
    }
    Ok(map)
}

#[tokio::main(flavor = "current_thread")] // Single-threaded runtime
async fn main() {
    // Show help
    if std::env::args().any(|arg| arg == "-h" || arg == "--help") {
        eprintln!("Usage: {}", std::env::args().next().unwrap());
        eprintln!();
        eprintln!("Configuration through env vars:");
        eprintln!();
        eprintln!("- `LOGINS_FILE`: The path to the logins file (defaults to `logins.txt`)");
        eprintln!("- `BASIC_AUTH_REALM`: The realm used for basic auth (defaults to `Login`)");
        eprintln!();
        eprintln!("Note: The logins file must contain username and password hash separated by a semicolon,");
        eprintln!("      one credentials pair per line. There should be no quoting or CSV header.");
        std::process::exit(1);
    }

    // Show version
    println!(
        "Starting traefik-phpbb-basic-auth v{}",
        env!("CARGO_PKG_VERSION")
    );

    // Read hashes
    let logins_file =
        std::env::var("LOGINS_FILE").unwrap_or_else(|_| "logins.txt".to_string());
    println!("Loading logins from {}", logins_file);
    LOGINS.with(|map| {
        let logins = match read_hashes(&logins_file) {
            Ok(map) => map,
            Err(e) => {
                eprintln!("Could not read logins file at {}: {:?}", logins_file, e);
                std::process::exit(2);
            }
        };
        map.replace(logins);
    });

    // Construct our SocketAddr to listen on...
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));

    // And a MakeService to handle each connection...
    let make_service = make_service_fn(|_conn| async { Ok::<_, Infallible>(service_fn(handle)) });

    // Then bind and serve...
    println!("Binding to {}", addr);
    let server = Server::bind(&addr).serve(make_service);

    // And run forever...
    if let Err(e) = server.await {
        eprintln!("Server error: {}", e);
    }
}
