use std::{
    collections::{HashMap, HashSet},
    net::IpAddr,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use axum::http::HeaderMap;

const DEFAULT_REQUESTS_PER_MINUTE: u32 = 60;
const DEFAULT_BURST_SIZE: u32 = 10;
const MAX_TRACKED_IPS: usize = 10_000;
const ENTRY_RETENTION: Duration = Duration::from_secs(5 * 60);
const LOOPBACK_PROXY_V4: &str = "127.0.0.1";
const LOOPBACK_PROXY_V6: &str = "::1";

#[derive(Clone)]
pub struct IpRateLimiter {
    state: Arc<Mutex<HashMap<IpAddr, Bucket>>>,
    refill_per_second: f64,
    burst_size: f64,
    trusted_proxies: Arc<HashSet<IpAddr>>,
}

#[derive(Debug, Clone, Copy)]
struct Bucket {
    tokens: f64,
    last_refill: Instant,
}

impl IpRateLimiter {
    pub fn from_env() -> Self {
        let requests_per_minute =
            parse_env_u32("PDFIN_RATE_LIMIT_PER_MINUTE", DEFAULT_REQUESTS_PER_MINUTE).max(1);
        let burst_size = parse_env_u32("PDFIN_RATE_LIMIT_BURST", DEFAULT_BURST_SIZE)
            .max(1)
            .min(requests_per_minute);

        Self {
            state: Arc::new(Mutex::new(HashMap::new())),
            refill_per_second: requests_per_minute as f64 / 60.0,
            burst_size: burst_size as f64,
            trusted_proxies: Arc::new(parse_trusted_proxies()),
        }
    }

    pub fn client_ip(&self, peer_ip: IpAddr, headers: &HeaderMap) -> IpAddr {
        if !self.trusted_proxies.contains(&peer_ip) {
            return peer_ip;
        }

        if let Some(cloudflare_ip) = parse_header_ip(headers, "cf-connecting-ip") {
            return cloudflare_ip;
        }

        let Some(forwarded_for) = headers.get("x-forwarded-for") else {
            return peer_ip;
        };

        let Ok(value) = forwarded_for.to_str() else {
            return peer_ip;
        };

        value
            .split(',')
            .rev()
            .filter_map(|entry| entry.trim().parse::<IpAddr>().ok())
            .find(|ip| !self.trusted_proxies.contains(ip))
            .unwrap_or(peer_ip)
    }

    pub fn allow(&self, ip: IpAddr) -> bool {
        let now = Instant::now();
        let Ok(mut state) = self.state.lock() else {
            // Fail closed if the limiter state is poisoned. Request concurrency,
            // body limits, and timeouts remain available as independent guards.
            return false;
        };

        if state.len() >= MAX_TRACKED_IPS && !state.contains_key(&ip) {
            state.retain(|_, bucket| now.duration_since(bucket.last_refill) < ENTRY_RETENTION);

            if state.len() >= MAX_TRACKED_IPS {
                return false;
            }
        }

        let bucket = state.entry(ip).or_insert(Bucket {
            tokens: self.burst_size,
            last_refill: now,
        });

        let elapsed = now.duration_since(bucket.last_refill).as_secs_f64();
        bucket.tokens = (bucket.tokens + elapsed * self.refill_per_second).min(self.burst_size);
        bucket.last_refill = now;

        if bucket.tokens < 1.0 {
            return false;
        }

        bucket.tokens -= 1.0;
        true
    }
}

fn parse_header_ip(headers: &HeaderMap, name: &'static str) -> Option<IpAddr> {
    headers.get(name)?.to_str().ok()?.trim().parse().ok()
}

fn parse_trusted_proxies() -> HashSet<IpAddr> {
    let mut proxies = HashSet::from([
        LOOPBACK_PROXY_V4.parse::<IpAddr>().expect("valid IPv4"),
        LOOPBACK_PROXY_V6.parse::<IpAddr>().expect("valid IPv6"),
    ]);

    let configured = std::env::var("PDFIN_TRUSTED_PROXY_IPS").unwrap_or_default();

    for value in configured.split(',').map(str::trim).filter(|value| !value.is_empty()) {
        match value.parse::<IpAddr>() {
            Ok(ip) => {
                proxies.insert(ip);
            }
            Err(_) => {
                tracing::warn!(proxy = %value, "Alamat trusted proxy tidak valid; diabaikan");
            }
        }
    }

    proxies
}

fn parse_env_u32(name: &str, default: u32) -> u32 {
    match std::env::var(name) {
        Ok(value) => value.parse::<u32>().unwrap_or_else(|_| {
            tracing::warn!(
                variable = name,
                "Nilai rate-limit environment tidak valid; memakai default"
            );
            default
        }),
        Err(_) => default,
    }
}

impl Default for IpRateLimiter {
    fn default() -> Self {
        Self::from_env()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn limiter_with(proxies: &[IpAddr]) -> IpRateLimiter {
        IpRateLimiter {
            state: Arc::new(Mutex::new(HashMap::new())),
            refill_per_second: 0.0,
            burst_size: 2.0,
            trusted_proxies: Arc::new(proxies.iter().copied().collect()),
        }
    }

    #[test]
    fn allows_burst_then_throttles() {
        let limiter = limiter_with(&[]);
        let ip = "127.0.0.1".parse().unwrap();

        assert!(limiter.allow(ip));
        assert!(limiter.allow(ip));
        assert!(!limiter.allow(ip));
    }

    #[test]
    fn different_ips_have_independent_buckets() {
        let limiter = limiter_with(&[]);
        let first = "127.0.0.1".parse().unwrap();
        let second = "127.0.0.2".parse().unwrap();

        assert!(limiter.allow(first));
        assert!(limiter.allow(first));
        assert!(!limiter.allow(first));
        assert!(limiter.allow(second));
    }

    #[test]
    fn does_not_trust_forwarded_for_from_untrusted_peer() {
        let limiter = limiter_with(&[]);
        let peer = "198.51.100.10".parse().unwrap();
        let client: IpAddr = "203.0.113.20".parse().unwrap();
        let mut headers = HeaderMap::new();
        headers.insert("x-forwarded-for", client.to_string().parse().unwrap());

        assert_eq!(limiter.client_ip(peer, &headers), peer);
    }

    #[test]
    fn resolves_forwarded_client_through_trusted_proxy() {
        let proxy = "127.0.0.1".parse().unwrap();
        let client: IpAddr = "203.0.113.20".parse().unwrap();
        let limiter = limiter_with(&[proxy]);
        let mut headers = HeaderMap::new();
        headers.insert("x-forwarded-for", client.to_string().parse().unwrap());

        assert_eq!(limiter.client_ip(proxy, &headers), client);
    }

    #[test]
    fn prefers_cloudflare_client_ip_through_trusted_proxy() {
        let proxy = "127.0.0.1".parse().unwrap();
        let client: IpAddr = "203.0.113.20".parse().unwrap();
        let mut headers = HeaderMap::new();
        headers.insert("cf-connecting-ip", client.to_string().parse().unwrap());

        let limiter = limiter_with(&[proxy]);
        assert_eq!(limiter.client_ip(proxy, &headers), client);
    }
}
