use std::net::IpAddr;

use serde::{Deserialize, Serialize};
use url::Url;

use crate::models::DownloadError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UrlPolicyConfig {
    pub allow_http: bool,
    pub allow_https: bool,
    pub allow_local_network_for_testing: bool,
    pub max_redirects: usize,
    pub max_download_size_bytes: u64,
}

impl Default for UrlPolicyConfig {
    fn default() -> Self {
        Self {
            allow_http: true,
            allow_https: true,
            allow_local_network_for_testing: false,
            max_redirects: 5,
            max_download_size_bytes: 2 * 1024 * 1024 * 1024,
        }
    }
}

#[derive(Debug, Clone)]
pub struct UrlPolicy {
    config: UrlPolicyConfig,
}

impl UrlPolicy {
    pub fn new(config: UrlPolicyConfig) -> Self {
        Self { config }
    }

    pub fn config(&self) -> &UrlPolicyConfig {
        &self.config
    }

    pub fn validate(&self, candidate: &str) -> Result<Url, DownloadError> {
        let parsed = Url::parse(candidate).map_err(|e| DownloadError::InvalidUrl {
            reason: e.to_string(),
        })?;

        if parsed.username() != "" || parsed.password().is_some() {
            return Err(DownloadError::PolicyViolation {
                reason: "URL credentials are not allowed".to_string(),
            });
        }

        let scheme = parsed.scheme();
        if (scheme == "http" && !self.config.allow_http)
            || (scheme == "https" && !self.config.allow_https)
            || (scheme != "http" && scheme != "https")
        {
            return Err(DownloadError::PolicyViolation {
                reason: format!("unsupported or blocked scheme: {scheme}"),
            });
        }

        let host = parsed
            .host_str()
            .ok_or_else(|| DownloadError::PolicyViolation {
                reason: "URL host is missing".to_string(),
            })?;

        if host.eq_ignore_ascii_case("localhost") || host.ends_with(".localhost") {
            return Err(DownloadError::PolicyViolation {
                reason: "localhost targets are not allowed".to_string(),
            });
        }

        if let Ok(ip) = host.parse::<IpAddr>() {
            validate_ip(ip, self.config.allow_local_network_for_testing)?;
        }

        Ok(parsed)
    }

    // TODO(security): Add DNS resolution + anti-rebinding checks per redirect hop.
    // TODO(security): Validate final resolved endpoint across proxy/VPN/network changes.
}

fn validate_ip(ip: IpAddr, allow_local_network_for_testing: bool) -> Result<(), DownloadError> {
    let blocked = match ip {
        IpAddr::V4(v4) => {
            v4.is_loopback()
                || v4.is_private()
                || v4.is_link_local()
                || v4.is_unspecified()
                || v4.is_multicast()
                || v4.is_documentation()
                || v4.is_broadcast()
        }
        IpAddr::V6(v6) => {
            v6.is_loopback()
                || v6.is_unspecified()
                || v6.is_multicast()
                || v6.is_unicast_link_local()
                || v6.is_unique_local()
        }
    };

    if blocked && !allow_local_network_for_testing {
        Err(DownloadError::PolicyViolation {
            reason: format!("IP target is blocked by policy: {ip}"),
        })
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_public_https() {
        let policy = UrlPolicy::new(UrlPolicyConfig::default());
        let url = policy
            .validate("https://example.com/file.bin")
            .expect("should validate");
        assert_eq!(url.scheme(), "https");
    }

    #[test]
    fn rejects_localhost_and_private_targets() {
        let policy = UrlPolicy::new(UrlPolicyConfig::default());

        assert!(policy.validate("http://localhost:8080/a").is_err());
        assert!(policy.validate("http://127.0.0.1/a").is_err());
        assert!(policy.validate("http://192.168.1.7/a").is_err());
    }

    #[test]
    fn rejects_non_http_schemes() {
        let policy = UrlPolicy::new(UrlPolicyConfig::default());

        assert!(policy.validate("file:///etc/passwd").is_err());
        assert!(policy.validate("data:text/plain,hello").is_err());
        assert!(policy.validate("ftp://example.com/file").is_err());
    }
}
