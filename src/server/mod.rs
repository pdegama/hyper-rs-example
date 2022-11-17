use std::{convert::Infallible, net::SocketAddr};
use hyper::{Body, Request, Response, Server, body};
use hyper::service::{make_service_fn, service_fn};

// body to text
pub async fn read_response_body(res: Request<Body>) -> Result<String, hyper::Error> {
    let bytes = body::to_bytes(res.into_body()).await?;
    Ok(String::from_utf8(bytes.to_vec()).expect("response was not valid utf-8"))
}

// request handler
pub async fn handle(req: Request<Body>) -> Result<Response<Body>, Infallible> {
//    println!("{}", req.uri().path());
//    println!("{:?}", req.headers());
//    println!("{}", read_response_body(req).await.unwrap());
    let r = Response::builder()
        .status(200)
        .header("Content-Type", "text/html")
        .header("Server", "Hyper")
        .body("Hello, Rust!".into());
    Ok(r.unwrap())
}


// server listen
pub async fn server_listen(port: u16){
    let addr = SocketAddr::from(([127, 0, 0, 1], port));

    let make_svc = make_service_fn(|_conn| async {
        Ok::<_, Infallible>(service_fn(handle))
    });

    let server = Server::bind(&addr).serve(make_svc);

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
    }
}
