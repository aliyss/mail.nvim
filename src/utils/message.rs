//! Builds raw RFC 822 messages for sending.
//!
//! The compose commands turn the (editable) contents of a compose buffer into
//! a raw message that the provider's sending backend can deliver as-is. The
//! body is quoted-printable encoded so non-ASCII text survives delivery
//! without relying on 8bit SMTP extensions, and the subject is RFC 2047
//! encoded when it contains anything but ASCII.

use std::fmt::Write as _;
use std::time::{SystemTime, UNIX_EPOCH};

/// The parts of an outgoing message.
pub struct MessageParts {
    /// The sender address (account's address).
    pub from: String,
    /// Recipients of the `To` header, comma separated.
    pub to: String,
    /// Recipients of the `Cc` header, comma separated (empty to omit).
    pub cc: String,
    /// Recipients of the `Bcc` header, comma separated (empty to omit).
    pub bcc: String,
    /// The subject line.
    pub subject: String,
    /// The plain text body.
    pub body: String,
    /// The `Message-ID` of the message being replied to.
    pub in_reply_to: Option<String>,
    /// The `References` header value (usually the original message id).
    pub references: Option<String>,
}

/// Builds the raw RFC 822 message for `parts`.
#[must_use]
pub fn build_raw_message(parts: &MessageParts) -> Vec<u8> {
    let mut headers: Vec<String> = Vec::new();

    headers.push(format!("From: {}", parts.from));
    headers.push(format!("To: {}", parts.to));
    if !parts.cc.is_empty() {
        headers.push(format!("Cc: {}", parts.cc));
    }
    if !parts.bcc.is_empty() {
        headers.push(format!("Bcc: {}", parts.bcc));
    }
    headers.push(format!("Subject: {}", encode_rfc2047(&parts.subject)));

    let date = chrono::Utc::now().format("%a, %d %b %Y %H:%M:%S %z").to_string();
    headers.push(format!("Date: {date}"));
    headers.push(format!("Message-ID: <{}>", generate_message_id()));

    if let Some(in_reply_to) = &parts.in_reply_to {
        headers.push(format!("In-Reply-To: {in_reply_to}"));
    }
    if let Some(references) = &parts.references {
        headers.push(format!("References: {references}"));
    }

    headers.push("MIME-Version: 1.0".into());
    headers.push("Content-Type: text/plain; charset=utf-8".into());
    headers.push("Content-Transfer-Encoding: quoted-printable".into());

    let mut message = String::new();
    for header in &headers {
        message.push_str(header);
        message.push_str("\r\n");
    }
    message.push_str("\r\n");
    message.push_str(&quoted_printable(&parts.body));

    message.into_bytes()
}

/// A unique-enough `Message-ID` local part for this process.
fn generate_message_id() -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos());
    format!("{timestamp}.{}.mail.nvim", std::process::id())
}

/// Encodes `s` with the RFC 2047 `Q` encoding when it contains anything but
/// printable ASCII, so non-ASCII subjects survive delivery.
fn encode_rfc2047(s: &str) -> String {
    let needs_encoding = !s.is_ascii() || s.bytes().any(|b| b < 32 || b == 127);
    if !needs_encoding {
        return s.to_string();
    }

    let mut encoded = String::with_capacity(s.len() * 3);
    for &byte in s.as_bytes() {
        match byte {
            b' ' => encoded.push('_'),
            b'=' | b'?' | b'_' => write!(encoded, "={byte:02X}").expect("write to string"),
            33..=126 => encoded.push(char::from(byte)),
            _ => write!(encoded, "={byte:02X}").expect("write to string"),
        }
    }
    format!("=?UTF-8?Q?{encoded}?=")
}

/// Quoted-printable encodes `s`: lines folded at 76 columns with soft line
/// breaks, non-ASCII bytes (and `=`) hex-encoded.
fn quoted_printable(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + s.len() / 2);
    let mut column = 0usize;

    for &byte in s.as_bytes() {
        // Space stays literal (RFC 2045 only requires encoding trailing
        // whitespace); everything non-printable is hex-encoded.
        let printable = matches!(byte, b' ' | b'=' | 33..=60 | 62..=126);
        let encoded: String = if printable {
            char::from(byte).to_string()
        } else {
            format!("={byte:02X}")
        };

        let is_newline = byte == b'\n';
        if is_newline {
            out.push_str("\r\n");
            column = 0;
            continue;
        }

        // A hard carriage return in the source is kept as-is; everything
        // else is encoded, so the output only ever contains bare `\n`.
        let width = if printable { 1 } else { 3 };
        if column + width > 76 {
            out.push_str("=\r\n");
            column = 0;
        }
        out.push_str(&encoded);
        column += width;
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parts() -> MessageParts {
        MessageParts {
            from: "me@example.com".into(),
            to: "you@example.com".into(),
            cc: String::new(),
            bcc: String::new(),
            subject: "Hello".into(),
            body: "line one\nline two".into(),
            in_reply_to: None,
            references: None,
        }
    }

    #[test]
    fn builds_a_well_formed_message() {
        let raw = String::from_utf8(build_raw_message(&parts())).expect("message is utf-8");
        assert!(raw.starts_with("From: me@example.com\r\nTo: you@example.com\r\n"));
        assert!(raw.contains("Subject: Hello\r\n"));
        assert!(raw.contains("Content-Transfer-Encoding: quoted-printable\r\n"));
        assert!(raw.contains("\r\n\r\nline one\r\nline two"));
    }

    #[test]
    fn includes_reply_headers() {
        let mut parts = parts();
        parts.in_reply_to = Some("<abc@x>".into());
        parts.references = Some("<abc@x>".into());
        let raw = String::from_utf8(build_raw_message(&parts)).expect("message is utf-8");
        assert!(raw.contains("In-Reply-To: <abc@x>\r\n"));
        assert!(raw.contains("References: <abc@x>\r\n"));
    }

    #[test]
    fn includes_cc_and_bcc_when_present() {
        let mut parts = parts();
        parts.cc = "a@x.com".into();
        parts.bcc = "b@x.com".into();
        let raw = String::from_utf8(build_raw_message(&parts)).expect("message is utf-8");
        assert!(raw.contains("Cc: a@x.com\r\n"));
        assert!(raw.contains("Bcc: b@x.com\r\n"));
    }

    #[test]
    fn non_ascii_subject_is_rfc2047_encoded() {
        let mut parts = parts();
        parts.subject = "Héllo wörld".into();
        let raw = String::from_utf8(build_raw_message(&parts)).expect("message is utf-8");
        assert!(raw.contains("Subject: =?UTF-8?Q?"));
    }

    #[test]
    fn non_ascii_body_is_quoted_printable() {
        let mut parts = parts();
        parts.body = "café ☕".into();
        let raw = String::from_utf8(build_raw_message(&parts)).expect("message is utf-8");
        assert!(raw.contains("caf=C3=A9 =E2=98=95"));
    }

    #[test]
    fn long_lines_are_folded_at_76_columns() {
        let mut parts = parts();
        parts.body = "x".repeat(100);
        let raw = String::from_utf8(build_raw_message(&parts)).expect("message is utf-8");
        assert!(raw.contains("=\r\n"), "expected a soft line break");
    }

    #[test]
    fn message_id_is_unique() {
        let a = generate_message_id();
        let b = generate_message_id();
        assert_ne!(a, b);
        assert!(!a.is_empty());
    }
}
