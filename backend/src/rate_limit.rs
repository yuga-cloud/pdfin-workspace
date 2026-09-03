use std::{
    collections::HashMap,
    net::IpAddr,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

const DEFAULT_REQUESTS_PER_MINUTE: u32 = 60;
const DEFAULT_BURST_SIZE: u32 = 10;
const MAX_TRACKED_IPS: usize = 10_000;
const ENTRY_RETENTION: Duration = Duration::from_secs(5 * 60);

#[derive(Clone)]
pub struct IpRateLimiter {
    state: Arc<Mutex<HashMap<IpAddr, Bucket>>>,
    refill_per_second: f64,
    burst_size: f64,
}

#[derive(Debug, Clone, Copy)]
struct Bucket {
    tokens: f64,
    last_refill: Instant,
}

impl IpRateLimiter {
    pub fn from_env() -> Self {
        let requests_per_minute = parse_env_u32(
            "PDFIN_RATE_LIMIT_PER_MINUTE",
            DEFAULT_REQUESTS_PER_MINUTE,
        )
        .max(1);
        let burst_size = parse_env_u32("PDFIN_RATE_LIMIT_BURST", DEFAULT_BURST_SIZE)
            .max(1)
            .min(requests_per_minute);

        Self {
            state: Arc::new(Mutex::new(HashMap::new())),
            refill_per_second: requests_per_minute as f64 / 60.0,
            burst_size: burst_size as f64,
        }
    }

    pub fn allow(&self, ip: IpAddr) -> bool {
        let now = Instant::now();
        let Ok(mut state) = self.state.lock() else {
            // Fail open if the limiter's internal mutex is poisoned. The request
            // concurrency and body limits still protect the backend from overload.
            return true;
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

fn parse_env_u32(name: &str, default: u32) -> u32 {
    match std::env::var(name) {
        Ok(value) => value.parse::<u32>().unwrap_or_else(|_| {
            tracing::warn!(variable = name, "Nilai rate-limit environment tidak valid; memakai default");
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

    #[test]
    fn allows_burst_then_throttles() {
        let limiter = IpRateLimiter {
            state: Arc::new(Mutex::new(HashMap::new())),
            refill_per_second: 0.0,
            burst_size: 2.0,
        };
        let ip = "127.0.0.1".parse().unwrap();

        assert!(limiter.allow(ip));
        assert!(limiter.allow(ip));
        assert!(!limiter.allow(ip));
    }

    #[test]
    fn different_ips_have_independent_buckets() {
        let limiter = IpRateLimiter {
            state: Arc::new(Mutex::new(HashMap::new())),
            refill_per_second: 0.0,
            burst_size: 1.0,
        };
        let first = "127.0.0.1".parse().unwrap();
        let second = "127.0.0.2".parse().unwrap();

        assert!(limiter.allow(first));
        assert!(!limiter.allow(first));
        assert!(limiter.allow(second));
    }
}
