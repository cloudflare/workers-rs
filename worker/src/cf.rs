#[cfg(feature = "timezone")]
use crate::Result;

/// In addition to the methods on the `Request` struct, the `Cf` struct on an inbound Request contains information about the request provided by Cloudflare’s edge.
///
/// [Details](https://developers.cloudflare.com/workers/runtime-apis/request#incomingrequestcfproperties)
#[derive(Debug, Clone)]
pub struct Cf {
    inner: worker_sys::IncomingRequestCfProperties,
}

unsafe impl Send for Cf {}
unsafe impl Sync for Cf {}

impl Cf {
    #[cfg(feature = "http")]
    pub(crate) fn new(inner: worker_sys::IncomingRequestCfProperties) -> Self {
        Self { inner }
    }

    #[cfg(feature = "http")]
    pub(crate) fn inner(&self) -> &worker_sys::IncomingRequestCfProperties {
        &self.inner
    }

    /// The three-letter airport code (e.g. `ATX`, `LUX`) representing
    /// the colocation which processed the request
    pub fn colo(&self) -> String {
        self.inner.colo().unwrap()
    }

    /// The Autonomous System Number (ASN) of the request, e.g. `395747`
    pub fn asn(&self) -> u32 {
        self.inner.asn().unwrap()
    }

    /// The Autonomous System organization name of the request, e.g. `Cloudflare, Inc.`
    pub fn as_organization(&self) -> String {
        self.inner.as_organization().unwrap()
    }

    /// The two-letter country code of origin for the request.
    /// This is the same value as that provided in the CF-IPCountry header, e.g.  `"US"`
    pub fn country(&self) -> Option<String> {
        self.inner.country().unwrap()
    }

    /// The HTTP Protocol (e.g. "HTTP/2") used by the request
    pub fn http_protocol(&self) -> String {
        self.inner.http_protocol().unwrap()
    }

    /// The browser-requested prioritization information in the request object,
    ///
    /// See [this blog post](https://blog.cloudflare.com/better-http-2-prioritization-for-a-faster-web/#customizingprioritizationwithworkers) for details.
    pub fn request_priority(&self) -> Option<RequestPriority> {
        if let Some(priority) = self.inner.request_priority().unwrap() {
            let mut weight = 1;
            let mut exclusive = false;
            let mut group = 0;
            let mut group_weight = 0;

            priority
                .as_str()
                .split(';')
                .map(|key_value_pair| {
                    let mut iter = key_value_pair.split('=');

                    // this pair is guaranteed to have 2 elements
                    let key = iter.next().unwrap(); // first element
                    let value = iter.next().unwrap(); // second element

                    (key, value)
                })
                .for_each(|(key, value)| match key {
                    "weight" => weight = value.parse().unwrap(),
                    "exclusive" => exclusive = value == "1",
                    "group" => group = value.parse().unwrap(),
                    "group-weight" => group_weight = value.parse().unwrap(),
                    _ => unreachable!(),
                });

            Some(RequestPriority {
                weight,
                exclusive,
                group,
                group_weight,
            })
        } else {
            None
        }
    }

    /// The cipher for the connection to Cloudflare, e.g. "AEAD-AES128-GCM-SHA256".
    pub fn tls_cipher(&self) -> String {
        self.inner.tls_cipher().unwrap()
    }

    /// Information about the client's authorization.
    /// Only set when using Cloudflare Access or API Shield.
    pub fn tls_client_auth(&self) -> Option<TlsClientAuth> {
        self.inner.tls_client_auth().unwrap().map(Into::into)
    }

    /// The TLS version of the connection to Cloudflare, e.g. TLSv1.3.
    pub fn tls_version(&self) -> String {
        // TODO: should this be strongly typed? with ordering, etc.?
        self.inner.tls_version().unwrap()
    }

    /// City of the incoming request, e.g. "Austin".
    pub fn city(&self) -> Option<String> {
        self.inner.city().unwrap()
    }

    /// Continent of the incoming request, e.g. "NA"
    pub fn continent(&self) -> Option<String> {
        self.inner.continent().unwrap()
    }

    /// Latitude and longitude of the incoming request, e.g. (30.27130, -97.74260)
    pub fn coordinates(&self) -> Option<(f32, f32)> {
        let lat_opt = self.inner.latitude().unwrap();
        let lon_opt = self.inner.longitude().unwrap();
        match (lat_opt, lon_opt) {
            (Some(lat_str), Some(lon_str)) => {
                // SAFETY: i think this is fine..?
                let lat = lat_str.parse().unwrap();
                let lon = lon_str.parse().unwrap();
                Some((lat, lon))
            }
            _ => None,
        }
    }

    /// Postal code of the incoming request, e.g. "78701"
    pub fn postal_code(&self) -> Option<String> {
        self.inner.postal_code().unwrap()
    }

    /// Metro code (DMA) of the incoming request, e.g. "635"
    pub fn metro_code(&self) -> Option<String> {
        self.inner.metro_code().unwrap()
    }

    /// If known, the [ISO 3166-2](https://en.wikipedia.org/wiki/ISO_3166-2) name for the first level region associated with the IP address of the incoming request, e.g. "Texas".
    pub fn region(&self) -> Option<String> {
        self.inner.region().unwrap()
    }

    /// If known, the [ISO 3166-2](https://en.wikipedia.org/wiki/ISO_3166-2) code for the first level region associated with the IP address of the incoming request, e.g. "TX".
    pub fn region_code(&self) -> Option<String> {
        self.inner.region_code().unwrap()
    }

    /// **Requires** `timezone` feature. Timezone of the incoming request
    #[cfg(feature = "timezone")]
    pub fn timezone(&self) -> Result<impl chrono::TimeZone> {
        let tz = self.inner.timezone()?;
        Ok(tz.parse::<chrono_tz::Tz>()?)
    }

    /// Timezone name of the incoming request
    pub fn timezone_name(&self) -> String {
        self.inner.timezone().unwrap()
    }

    /// Whether the country of the incoming request is in the EU
    pub fn is_eu_country(&self) -> bool {
        self.inner.is_eu_country().unwrap() == Some("1".to_string())
    }
}

/// Browser-requested prioritization information.
#[derive(Debug, Clone, Copy)]
pub struct RequestPriority {
    /// The browser-requested weight for the HTTP/2 prioritization
    pub weight: usize,

    /// The browser-requested HTTP/2 exclusive flag (true for Chromium-based browsers, false for others).
    pub exclusive: bool,

    /// HTTP/2 stream ID for the request group (only non-zero for Firefox)
    pub group: usize,

    /// HTTP/2 weight for the request group (only non-zero for Firefox)
    pub group_weight: usize,
}

impl From<worker_sys::IncomingRequestCfProperties> for Cf {
    fn from(inner: worker_sys::IncomingRequestCfProperties) -> Self {
        Self { inner }
    }
}

/// Only set when using Cloudflare Access or API Shield
#[derive(Debug)]
pub struct TlsClientAuth {
    inner: worker_sys::TlsClientAuth,
}

impl TlsClientAuth {
    pub fn cert_issuer_dn_legacy(&self) -> String {
        self.inner.cert_issuer_dn_legacy().unwrap()
    }

    pub fn cert_issuer_dn(&self) -> String {
        self.inner.cert_issuer_dn().unwrap()
    }

    pub fn cert_issuer_dn_rfc2253(&self) -> String {
        self.inner.cert_issuer_dn_rfc2253().unwrap()
    }

    pub fn cert_subject_dn_legacy(&self) -> String {
        self.inner.cert_subject_dn_legacy().unwrap()
    }

    pub fn cert_verified(&self) -> String {
        self.inner.cert_verified().unwrap()
    }

    pub fn cert_not_after(&self) -> String {
        self.inner.cert_not_after().unwrap()
    }

    pub fn cert_subject_dn(&self) -> String {
        self.inner.cert_subject_dn().unwrap()
    }

    pub fn cert_fingerprint_sha1(&self) -> String {
        self.inner.cert_fingerprint_sha1().unwrap()
    }

    pub fn cert_not_before(&self) -> String {
        self.inner.cert_not_before().unwrap()
    }

    pub fn cert_serial(&self) -> String {
        self.inner.cert_serial().unwrap()
    }

    pub fn cert_presented(&self) -> String {
        self.inner.cert_presented().unwrap()
    }

    pub fn cert_subject_dn_rfc2253(&self) -> String {
        self.inner.cert_subject_dn_rfc2253().unwrap()
    }
}

impl From<worker_sys::TlsClientAuth> for TlsClientAuth {
    fn from(inner: worker_sys::TlsClientAuth) -> Self {
        Self { inner }
    }
}
