use futures_util::stream::{Stream, StreamExt, TryStreamExt};
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server, body};
use std::convert::Infallible;
use std::io;
use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio_rustls::server::TlsStream;
use tokio_rustls::TlsAcceptor;

// body to text
pub async fn read_response_body(res: Request<Body>) -> Result<String, hyper::Error> {
    let bytes = body::to_bytes(res.into_body()).await?;
    Ok(String::from_utf8(bytes.to_vec()).expect("response was not valid utf-8"))
}

// request handler
pub async fn handle(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    println!("{}", req.uri().path());
    println!("{:?}", req.headers());
    println!("{}", read_response_body(req).await.unwrap());
    let r = Response::builder()
        .status(200)
        .header("Content-Type", "text/html")
        .header("Server", "Hyper")
        .body("Hello, Rust!".into());
    Ok(r.unwrap())
}

fn error(err: String) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::Other, err)
}

pub async fn server_listen(port: u16) -> io::Result<()> {

    let addr = SocketAddr::from(([127, 0, 0, 1], port));

    // Build TLS configuration.
    let tls_cfg = {
        // Load public certificate.
        let mut cert_reader = io::BufReader::new(std::fs::File::open("cert.pem")?);
        let certs = rustls::internal::pemfile::certs(&mut cert_reader).unwrap();
        // Load private key.
        let mut key_reader = io::BufReader::new(std::fs::File::open("key.pem")?);
        // Load and return a single private key.
        let mut keys = rustls::internal::pemfile::pkcs8_private_keys(&mut key_reader).unwrap();
        // Do not use client certificate authentication.
        let mut cfg = rustls::ServerConfig::new(rustls::NoClientAuth::new());
        // Select a certificate to use.
        cfg.set_single_cert(certs, keys.remove(0)).unwrap();
        // Configure ALPN to accept HTTP/2, HTTP/1.1 in that order.
        cfg.set_protocols(&[b"h2".to_vec(), b"http/1.1".to_vec()]);
        std::sync::Arc::new(cfg)
    };

    // Create a TCP listener via tokio.
    let mut tcp = TcpListener::bind(&addr).await?;
    let tls_acceptor = &TlsAcceptor::from(tls_cfg);
    // Prepare a long-running future stream to accept and serve clients.
    let incoming_tls_stream = tcp
        .incoming()
        .map_err(|e| error(format!("Incoming failed: {:?}", e)))
        // (base: https://github.com/cloudflare/wrangler/pull/1485/files)
        .filter_map(|s| async {
            let client = match s {
                Ok(x) => x,
                Err(_e) => {
                    //eprintln!("Failed to accept client: {}", e);
                    return None;
                }
            };
            match tls_acceptor.accept(client).await {
                Ok(x) => Some(Ok(x)),
                Err(_e) => {
                    //eprintln!("Client connection error: {}", e);
                    None
                }
            }
        });

    let make_svc = make_service_fn(|_conn| async {
        // service_fn converts our function into a `Service`
        Ok::<_, Infallible>(service_fn(handle))
    });

    let server = Server::builder(HyperAcceptor {
        acceptor: Box::pin(incoming_tls_stream),
    })
        .serve(make_svc);

    // Run this server for... forever!
    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
    Ok(())
}

struct HyperAcceptor<S> {
    acceptor: core::pin::Pin<Box<S>>,
}

impl<S> hyper::server::accept::Accept for HyperAcceptor<S>
    where
        S: Stream<Item = Result<TlsStream<TcpStream>, io::Error>>,
{
    type Conn = TlsStream<TcpStream>;
    type Error = io::Error;

    fn poll_accept(
        mut self: core::pin::Pin<&mut Self>,
        cx: &mut core::task::Context,
    ) -> core::task::Poll<Option<Result<Self::Conn, Self::Error>>> {
        self.acceptor.as_mut().poll_next(cx)
    }
}