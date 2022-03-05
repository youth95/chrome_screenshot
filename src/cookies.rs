use std::fmt::{Display, Write};

use crypto::{
    aes,
    blockmodes::PkcsPadding,
    buffer::{BufferResult, ReadBuffer, RefReadBuffer, RefWriteBuffer, WriteBuffer},
};
use headless_chrome::protocol::cdp::Network::{self, CookieParam};
use pbkdf2::{
    password_hash::{Ident, PasswordHasher, SaltString},
    Pbkdf2,
};

use rusqlite::Connection;

pub fn get_chrome_derived_key() -> String {
    const SERVICE: &str = "Chrome Safe Storage";
    const ACCOUNT: &str = "Chrome";
    let password =
        keytar::get_password(SERVICE, ACCOUNT).expect("would not get chrome derived key");
    return password.password;
}

pub struct CookieRecord {
    name: String,
    encrypted_value: Vec<u8>,
}

fn decrypt(encrypted_data: &[u8], key: &[u8], iv: &[u8]) -> Vec<u8> {
    let mut decryptor = aes::cbc_decryptor(aes::KeySize::KeySize128, key, iv, PkcsPadding);

    let mut final_result = Vec::<u8>::new();
    let mut read_buffer = RefReadBuffer::new(encrypted_data);
    let mut buffer = [0; 4096];
    let mut write_buffer = RefWriteBuffer::new(&mut buffer);

    loop {
        let result = decryptor
            .decrypt(&mut read_buffer, &mut write_buffer, true)
            .unwrap();
        final_result.extend(
            write_buffer
                .take_read_buffer()
                .take_remaining()
                .iter()
                .map(|&i| i),
        );
        match result {
            BufferResult::BufferUnderflow => break,
            BufferResult::BufferOverflow => {}
        }
    }

    final_result
}

pub struct Cookies(Vec<Network::CookieParam>);

pub fn make_cookies(host_key: &str) -> Cookies {
    let password = get_chrome_derived_key();
    let salt = SaltString::new(base64::encode(b"saltysalt").as_str()).unwrap();
    let password_hash = Pbkdf2
        .hash_password_customized(
            password.as_bytes(),
            Some(Ident::new("pbkdf2")),
            None,
            pbkdf2::Params {
                rounds: 1003,
                output_length: 16,
            },
            &salt,
        )
        .unwrap();
    let key = password_hash.hash.unwrap();
    let iv = [0x20u8; 16];

    let cookies_store_path = dirs::home_dir()
        .expect("Would not get home dir")
        .join("Library/ApplicationSupport/Google/Chrome/Default/Cookies");
    let connection =
        Connection::open(cookies_store_path).expect("Would not get database connection");
    let mut stmt = connection
        .prepare("select name,encrypted_value from cookies where host_key = :host_key")
        // .prepare("select name,encrypted_value from cookies")
        .unwrap();
    let cookies_record_iter = stmt
        .query_map(&[(":host_key", host_key)], |row| {
            Ok(CookieRecord {
                name: row.get(0).unwrap(),
                encrypted_value: row.get(1).unwrap(),
            })
        })
        .unwrap();
    let mut result = Vec::<Network::CookieParam>::new();
    for record in cookies_record_iter {
        if let Ok(CookieRecord {
            name,
            encrypted_value,
        }) = record
        {
            if encrypted_value.len() > 0 {
                let data = decrypt(&encrypted_value[3..], key.as_bytes(), &iv);
                let value = String::from_utf8(data).unwrap();
                result.push(make_cookie(name, value, host_key.to_string()));
            }
        }
    }
    Cookies(result)
}

impl Into<Vec<Network::CookieParam>> for Cookies {
    fn into(self) -> Vec<Network::CookieParam> {
        self.0
    }
}

impl Display for Cookies {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let cookies_len = self.0.len();
        for (i, cookie) in self.0.iter().enumerate() {
            f.write_fmt(format_args!("{}={}", cookie.name, cookie.value))
                .unwrap();
            if i < cookies_len - 1 {
                f.write_char(',').unwrap();
            }
        }
        Ok(())
    }
}

fn make_cookie(name: String, value: String, domain: String) -> CookieParam {
    CookieParam {
        name,
        value,
        url: Default::default(),
        domain: Some(domain),
        path: Default::default(),
        secure: Default::default(),
        http_only: Default::default(),
        same_site: Default::default(),
        expires: Default::default(),
        priority: Default::default(),
        same_party: Default::default(),
        source_scheme: Default::default(),
        source_port: Default::default(),
        partition_key: Default::default(),
    }
}
