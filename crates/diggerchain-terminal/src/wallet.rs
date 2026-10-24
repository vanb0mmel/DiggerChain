use std::fs;
use std::io::{self, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::Path;
use std::time::Duration;

const PORT: u16 = 8766;

fn response(request: &[u8], page: &[u8]) -> Vec<u8> {
    let text = std::str::from_utf8(request).unwrap_or("");
    let mut lines = text.split("\r\n");
    let first = lines.next().unwrap_or("");
    let hosts: Vec<_> = lines
        .filter_map(|line| {
            let (name, value) = line.split_once(':')?;
            name.eq_ignore_ascii_case("host")
                .then(|| value.trim().to_ascii_lowercase())
        })
        .collect();
    let allowed = hosts.len() == 1
        && [format!("127.0.0.1:{PORT}"), format!("localhost:{PORT}")].contains(&hosts[0]);
    let (status, body): (&str, &[u8]) = if !allowed {
        ("403 Forbidden", b"Localhost Host header required.")
    } else if matches!(first, "GET / HTTP/1.1" | "GET /index.html HTTP/1.1") {
        ("200 OK", page)
    } else {
        ("404 Not Found", b"Not found.")
    };
    let mut result = format!(
        "HTTP/1.1 {status}\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nCache-Control: no-store\r\nX-Content-Type-Options: nosniff\r\nContent-Security-Policy: base-uri 'none'; frame-ancestors 'none'\r\nReferrer-Policy: no-referrer\r\nConnection: close\r\n\r\n", body.len()
    ).into_bytes();
    result.extend_from_slice(body);
    result
}

fn serve(mut stream: TcpStream, page: &[u8]) -> io::Result<()> {
    stream.set_read_timeout(Some(Duration::from_secs(2)))?;
    stream.set_write_timeout(Some(Duration::from_secs(2)))?;
    let mut request = Vec::new();
    let mut buffer = [0; 1024];
    while request.len() < 8192 {
        let count = stream.read(&mut buffer)?;
        if count == 0 {
            return Ok(());
        }
        request.extend_from_slice(&buffer[..count]);
        if request.windows(4).any(|bytes| bytes == b"\r\n\r\n") {
            stream.write_all(&response(&request, page))?;
            return stream.flush();
        }
    }
    Err(io::Error::new(
        io::ErrorKind::InvalidInput,
        "Wallet page request headers exceeded 8 KiB",
    ))
}

pub fn run(path: &Path) -> io::Result<()> {
    let file = fs::File::open(path)?;
    let mut page = Vec::new();
    file.take(512 * 1024 + 1).read_to_end(&mut page)?;
    if page.len() > 512 * 1024 || !page.starts_with(b"<!doctype html>") {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "--wallet requires the public DiggerChain HTML handoff (at most 512 KiB)",
        ));
    }
    let listener = TcpListener::bind(("127.0.0.1", PORT))?;
    println!("DiggerChain wallet page: http://127.0.0.1:{PORT}");
    println!(
        "Local public page only. Signing stays in your browser wallet. Ctrl+C stops this page."
    );
    for stream in listener.incoming() {
        if let Err(error) = serve(stream?, &page)
            && !matches!(
                error.kind(),
                io::ErrorKind::BrokenPipe
                    | io::ErrorKind::ConnectionReset
                    | io::ErrorKind::TimedOut
                    | io::ErrorKind::WouldBlock
            )
        {
            eprintln!("Wallet page connection: {error}");
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn public_page_rejects_foreign_hosts_and_file_or_write_routes() {
        let page = b"<!doctype html><title>DiggerChain</title>";
        let valid = response(b"GET / HTTP/1.1\r\nHost: 127.0.0.1:8766\r\n\r\n", page);
        assert!(valid.starts_with(b"HTTP/1.1 200 OK"));
        assert!(valid.ends_with(page));
        for request in [
            &b"GET / HTTP/1.1\r\nHost: attacker.example:8766\r\n\r\n"[..],
            &b"GET /private-save.json HTTP/1.1\r\nHost: localhost:8766\r\n\r\n"[..],
            &b"POST / HTTP/1.1\r\nHost: localhost:8766\r\n\r\n"[..],
            &b"GET / HTTP/1.1\r\nHost: localhost:8766\r\nHost: attacker.example\r\n\r\n"[..],
        ] {
            assert!(!response(request, page).ends_with(page));
        }
    }
}
