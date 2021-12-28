use hyper::server::conn::AddrStream;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use std::convert::Infallible;
use std::env;
use std::net::SocketAddr;
use std::path::PathBuf;
use tokio::fs;

#[derive(Clone)]
struct AppContext {
    root: PathBuf,
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install CTRL+C signal handler");
}

async fn serve(
    mut req: Request<Body>,
    ip: SocketAddr,
    context: AppContext,
) -> Result<Response<Body>, hyper::Error> {
    let path: PathBuf = req.uri().path().into();
    let local_path = context.root.join(path.strip_prefix("/").unwrap());

    let body = req.body_mut();
    let data = hyper::body::to_bytes(body).await.unwrap(); // TODO: unwrap

    let response = match match *req.method() {
        // TODO: [GET /] should return an index page which helps the user to upload or download
        // files. This can be a static html page which is included into the binary itself for less
        // dependencies.
        Method::GET => match fs::read(local_path).await {
            Ok(data) => Ok((StatusCode::OK, data.into())),
            Err(_) => Err(StatusCode::NOT_FOUND)
        },
        Method::POST => match fs::write(local_path, data).await {
            Ok(()) => Ok((StatusCode::OK, Body::empty())),
            Err(_) => Err(StatusCode::INSUFFICIENT_STORAGE)
        },
        Method::PUT => match fs::write(local_path, data).await {
            Ok(()) => Ok((StatusCode::OK, Body::empty())),
            Err(_) => Err(StatusCode::INSUFFICIENT_STORAGE)
        },
        Method::DELETE => match fs::remove_file(local_path).await {
            Ok(()) => Ok((StatusCode::OK, Body::empty())),
            Err(_) => Err(StatusCode::NOT_FOUND)
        },
        _ => Result::Err(StatusCode::METHOD_NOT_ALLOWED),
    } {
        Ok((status, body)) => Response::builder().status(status).body(body).unwrap(),
        Err(status) => Response::builder()
            .status(status)
            .body(Body::empty())
            .unwrap(),
    };

    let log_line = format!(
        "[{status}] {ip} {method} {uri}",
        status = response.status().as_u16(),
        ip = ip,
        method = req.method(),
        uri = req.uri().path()
    );
    if response.status() == 200 {
        println!("{}", log_line);
    } else {
        eprintln!("{}", log_line);
    }

    Ok(response)
}

#[tokio::main]
async fn main() {
    let args = env::args().collect::<Vec<String>>();

    let port = args
        .get(1)
        .map_or(8000, |port_str| port_str.parse().unwrap());
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    let dir = args
        .get(2)
        .map(|dir_str| PathBuf::from(dir_str))
        .unwrap_or(env::current_dir().unwrap());

    if !dir.exists() {
        fs::create_dir_all(&dir).await.unwrap();
    }

    let context = AppContext { root: dir.clone() };

    let make_service = make_service_fn(move |conn: &AddrStream| {
        let context = context.clone();
        let ip = conn.remote_addr();
        let service = service_fn(move |req| serve(req, ip, context.clone()));
        async move { Ok::<_, Infallible>(service) }
    });

    let server = Server::bind(&addr).serve(make_service);

    let graceful = server.with_graceful_shutdown(shutdown_signal());

    println!(
        "Starting server on http://localhost:{} in {}",
        port,
        dir.display()
    );

    // Run this server for... forever!
    if let Err(e) = graceful.await {
        eprintln!("server error: {}", e);
    }
}
