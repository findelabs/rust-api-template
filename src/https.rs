use core::time::Duration;
use reqwest_middleware::{ClientBuilder as ReqwestClientBuilder, Extension};
use reqwest_tracing::{OtelName, SpanBackendWithUrl, TracingMiddleware};
use std::error::Error;

//pub type Client = hyper::client::Client<HttpsConnector<HttpConnector>, Body>;
type BoxResult<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;

#[derive(Debug, Clone)]
pub struct HttpsClient {
    pub client: reqwest_middleware::ClientWithMiddleware
}

//impl HttpsClient {
//    pub async fn request(&self, req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
//        let Self(internal) = self;
//        internal.request(req).await
//    }
//}

#[derive(Debug, Clone)]
pub struct ClientConfig<'a> {
    timeout: u64,
    set_nodelay: bool,
    enforce_http: bool,
    set_reuse_address: bool,
    accept_invalid_hostnames: bool,
    accept_invalid_certs: bool,
    import_cert: Option<&'a str>
}

#[derive(Debug, Clone, Default)]
pub struct ClientBuilder<'a> {
    config: ClientConfig<'a>,
}

impl Default for ClientConfig<'_> {
    fn default() -> Self {
        ClientConfig {
            timeout: 60u64,
            set_nodelay: false,
            enforce_http: false,
            set_reuse_address: false,
            accept_invalid_hostnames: false,
            accept_invalid_certs: true,
            import_cert: None,
        }
    }
}

impl Default for HttpsClient {
    fn default() -> Self {
        ClientBuilder::default().build().unwrap()
    }
}

impl<'a> ClientBuilder<'a> {
    pub fn new() -> Self {
        let config = ClientConfig::default();
        Self { config }
    }
    pub fn timeout(mut self, arg: u64) -> Self {
        self.config.timeout = arg;
        self
    }
    pub fn nodelay(mut self, arg: bool) -> Self {
        self.config.set_nodelay = arg;
        self
    }
    pub fn enforce_http(mut self, arg: bool) -> Self {
        self.config.enforce_http = arg;
        self
    }
    pub fn reuse_address(mut self, arg: bool) -> Self {
        self.config.set_reuse_address = arg;
        self
    }
    pub fn accept_invalid_hostnames(mut self, arg: bool) -> Self {
        self.config.accept_invalid_hostnames = arg;
        self
    }
    pub fn accept_invalid_certs(mut self, arg: bool) -> Self {
        self.config.accept_invalid_certs = arg;
        self
    }
    pub fn import_cert(mut self, arg: Option<&'a str>) -> Self {
        self.config.import_cert = arg;
        self
    }
    pub fn build(&mut self) -> BoxResult<HttpsClient> {
        // Create timeout Duration
        let timeout = Duration::new(self.config.timeout, 0);

		let builder = if let Some(path) = self.config.import_cert {
            let cert = &std::fs::read(path).expect("Failed reading in root cert");
            let import_cert =
                reqwest::tls::Certificate::from_pem(cert).expect("Root cert is not in PEM format");

            log::debug!("Reading in root cert at {}", &path);

            reqwest::Client::builder()
                .danger_accept_invalid_hostnames(self.config.accept_invalid_hostnames)
                .danger_accept_invalid_certs(self.config.accept_invalid_certs)
                .connect_timeout(timeout)
                .redirect(reqwest::redirect::Policy::none())
                .tcp_nodelay(self.config.set_nodelay)
                .https_only(self.config.enforce_http)
                .add_root_certificate(import_cert)
        } else {
            reqwest::Client::builder()
                .danger_accept_invalid_hostnames(self.config.accept_invalid_hostnames)
                .danger_accept_invalid_certs(self.config.accept_invalid_certs)
                .connect_timeout(timeout)
                .redirect(reqwest::redirect::Policy::none())
                .tcp_nodelay(self.config.set_nodelay)
                .https_only(self.config.enforce_http)
        };

        let reqwest_client = builder.build()?;

		let client = ReqwestClientBuilder::new(reqwest_client)
            // Insert the tracing middleware
            .with_init(Extension(OtelName("reqwest-client".into())))
            .with(TracingMiddleware::<SpanBackendWithUrl>::new())
            .build();

        Ok(HttpsClient {
            client,
        })
    }
}
