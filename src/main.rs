#[macro_use]
extern crate lazy_static;

use hyper::{Body, Client, Method, Request};
use std::{collections::HashSet, process::Command};
use warp::{Filter, Rejection, Reply};

use std::sync::Mutex;

type Result<T> = std::result::Result<T, Rejection>;

lazy_static! {
    static ref CHROME_INSTANCES: Mutex<HashSet<u32>> = Mutex::new(HashSet::new());
    static ref DEFAULT_PORT: u32 = {
        let default_port = std::env::args()
            .nth(4)
            .unwrap_or("9222".into())
            .parse::<u32>()
            .unwrap_or_default();
        let default_port = if default_port == 0 {
            9222
        } else {
            default_port
        };

        default_port
    };
    static ref DEFAULT_PORT_SERVER: u16 = {
        let default_port = std::env::args()
            .nth(5)
            .unwrap_or("6000".into())
            .parse::<u16>()
            .unwrap_or_default();
        let default_port = if default_port == 0 {
            6000
        } else {
            default_port
        };

        default_port
    };
    static ref CHROME_ARGS: [&'static str; 66] = {
        let headless = std::env::args()
        .nth(6)
        .unwrap_or("true".into());

        let headless = if headless != "false" {
            match std::env::var("HEADLESS") {
                Ok(h) => {
                    if h == "new" {
                        "--headless=new"
                    } else {
                        "--headless=old"
                    }
                }
                _ => "--headless=old"
            }
        } else {
            ""
        };

        [
            // *SPECIAL*
            "--remote-debugging-address=0.0.0.0",
            "--remote-debugging-port=9222",
            // *SPECIAL*
            headless,
            "--no-sandbox",
            "--no-first-run",
            "--hide-scrollbars",
            // "--allow-pre-commit-input",
            "--user-data-dir=~/.config/google-chrome",
            "--allow-running-insecure-content",
            "--autoplay-policy=user-gesture-required",
            "--ignore-certificate-errors",
            "--no-default-browser-check",
            "--no-zygote",
            "--disable-gpu",
            "--disable-gpu-sandbox",
            "--disable-setuid-sandbox",
            "--disable-dev-shm-usage", // required or else container will crash not enough memory
            "--disable-threaded-scrolling",
            "--disable-demo-mode",
            "--disable-dinosaur-easter-egg",
            "--disable-fetching-hints-at-navigation-start",
            "--disable-site-isolation-trials",
            "--disable-web-security",
            "--disable-threaded-animation",
            "--disable-sync",
            "--disable-print-preview",
            "--disable-partial-raster",
            "--disable-in-process-stack-traces",
            "--disable-v8-idle-tasks",
            "--disable-low-res-tiling",
            "--disable-speech-api",
            "--disable-smooth-scrolling",
            "--disable-default-apps",
            "--disable-prompt-on-repost",
            "--disable-domain-reliability",
            "--disable-component-update",
            "--disable-background-timer-throttling",
            "--disable-breakpad",
            "--disable-software-rasterizer",
            "--disable-extensions",
            "--disable-popup-blocking",
            "--disable-hang-monitor",
            "--disable-image-animation-resync",
            "--disable-client-side-phishing-detection",
            "--disable-component-extensions-with-background-pages",
            "--disable-ipc-flooding-protection",
            "--disable-background-networking",
            "--disable-renderer-backgrounding",
            "--disable-field-trial-config",
            "--disable-back-forward-cache",
            "--disable-backgrounding-occluded-windows",
            // "--enable-automation",
            "--log-level=3",
            "--enable-logging=stderr",
            "--enable-features=SharedArrayBuffer,NetworkService,NetworkServiceInProcess",
            "--metrics-recording-only",
            "--use-mock-keychain",
            "--force-color-profile=srgb",
            "--mute-audio",
            "--no-service-autorun",
            "--password-store=basic",
            "--export-tagged-pdf",
            "--no-pings",
            "--use-gl=swiftshader",
            "--window-size=1920,1080",
            "--disable-vulkan-fallback-to-gl-for-testing",
            "--disable-vulkan-surface",
            "--disable-features=AudioServiceOutOfProcess,IsolateOrigins,site-per-process,ImprovedCookieControls,LazyFrameLoading,GlobalMediaControls,DestroyProfileOnBrowserClose,MediaRouter,DialMediaRouteProvider,AcceptCHFrame,AutoExpandDetailsElement,CertificateTransparencyComponentUpdater,AvoidUnnecessaryBeforeUnloadCheckSync,Translate"
        ]
    };
}

/// shutdown the chrome instance by process id
#[cfg(target_os = "windows")]
fn shutdown(pid: &u32) {
    let _ = Command::new("taskkill")
        .args(["/PID", &pid.to_string(), "/F"])
        .spawn();
}

/// shutdown the chrome instance by process id
#[cfg(not(target_os = "windows"))]
fn shutdown(pid: &u32) {
    let _ = Command::new("kill").args(["-9", &pid.to_string()]).spawn();
}

/// fork a chrome process
fn fork(chrome_path: &String, chrome_address: &String, port: Option<u32>) -> String {
    let mut command = Command::new(chrome_path);
    let mut chrome_args = CHROME_ARGS.map(|e| e.to_string());

    if !chrome_address.is_empty() {
        chrome_args[0] = format!("--remote-debugging-address={}", &chrome_address.to_string());
    }

    match port {
        Some(port) => {
            chrome_args[1] = format!("--remote-debugging-port={}", &port.to_string());
        }
        _ => (),
    };

    let id = if let Ok(child) = command.args(chrome_args).spawn() {
        let cid = child.id();
        println!("Chrome PID: {}", cid);

        match CHROME_INSTANCES.lock() {
            Ok(mut mutx) => {
                mutx.insert(cid.to_owned());
            }
            _ => (),
        }

        cid
    } else {
        println!("chrome command didn't start");
        0
    }
    .to_string();

    id
}

/// get json endpoint for chrome instance proxying
async fn version_handler(endpoint_path: Option<&str>) -> Result<impl Reply> {
    lazy_static! {
        static ref ENDPOINT: String = {
            let default_port = std::env::args()
                .nth(4)
                .unwrap_or("9222".into())
                .parse::<u32>()
                .unwrap_or_default();
            let default_port = if default_port == 0 {
                9222
            } else {
                default_port
            };
            format!("http://127.0.0.1:{}/json/version", default_port)
        };
    }
    let req = Request::builder()
        .method(Method::GET)
        .uri(endpoint_path.unwrap_or(ENDPOINT.as_str()))
        .header("content-type", "application/json")
        .body(Body::default())
        .unwrap_or_default();

    let client = Client::new();
    let resp = client.request(req).await.unwrap_or_default();

    Ok(resp)
}

/// get json endpoint for chrome instance proxying
async fn version_handler_with_path(port: u32) -> Result<impl Reply> {
    let req = Request::builder()
        .method(Method::GET)
        .uri(format!("http://127.0.0.1:{}/json/version", port))
        .header("content-type", "application/json")
        .body(Body::default())
        .unwrap_or_default();

    let client = Client::new();
    let resp = client.request(req).await.unwrap_or_default();

    Ok(resp)
}

/// health check server
async fn hc() -> Result<impl Reply> {
    Ok("healthy!")
}

#[tokio::main]
async fn main() {
    let chrome_path = std::env::args().nth(1).unwrap_or("".to_string());
    let chrome_address = std::env::args().nth(2).unwrap_or("".to_string());
    let auto_start = std::env::args().nth(3).unwrap_or_default();

    let chrome_path_1 = chrome_path.clone();
    let chrome_address_1 = chrome_address.clone();
    let chrome_address_2 = chrome_address.clone();

    // init chrome process
    if auto_start == "init" {
        fork(&chrome_path, &chrome_address_1, Some(*DEFAULT_PORT));
    }

    let health_check = warp::path::end()
        .and_then(hc)
        .with(warp::cors().allow_any_origin());

    let chrome_init = move || fork(&chrome_path, &chrome_address_1, None);
    let chrome_init_args = move |port: u32| fork(&chrome_path_1, &chrome_address_2, Some(port));
    let json_args = move || version_handler(None);
    let json_args_with_port = move |port| version_handler_with_path(port);

    let fork = warp::path!("fork").map(chrome_init);
    let fork_with_port = warp::path!("fork" / u32).map(chrome_init_args);

    let version = warp::path!("json" / "version").and_then(json_args);

    let version_with_port = warp::path!("json" / "version" / u32).and_then(json_args_with_port);

    let shutdown = warp::path!("shutdown" / u32).map(|cid: u32| {
        match CHROME_INSTANCES.lock() {
            Ok(mutx) => {
                let pid = mutx.get(&cid);

                match pid {
                    Some(pid) => {
                        shutdown(pid);
                    }
                    _ => (),
                }
            }
            _ => (),
        };

        "0"
    });

    let ctrls = warp::post().and(fork.with(warp::cors().allow_any_origin()));
    let ctrls_fork = warp::post().and(fork_with_port.with(warp::cors().allow_any_origin()));
    let shutdown = warp::post().and(shutdown.with(warp::cors().allow_any_origin()));
    let version_port = warp::post().and(version_with_port.with(warp::cors().allow_any_origin()));

    let routes = warp::get()
        .and(health_check)
        .or(shutdown)
        .or(version)
        .or(ctrls_fork)
        .or(version_port)
        .or(ctrls);

    println!(
        "Chrome server at {}:{}",
        if chrome_address.is_empty() {
            "localhost"
        } else {
            &chrome_address
        },
        DEFAULT_PORT_SERVER.to_string()
    );
    warp::serve(routes)
        .run(([0, 0, 0, 0], DEFAULT_PORT_SERVER.to_owned()))
        .await;
}
