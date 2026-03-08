use std::time::{SystemTime, UNIX_EPOCH};
use totp_rs::{Algorithm, Secret, TOTP};

pub struct TotpParams {
    pub secret: String,
    pub algorithm: String,
    pub digits: u32,
    pub period: u32,
    #[allow(dead_code)]
    pub issuer: Option<String>,
    #[allow(dead_code)]
    pub account: Option<String>,
}

pub struct TotpCode {
    pub code: String,
    pub remaining: u64,
    pub period: u64,
}

/// Generate a random Base32-encoded TOTP secret.
pub fn generate_secret() -> String {
    let secret = Secret::generate_secret();
    secret.to_encoded().to_string()
}

/// Generate the current TOTP code from a Base32 secret.
pub fn generate_code(
    secret_b32: &str,
    algorithm: &str,
    digits: u32,
    period: u32,
) -> Result<TotpCode, String> {
    let totp = build_totp(secret_b32, algorithm, digits, period, None, None)?;

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_secs();

    let code = totp.generate(now);
    let remaining = period as u64 - (now % period as u64);

    Ok(TotpCode {
        code,
        remaining,
        period: period as u64,
    })
}

/// Build an otpauth:// URI.
pub fn generate_uri(
    secret_b32: &str,
    account: &str,
    issuer: &str,
    algorithm: &str,
    digits: u32,
    period: u32,
) -> Result<String, String> {
    let totp = build_totp(
        secret_b32,
        algorithm,
        digits,
        period,
        Some(issuer.to_string()),
        Some(account.to_string()),
    )?;
    Ok(totp.get_url())
}

/// Generate a QR code as an SVG string for the given otpauth:// URI.
pub fn generate_qr_svg(uri: &str) -> Result<String, String> {
    use qrcode::render::svg;
    use qrcode::QrCode;

    let code = QrCode::new(uri.as_bytes()).map_err(|e| e.to_string())?;
    let svg = code
        .render::<svg::Color>()
        .min_dimensions(200, 200)
        .dark_color(svg::Color("#000000"))
        .light_color(svg::Color("#ffffff"))
        .build();
    Ok(svg)
}

/// Parse an otpauth:// URI and extract TOTP parameters.
pub fn parse_otpauth_uri(uri: &str) -> Result<TotpParams, String> {
    let totp = TOTP::from_url(uri).map_err(|e| format!("Invalid otpauth URI: {}", e))?;

    let algorithm = match totp.algorithm {
        Algorithm::SHA1 => "SHA1",
        Algorithm::SHA256 => "SHA256",
        Algorithm::SHA512 => "SHA512",
    }
    .to_string();

    let secret_b32 = Secret::Raw(totp.secret.clone())
        .to_encoded()
        .to_string();

    Ok(TotpParams {
        secret: secret_b32,
        algorithm,
        digits: totp.digits as u32,
        period: totp.step as u32,
        issuer: totp.issuer.clone(),
        account: Some(totp.account_name.clone()),
    })
}

/// Seconds remaining until the current code expires.
#[allow(dead_code)]
pub fn time_remaining(period: u32) -> u64 {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    period as u64 - (now % period as u64)
}

fn build_totp(
    secret_b32: &str,
    algorithm: &str,
    digits: u32,
    period: u32,
    issuer: Option<String>,
    account: Option<String>,
) -> Result<TOTP, String> {
    let algo = match algorithm.to_uppercase().as_str() {
        "SHA1" => Algorithm::SHA1,
        "SHA256" => Algorithm::SHA256,
        "SHA512" => Algorithm::SHA512,
        _ => return Err(format!("Unsupported algorithm: {}", algorithm)),
    };

    let secret_bytes = Secret::Encoded(secret_b32.to_string())
        .to_bytes()
        .map_err(|e| format!("Invalid Base32 secret: {}", e))?;

    TOTP::new(
        algo,
        digits as usize,
        1, // skew (allow 1 step drift)
        period as u64,
        secret_bytes,
        issuer,
        account.unwrap_or_default(),
    )
    .map_err(|e| format!("Failed to create TOTP: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_secret() {
        let secret = generate_secret();
        assert!(!secret.is_empty());
        assert!(secret.len() >= 16);
    }

    #[test]
    fn test_generate_code() {
        let secret = generate_secret();
        let result = generate_code(&secret, "SHA1", 6, 30);
        assert!(result.is_ok());
        let code = result.unwrap();
        assert_eq!(code.code.len(), 6);
        assert!(code.remaining > 0 && code.remaining <= 30);
    }

    #[test]
    fn test_generate_uri() {
        let secret = generate_secret();
        let uri = generate_uri(&secret, "user@example.com", "TestApp", "SHA1", 6, 30);
        assert!(uri.is_ok());
        let uri = uri.unwrap();
        assert!(uri.starts_with("otpauth://totp/"));
    }

    #[test]
    fn test_parse_otpauth_uri() {
        let secret = generate_secret();
        let uri =
            generate_uri(&secret, "user@example.com", "TestApp", "SHA1", 6, 30).unwrap();
        let params = parse_otpauth_uri(&uri);
        assert!(params.is_ok());
        let params = params.unwrap();
        assert_eq!(params.algorithm, "SHA1");
        assert_eq!(params.digits, 6);
        assert_eq!(params.period, 30);
    }

    #[test]
    fn test_qr_svg() {
        let svg = generate_qr_svg("otpauth://totp/Test?secret=JBSWY3DPEHPK3PXP");
        assert!(svg.is_ok());
        let svg = svg.unwrap();
        assert!(svg.contains("<svg"));
    }
}
