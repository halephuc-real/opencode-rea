use anyhow::{anyhow, Result};
use base64::Engine;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use rand::RngCore;
use sha2::{Digest, Sha256};
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::time::Duration;
use tiny_http::{Response, Server};

pub fn signin() -> Result<String> {

    let mut verifier = vec![0u8; 64];
    rand::thread_rng().fill_bytes(&mut verifier);
    let verifier = URL_SAFE_NO_PAD.encode(&verifier);
    let mut state = vec![0u8; 32];
    rand::thread_rng().fill_bytes(&mut state);
    let state = URL_SAFE_NO_PAD.encode(&state);

    let challenge = {
        let h = Sha256::digest(verifier.as_bytes());
        URL_SAFE_NO_PAD.encode(h)
    };

    let preferred = "127.0.0.1:47913";
    let listener = TcpListener::bind(preferred).or_else(|_| TcpListener::bind("127.0.0.1:0"))?;
    let port = listener.local_addr()?.port();

    let server = Server::from_listener(listener, None).map_err(|e| anyhow!(e.to_string()))?;

    let callback_url = format!("http://127.0.0.1:{port}/callback");
    let url = format!(
        "https://nano-gpt.com/auth?callback_url={}&code_challenge={}&code_challenge_method=S256&scope=api.use%20models.read&state={}&client_name=opencode-rea",
        urlencoding::encode(&callback_url),
        urlencoding::encode(&challenge),
        urlencoding::encode(&state)
    );
    eprintln!("Dang mo trinh duyet de dang nhap NanoGPT. Sau khi dong y, quay lai cua so nay.");
    eprintln!("Neu khong tu mo, mo link: {url}");
    let _ = open::that(&url);

    let deadline = std::time::Instant::now() + Duration::from_secs(300);
    let mut code: Option<String> = None;
    while std::time::Instant::now() < deadline {

        match server.recv_timeout(Duration::from_secs(1)) {
            Ok(Some(req)) => {

                let url = req.url().to_string();
                let method_ok = req.method() == &tiny_http::Method::Get;

                let mut received_state = None;
                let mut received_code = None;
                let mut received_error = None;
                if let Some(q) = url.split('?').nth(1) {
                    for pair in q.split('&') {
                        let mut kv = pair.splitn(2, '=');
                        let k = kv.next().unwrap_or("");
                        let v = kv.next().unwrap_or("");
                        let v = urlencoding::decode(v).unwrap_or_default();
                        match k {
                            "state" => received_state = Some(v.to_string()),
                            "code" => received_code = Some(v.to_string()),
                            "error" => received_error = Some(v.to_string()),
                            _ => {}
                        }
                    }
                }

                if !method_ok || received_state.as_deref() != Some(state.as_str()) {
                    let _ = req.respond(Response::from_string("bad request").with_status_code(400));
                    continue;
                }
                if let Some(err) = received_error {
                    let _ = req.respond(Response::from_string("error").with_status_code(400));
                    return Err(anyhow!("sign-in error: {err}"));
                }
                if let Some(c) = received_code {
                    let _ = req.respond(Response::from_string("ok").with_status_code(200));
                    code = Some(c);
                    break;
                }
                let _ = req.respond(Response::from_string("bad request").with_status_code(400));
            }
            Ok(None) => {}
            Err(_) => break,
        }
    }

    let code = code.ok_or_else(|| anyhow!("no callback received within 5 minutes"))?;

    let client = reqwest::blocking::Client::new();
    let resp = client
        .post("https://nano-gpt.com/api/v1/auth/keys")
        .json(&serde_json::json!({
            "grant_type": "authorization_code",
            "code": code,
            "code_verifier": verifier
        }))
        .send()?;
    if !resp.status().is_success() {
        return Err(anyhow!("code exchange failed: {}", resp.status()));
    }
    let v: serde_json::Value = resp.json()?;
    let key = v.get("key").and_then(|x| x.as_str()).ok_or_else(|| anyhow!("no key in response"))?;
    Ok(key.to_string())
}
