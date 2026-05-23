use may_minihttp::{HttpServerWithHeaders, HttpService, Request, Response};
use std::env;
use std::ffi::OsStr;
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
            return respond(
                rsp,
                200,
                "Ok",
                "text/plain; charset=utf-8",
                b"ok".to_vec(),
                !is_head,
            );
        }

        if !is_head && req.method() != "GET" {
            return respond(
                rsp,
                405,
                "Method Not Allowed",
                "text/plain; charset=utf-8",
                b"method not allowed".to_vec(),
                true,
            );
        }

        let path = match resolve_path(&self.root, req.path()) {
            Some(path) => path,
            None => {
                return respond(
                    rsp,
                    400,
                    "Bad Request",
                    "text/plain; charset=utf-8",
                    b"bad request".to_vec(),
                    !is_head,
                )
            }
        };

        let file = pick_file(&self.root, path);
        match fs::read(&file) {
            Ok(body) => respond(rsp, 200, "Ok", content_type(&file), body, !is_head),
            Err(_) => {
                let not_found = self.root.join("404.html");
                match fs::read(&not_found) {
                    Ok(body) => respond(
                        rsp,
                        404,
                        "Not Found",
                        "text/html; charset=utf-8",
                        body,
                        !is_head,
                    ),
                    Err(_) => respond(
                        rsp,
                        404,
                        "Not Found",
                        "text/plain; charset=utf-8",
                        b"not found".to_vec(),
                        !is_head,
                    ),
                }
            }
        }
    }
}

fn main() {
    env_logger::init();

    let bind = env::var("BIND").unwrap_or_else(|_| "0.0.0.0:8080".to_string());
    let root = env::var("WEB_ROOT").unwrap_or_else(|_| "/var/www/html".to_string());
    let service = StaticSite {
        root: PathBuf::from(root),
    };

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
        let index = path.join("index.html");
        if index.is_file() {
            return index;
        }
    }

    if path.is_file() {
        return path;
    }

    if path.extension().is_some() {
        if let Some(asset) = asset_file_for_deep_path(root, &path) {
            return asset;
        }
    }

    if path.extension().is_none() {
        return root.join("index.html");
    }

    path
}

fn asset_file_for_deep_path(root: &Path, path: &Path) -> Option<PathBuf> {
    if path.extension().is_none() {
        return None;
    }

    let relative = path.strip_prefix(root).ok()?;
    let components = relative.components().collect::<Vec<_>>();
    let assets_position = components
        .iter()
        .rposition(|component| component.as_os_str() == OsStr::new("assets"))?;

    let mut asset = root.to_path_buf();
    for component in components.iter().skip(assets_position) {
        match component {
            Component::Normal(part) => asset.push(part),
            _ => return None,
        }
    }

    Some(asset)
}

fn respond(
    rsp: &mut Response,
    code: usize,
    message: &'static str,
    content_type: &'static str,
    body: Vec<u8>,
    include_body: bool,
) -> io::Result<()> {
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
        "text/plain; charset=utf-8" => "Content-Type: text/plain; charset=utf-8",
        "text/html; charset=utf-8" => "Content-Type: text/html; charset=utf-8",
        "text/css; charset=utf-8" => "Content-Type: text/css; charset=utf-8",
        "application/javascript; charset=utf-8" => {
            "Content-Type: application/javascript; charset=utf-8"
        }
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

#[cfg(test)]
mod tests {
    use super::{pick_file, resolve_path};
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_root() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("client-minihttp-test-{nanos}"));
        fs::create_dir_all(&root).expect("failed to create temp root");
        root
    }

    #[test]
    fn existing_static_assets_are_served_directly() {
        let root = temp_root();
        let asset = root.join("assets").join("app.js");
        fs::create_dir_all(asset.parent().expect("asset should have a parent"))
            .expect("failed to create asset directory");
        fs::write(&asset, b"console.log('ok');").expect("failed to write asset");

        assert_eq!(pick_file(&root, asset.clone()), asset);

        fs::remove_dir_all(root).expect("failed to remove temp root");
    }

    #[test]
    fn deep_relative_asset_paths_are_mapped_to_root_assets() {
        let root = temp_root();
        let asset = root.join("assets").join("app.js");
        fs::create_dir_all(asset.parent().expect("asset should have a parent"))
            .expect("failed to create asset directory");
        fs::write(&asset, b"console.log('ok');").expect("failed to write asset");

        let deep_asset = resolve_path(&root, "/foo/assets/app.js").expect("path should resolve");
        let prefixed_deep_asset =
            resolve_path(&root, "/maintenance/foo/assets/app.js").expect("path should resolve");

        assert_eq!(pick_file(&root, deep_asset), asset);
        assert_eq!(pick_file(&root, prefixed_deep_asset), asset);

        fs::remove_dir_all(root).expect("failed to remove temp root");
    }

    #[test]
    fn extensionless_paths_with_assets_segment_fallback_to_root_index() {
        let root = temp_root();
        let index = root.join("index.html");
        fs::write(&index, b"<main>maintenance</main>").expect("failed to write index");

        let assets = resolve_path(&root, "/foo/assets").expect("path should resolve");
        let nested_assets = resolve_path(&root, "/foo/assets/bar").expect("path should resolve");

        assert_eq!(pick_file(&root, assets), index);
        assert_eq!(pick_file(&root, nested_assets), index);

        fs::remove_dir_all(root).expect("failed to remove temp root");
    }

    #[test]
    fn missing_extensionless_paths_fallback_to_root_index() {
        let root = temp_root();
        let index = root.join("index.html");
        fs::write(&index, b"<main>maintenance</main>").expect("failed to write index");

        let abc = resolve_path(&root, "/abc").expect("path should resolve");
        let nested = resolve_path(&root, "/foo/bar").expect("path should resolve");
        let nested_with_slash = resolve_path(&root, "/some/subpath/").expect("path should resolve");

        assert_eq!(pick_file(&root, abc), index);
        assert_eq!(pick_file(&root, nested), index);
        assert_eq!(pick_file(&root, nested_with_slash), index);

        fs::remove_dir_all(root).expect("failed to remove temp root");
    }

    #[test]
    fn missing_assets_with_extensions_do_not_fallback_to_index() {
        let root = temp_root();
        fs::write(root.join("index.html"), b"<main>maintenance</main>")
            .expect("failed to write index");

        let missing_js = resolve_path(&root, "/assets/missing.js").expect("path should resolve");
        let missing_css = resolve_path(&root, "/assets/missing.css").expect("path should resolve");
        let missing_svg = resolve_path(&root, "/assets/missing.svg").expect("path should resolve");

        assert_eq!(
            pick_file(&root, missing_js),
            root.join("assets").join("missing.js")
        );
        assert_eq!(
            pick_file(&root, missing_css),
            root.join("assets").join("missing.css")
        );
        assert_eq!(
            pick_file(&root, missing_svg),
            root.join("assets").join("missing.svg")
        );

        fs::remove_dir_all(root).expect("failed to remove temp root");
    }

    #[test]
    fn path_traversal_is_rejected() {
        let root = temp_root();

        assert!(resolve_path(&root, "/../secret.txt").is_none());
        assert!(resolve_path(&root, "/foo/../../secret.txt").is_none());

        fs::remove_dir_all(root).expect("failed to remove temp root");
    }
}
