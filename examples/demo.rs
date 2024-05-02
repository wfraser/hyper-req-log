use std::convert::Infallible;
use bytes::Bytes;
use http_body_util::Full;
use hyper::{Method, StatusCode};
use hyper_util::rt::TokioIo;
use hyper_req_log::LogRequest;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080").await?;
    loop {
        let (stream, remote) = listener.accept().await?;
        let io = TokioIo::new(stream);
        tokio::task::spawn(async move {
            if let Err(err) = hyper::server::conn::http1::Builder::new()
                .serve_connection(io, hyper::service::service_fn(|req| async move {
                    let mut log = LogRequest::from_request(&req);
                    log.set_action("unset");
                    log.set_remote(remote);

                    let resp = handle_request(req, &mut log);

                    log.set_response(&resp);
                    Ok::<_, Infallible>(resp)
                }))
                .await
            {
                eprintln!("Error serving connection: {:?}", err);
            }
        });
    }
}

fn handle_request(req: hyper::Request<hyper::body::Incoming>, log: &mut LogRequest<&'static str>)
    -> hyper::Response<Full<Bytes>>
{
    let resp = hyper::Response::builder()
        .header("server", "hyper-req-log demo server")
        .header("content-type", "text/plain; charset=utf-8");

    // this is bad, don't do this for real
    match req.headers().get("authorization").map(|v| v.as_bytes()).unwrap_or(b"") {
        b"Basic YWxpY2U6bG9va2dsYXNz" => { // "alice:lookglass"
            log.set_user("alice@example.com".to_owned());
        }
        _ => {
            log.set_action("unauthorized");
            return resp.status(401)
                .body(Full::from("authorization required"))
                .unwrap();
        }
    }

    match *req.method() {
        Method::GET => {
            log.set_action("get");
            let path = req.uri().path();
            resp.body(Full::from(format!("get from path {path}"))).unwrap()
        }
        Method::POST => {
            log.set_action("post");
            resp.body(Full::from("post ok")).unwrap()
        }
        _ => {
            log.set_action("error");
            resp.status(StatusCode::METHOD_NOT_ALLOWED)
                .body(Full::from(""))
                .unwrap()
        }
    }
}
