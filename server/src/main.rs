use may_minihttp::{HttpServerWithHeaders, HttpService, Request, Response};
use std::env;
use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};

#[derive(Clone)]
struct StaticSite {
    root: PathBuf,
}

impl HttpService for StaticSite {
    fn call(&mut self, req: Request, rsp: &mut Response) -> io::Result<()> {
        let is_head = req.method() == "HEAD";

        if req.path().split('?').next().unwrap_or("/") == "/healthz" {
            return respond(rsp, 200, "Ok", "text/plain; charset=utf-8", b"ok".to_vec(), !is_head);
        }

        if !is_head && req.method() != "GET" {
            return respond(rsp, 405, "Method Not Allowed", "text/plain; charset=utf-8", b"method not allowed".to_vec(), true);
        }

        let path = match resolve_path(&self.root, req.path()) {
            Some(path) => path,
            None => return respond(rsp, 400, "Bad Request", "text/plain; charset=utf-8", b"bad request".to_vec(), !is_head),
        };

        let file = pick_file(&self.root, path);
        match fs::read(&file) {
            Ok(body) => respond(rsp, 200, "Ok", content_type(&file), body, !is_head),
            Err(_) => {
                let not_found = self.root.join("404.html");
                match fs::read(&not_found) {
                    Ok(body) => respond(rsp, 404, "Not Found", "text/html; charset=utf-8", body, !is_head),
                    Err(_) => respond(rsp, 404, "Not Found", "text/plain; charset=utf-8", b"not found".to_vec(), !is_head),
                }
            }
        }
    }
}

fn main() {
    env_logger::init();

    let bind = env::var("BIND").unwrap_or_else(|_| "0.0.0.0:8080".to_string());
    let root = env::var("WEB_ROOT").unwrap_or_else(|_| "/var/www/html".to_string());
    let service = StaticSite { root: PathBuf::from(root) };

    let server = HttpServerWithHeaders::<_, 128>(service)
        .start(&bind)
        .expect("failed to start may_minihttp server");
    log::info!("serving static site on {bind}");
    server.wait();
}

fn resolve_path(root: &Path, raw_path: &str) -> Option<PathBuf> {
    let path = raw_path.split('?').next().unwrap_or("/");
    let mut resolved = root.to_path_buf();

    for component in Path::new(path.trim_start_matches('/')).components() {
        match component {
            Component::Normal(part) => resolved.push(part),
            Component::CurDir => {}
            Component::RootDir | Component::ParentDir | Component::Prefix(_) => return None,
        }
    }

    Some(resolved)
}

fn pick_file(root: &Path, path: PathBuf) -> PathBuf {
    if path.is_dir() {
        return path.join("index.html");
    }

    if path.is_file() {
        return path;
    }

    if path.extension().is_none() {
        let index = path.join("index.html");
        if index.starts_with(root) {
            return index;
        }
    }

    path
}

fn respond(rsp: &mut Response, code: usize, message: &'static str, content_type: &'static str, body: Vec<u8>, include_body: bool) -> io::Result<()> {
    rsp.status_code(code, message)
        .header(content_type_header(content_type))
        .header("Cache-Control: public, max-age=300")
        .header("X-Content-Type-Options: nosniff");
    if include_body {
        rsp.body_vec(body);
    }
    Ok(())
}

fn content_type_header(content_type: &'static str) -> &'static str {
    match content_type {
        "text/html; charset=utf-8" => "Content-Type: text/html; charset=utf-8",
        "text/css; charset=utf-8" => "Content-Type: text/css; charset=utf-8",
        "application/javascript; charset=utf-8" => "Content-Type: application/javascript; charset=utf-8",
        "application/json; charset=utf-8" => "Content-Type: application/json; charset=utf-8",
        "image/svg+xml" => "Content-Type: image/svg+xml",
        "image/png" => "Content-Type: image/png",
        "image/jpeg" => "Content-Type: image/jpeg",
        "image/gif" => "Content-Type: image/gif",
        "image/webp" => "Content-Type: image/webp",
        "image/x-icon" => "Content-Type: image/x-icon",
        "font/woff2" => "Content-Type: font/woff2",
        _ => "Content-Type: application/octet-stream",
    }
}

fn content_type(path: &Path) -> &'static str {
    match path.extension().and_then(|ext| ext.to_str()).unwrap_or("") {
        "html" => "text/html; charset=utf-8",
        "css" => "text/css; charset=utf-8",
        "js" => "application/javascript; charset=utf-8",
        "json" | "webmanifest" => "application/json; charset=utf-8",
        "svg" => "image/svg+xml",
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "ico" => "image/x-icon",
        "woff2" => "font/woff2",
        _ => "application/octet-stream",
    }
}
