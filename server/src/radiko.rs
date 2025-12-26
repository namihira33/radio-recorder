use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AuthToken {
    pub auth_token: String,
    pub area_id: String,
    pub is_premium: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Station {
    pub id: String,
    pub name: String,
    pub area_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Program {
    pub id: String,
    pub title: String,
    pub description: String,
    pub start_time: String,
    pub end_time: String,
    pub station_id: String,
    pub performer: Option<String>,
}

#[derive(Debug)]
pub struct RadikoError(pub String);

impl std::fmt::Display for RadikoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for RadikoError {}

pub async fn authenticate(email: &str, password: &str) -> Result<AuthToken, RadikoError> {
    let client = Client::new();

    // Step 1: Get auth1_fms token
    let auth1_response = client
        .get("https://radiko.jp/v2/api/auth1")
        .header("X-Radiko-App", "pc_html5")
        .header("X-Radiko-App-Version", "0.0.1")
        .header("X-Radiko-Device", "pc")
        .header("X-Radiko-User", "dummy_user")
        .send()
        .await
        .map_err(|e| RadikoError(format!("Auth1 request failed: {}", e)))?;

    let auth_token = auth1_response
        .headers()
        .get("X-Radiko-AuthToken")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .ok_or_else(|| RadikoError("Failed to get auth token".to_string()))?;

    let key_length: usize = auth1_response
        .headers()
        .get("X-Radiko-KeyLength")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse().ok())
        .unwrap_or(16);

    let key_offset: usize = auth1_response
        .headers()
        .get("X-Radiko-KeyOffset")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    // Step 2: Generate partial key
    let auth_key = "bcd151073c03b352e1ef2fd66c32209da9ca0afa";
    let partial_key = &auth_key[key_offset..key_offset + key_length];
    let partial_key_base64 = base64_encode(partial_key);

    // Step 3: Auth2 to get area
    let auth2_response = client
        .get("https://radiko.jp/v2/api/auth2")
        .header("X-Radiko-AuthToken", &auth_token)
        .header("X-Radiko-PartialKey", &partial_key_base64)
        .header("X-Radiko-Device", "pc")
        .header("X-Radiko-User", "dummy_user")
        .send()
        .await
        .map_err(|e| RadikoError(format!("Auth2 request failed: {}", e)))?;

    let area_info = auth2_response
        .text()
        .await
        .map_err(|e| RadikoError(format!("Failed to read auth2 response: {}", e)))?;

    let area_id = area_info
        .split(',')
        .next()
        .unwrap_or("JP13")
        .trim()
        .to_string();

    // Step 4: Premium login if credentials provided
    let is_premium = if !email.is_empty() && !password.is_empty() {
        login_premium(&client, email, password).await.unwrap_or(false)
    } else {
        false
    };

    Ok(AuthToken {
        auth_token,
        area_id,
        is_premium,
    })
}

async fn login_premium(client: &Client, email: &str, password: &str) -> Result<bool, RadikoError> {
    let params = [("mail", email), ("pass", password)];

    let response = client
        .post("https://radiko.jp/ap/member/login/login")
        .form(&params)
        .send()
        .await
        .map_err(|e| RadikoError(format!("Premium login failed: {}", e)))?;

    Ok(response.status().is_success())
}

fn base64_encode(input: &str) -> String {
    use std::io::Write;
    let mut buf = Vec::new();
    {
        let mut encoder = Base64Encoder::new(&mut buf);
        encoder.write_all(input.as_bytes()).unwrap();
        encoder.finish().unwrap();
    }
    String::from_utf8(buf).unwrap()
}

struct Base64Encoder<W: std::io::Write> {
    writer: W,
    buffer: [u8; 3],
    buffer_len: usize,
}

impl<W: std::io::Write> Base64Encoder<W> {
    fn new(writer: W) -> Self {
        Self {
            writer,
            buffer: [0; 3],
            buffer_len: 0,
        }
    }

    fn finish(mut self) -> std::io::Result<W> {
        if self.buffer_len > 0 {
            self.flush_buffer()?;
        }
        Ok(self.writer)
    }

    fn flush_buffer(&mut self) -> std::io::Result<()> {
        const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

        let b = &self.buffer[..self.buffer_len];
        let mut out = [b'='; 4];

        match self.buffer_len {
            1 => {
                out[0] = ALPHABET[(b[0] >> 2) as usize];
                out[1] = ALPHABET[((b[0] & 0x03) << 4) as usize];
            }
            2 => {
                out[0] = ALPHABET[(b[0] >> 2) as usize];
                out[1] = ALPHABET[(((b[0] & 0x03) << 4) | (b[1] >> 4)) as usize];
                out[2] = ALPHABET[((b[1] & 0x0f) << 2) as usize];
            }
            3 => {
                out[0] = ALPHABET[(b[0] >> 2) as usize];
                out[1] = ALPHABET[(((b[0] & 0x03) << 4) | (b[1] >> 4)) as usize];
                out[2] = ALPHABET[(((b[1] & 0x0f) << 2) | (b[2] >> 6)) as usize];
                out[3] = ALPHABET[(b[2] & 0x3f) as usize];
            }
            _ => {}
        }

        self.writer.write_all(&out)?;
        self.buffer_len = 0;
        Ok(())
    }
}

impl<W: std::io::Write> std::io::Write for Base64Encoder<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        for &byte in buf {
            self.buffer[self.buffer_len] = byte;
            self.buffer_len += 1;
            if self.buffer_len == 3 {
                self.flush_buffer()?;
            }
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

pub async fn get_stations(token: &AuthToken) -> Result<Vec<Station>, RadikoError> {
    let client = Client::new();

    let url = format!(
        "https://radiko.jp/v3/station/list/{}.xml",
        token.area_id
    );

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| RadikoError(format!("Failed to get stations: {}", e)))?;

    let xml = response
        .text()
        .await
        .map_err(|e| RadikoError(format!("Failed to read stations response: {}", e)))?;

    let mut stations = Vec::new();
    for line in xml.lines() {
        if line.contains("<id>") {
            let id = extract_tag_content(line, "id");
            if let Some(id) = id {
                stations.push(Station {
                    id: id.clone(),
                    name: id.clone(),
                    area_id: token.area_id.clone(),
                });
            }
        }
        if line.contains("<name>") {
            if let Some(name) = extract_tag_content(line, "name") {
                if let Some(station) = stations.last_mut() {
                    station.name = name;
                }
            }
        }
    }

    Ok(stations)
}

fn extract_tag_content(line: &str, tag: &str) -> Option<String> {
    let start_tag = format!("<{}>", tag);
    let end_tag = format!("</{}>", tag);

    if let Some(start) = line.find(&start_tag) {
        if let Some(end) = line.find(&end_tag) {
            let content_start = start + start_tag.len();
            if content_start < end {
                return Some(line[content_start..end].to_string());
            }
        }
    }
    None
}

pub async fn get_programs(
    token: &AuthToken,
    station_id: &str,
    date: &str,
) -> Result<Vec<Program>, RadikoError> {
    let client = Client::new();

    let url = format!(
        "https://radiko.jp/v3/program/station/date/{}/{}.xml",
        date, station_id
    );

    let response = client
        .get(&url)
        .header("X-Radiko-AuthToken", &token.auth_token)
        .send()
        .await
        .map_err(|e| RadikoError(format!("Failed to get programs: {}", e)))?;

    let xml = response
        .text()
        .await
        .map_err(|e| RadikoError(format!("Failed to read programs response: {}", e)))?;

    let mut programs = Vec::new();
    let mut current_program: Option<Program> = None;

    for line in xml.lines() {
        let line = line.trim();

        if line.starts_with("<prog ") {
            let id = extract_attr(line, "id").unwrap_or_default();
            let ft = extract_attr(line, "ft").unwrap_or_default();
            let to = extract_attr(line, "to").unwrap_or_default();

            current_program = Some(Program {
                id,
                title: String::new(),
                description: String::new(),
                start_time: ft,
                end_time: to,
                station_id: station_id.to_string(),
                performer: None,
            });
        }

        if let Some(ref mut prog) = current_program {
            if let Some(title) = extract_tag_content(line, "title") {
                prog.title = title;
            }
            if let Some(desc) = extract_tag_content(line, "desc") {
                prog.description = desc;
            }
            if let Some(pfm) = extract_tag_content(line, "pfm") {
                prog.performer = Some(pfm);
            }
        }

        if line.starts_with("</prog>") {
            if let Some(prog) = current_program.take() {
                if !prog.title.is_empty() {
                    programs.push(prog);
                }
            }
        }
    }

    Ok(programs)
}

fn extract_attr(line: &str, attr: &str) -> Option<String> {
    let pattern = format!("{}=\"", attr);
    if let Some(start) = line.find(&pattern) {
        let value_start = start + pattern.len();
        if let Some(end) = line[value_start..].find('"') {
            return Some(line[value_start..value_start + end].to_string());
        }
    }
    None
}

pub fn get_stream_url(token: &AuthToken, station_id: &str) -> String {
    format!(
        "https://radiko.jp/v2/api/ts/playlist.m3u8?station_id={}&l=15&lsid={}&type=b",
        station_id, token.auth_token
    )
}
