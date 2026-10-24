//! DNS for catalog media.
//!
//! Some networks answer every lookup with an address in `198.18.0.0/15` and
//! then drop the connection. Those addresses are not the CDN. When that
//! happens, look the name up over HTTPS and connect to the address that
//! actually serves the file. A normal lookup is used as-is.

use std::collections::HashMap;
use std::future::poll_fn;
use std::net::{IpAddr, SocketAddr, ToSocketAddrs};
use std::sync::{Arc, Mutex};
use std::task::{Poll, Waker};
use std::thread;
use std::time::{Duration, Instant};

use reqwest::blocking::Client;
use reqwest::dns::{Addrs, Name, Resolve, Resolving};

#[derive(Clone)]
pub struct PlaybackDns {
    inner: Arc<Inner>,
}

struct Inner {
    cache: Mutex<HashMap<String, CacheEntry>>,
    doh: Client,
}

struct CacheEntry {
    addrs: Vec<SocketAddr>,
    at: Instant,
}

impl PlaybackDns {
    pub fn new() -> anyhow::Result<Self> {
        let doh = Client::builder()
            .timeout(Duration::from_secs(8))
            .connect_timeout(Duration::from_secs(5))
            .user_agent("Streamovie-macOS/0.1.0")
            .build()?;
        Ok(Self {
            inner: Arc::new(Inner {
                cache: Mutex::new(HashMap::new()),
                doh,
            }),
        })
    }

    fn cached(&self, host: &str) -> Option<Vec<SocketAddr>> {
        let cache = self.inner.cache.lock().ok()?;
        let entry = cache.get(host)?;
        if entry.at.elapsed() > Duration::from_secs(600) {
            return None;
        }
        Some(entry.addrs.clone())
    }
}

impl Inner {
    fn store(&self, host: &str, addrs: &[SocketAddr]) {
        if let Ok(mut cache) = self.cache.lock() {
            cache.insert(
                host.to_string(),
                CacheEntry {
                    addrs: addrs.to_vec(),
                    at: Instant::now(),
                },
            );
        }
    }

    fn lookup(&self, host: &str) -> Result<Vec<SocketAddr>, String> {
        if let Ok(ip) = host.parse::<IpAddr>() {
            return Ok(vec![SocketAddr::new(ip, 0)]);
        }
        let system: Vec<SocketAddr> = (host, 0)
            .to_socket_addrs()
            .map(|addrs| addrs.collect())
            .unwrap_or_default();
        let real: Vec<SocketAddr> = system
            .iter()
            .copied()
            .filter(|addr| !is_intercepted(addr.ip()))
            .collect();
        if !real.is_empty() {
            self.store(host, &real);
            return Ok(real);
        }
        match self.doh(host) {
            Ok(ips) => {
                let addrs = ips
                    .into_iter()
                    .map(|ip| SocketAddr::new(ip, 0))
                    .collect::<Vec<_>>();
                self.store(host, &addrs);
                Ok(addrs)
            }
            Err(_) if !system.is_empty() => Ok(system),
            Err(err) => Err(err),
        }
    }

    fn doh(&self, host: &str) -> Result<Vec<IpAddr>, String> {
        let urls = [
            format!("https://1.1.1.1/dns-query?name={host}&type=A"),
            format!("https://dns.google/resolve?name={host}&type=A"),
        ];
        let mut last_error = format!("No address for {host}");
        for url in urls {
            let response = match self
                .doh
                .get(&url)
                .header("accept", "application/dns-json")
                .send()
            {
                Ok(response) if response.status().is_success() => response,
                Ok(response) => {
                    last_error = format!("DNS lookup failed ({})", response.status());
                    continue;
                }
                Err(err) => {
                    last_error = err.to_string();
                    continue;
                }
            };
            let body = response.text().map_err(|err| err.to_string())?;
            let value: serde_json::Value =
                serde_json::from_str(&body).map_err(|err| err.to_string())?;
            let mut ips = Vec::new();
            if let Some(answers) = value.get("Answer").and_then(|item| item.as_array()) {
                for answer in answers {
                    if answer.get("type").and_then(|kind| kind.as_u64()) != Some(1) {
                        continue;
                    }
                    let Some(data) = answer.get("data").and_then(|item| item.as_str()) else {
                        continue;
                    };
                    if let Ok(ip) = data.parse::<IpAddr>() {
                        if !is_intercepted(ip) {
                            ips.push(ip);
                        }
                    }
                }
            }
            if !ips.is_empty() {
                return Ok(ips);
            }
            last_error = format!("No address for {host}");
        }
        Err(last_error)
    }
}

fn is_intercepted(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip) => {
            let [first, second, _, _] = ip.octets();
            first == 198 && (second == 18 || second == 19)
        }
        IpAddr::V6(_) => false,
    }
}

impl Resolve for PlaybackDns {
    fn resolve(&self, name: Name) -> Resolving {
        let host = name.as_str().trim_end_matches('.').to_string();
        if let Some(addrs) = self.cached(&host) {
            return Box::pin(std::future::ready(Ok(Box::new(addrs.into_iter()) as Addrs)));
        }

        let inner = self.inner.clone();
        let slot = Arc::new(Mutex::new(None));
        let waker_slot: Arc<Mutex<Option<Waker>>> = Arc::new(Mutex::new(None));
        let slot_thread = slot.clone();
        let waker_thread = waker_slot.clone();
        thread::spawn(move || {
            let result = inner.lookup(&host);
            if let Ok(mut guard) = slot_thread.lock() {
                *guard = Some(result);
            }
            if let Ok(mut guard) = waker_thread.lock() {
                if let Some(waker) = guard.take() {
                    waker.wake();
                }
            }
        });

        Box::pin(poll_fn(move |cx| {
            let ready = slot.lock().ok().and_then(|mut guard| guard.take());
            if let Some(result) = ready {
                return Poll::Ready(finish(result));
            }
            if let Ok(mut guard) = waker_slot.lock() {
                *guard = Some(cx.waker().clone());
            }
            let ready = slot.lock().ok().and_then(|mut guard| guard.take());
            if let Some(result) = ready {
                return Poll::Ready(finish(result));
            }
            Poll::Pending
        }))
    }
}

fn finish(result: Result<Vec<SocketAddr>, String>) -> Result<Addrs, Box<dyn std::error::Error + Send + Sync>> {
    match result {
        Ok(addrs) => Ok(Box::new(addrs.into_iter()) as Addrs),
        Err(err) => Err(Box::new(std::io::Error::other(err))),
    }
}

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr};

    use super::is_intercepted;

    #[test]
    fn intercepted_range_is_the_benchmark_block() {
        assert!(is_intercepted(IpAddr::V4(Ipv4Addr::new(198, 18, 1, 78))));
        assert!(is_intercepted(IpAddr::V4(Ipv4Addr::new(198, 19, 0, 1))));
        assert!(!is_intercepted(IpAddr::V4(Ipv4Addr::new(185, 105, 102, 146))));
    }
}
