#[macro_use]
extern crate lazy_static;

use hyper::{Body, Client, Method, Request};
use std::{collections::HashSet, process::Command};
use warp::{Filter, Rejection, Reply};

use std::sync::Mutex;

type Result<T> = std::result::Result<T, Rejection>;

/// static chrome arguments to start application
static CHROME_ARGS: [&'static str; 48] = [
    "--headless",
    "--no-sandbox",
    "--no-first-run",
    // *special*
    "--remote-debugging-address=0.0.0.0",
    "--remote-debugging-port=9222",
    // *special*
    "--user-data-dir=~/.config/google-chrome",
    "--hide-scrollbars",
    "--disable-site-isolation-trials",
    // r#"--js-flags="--max-old-space-size=500"#,
    "--allow-pre-commit-input",
    "--allow-running-insecure-content",
    "--autoplay-policy=user-gesture-required",
    "--ignore-certificate-errors",
    "--no-default-browser-check",
    "--disable-sync",
    "--disable-gpu",
    "--disable-default-apps",
    "--disable-prompt-on-repost",
    "--disable-dev-shm-usage",
    "--disable-domain-reliability",
    "--disable-component-update",
    "--disable-background-timer-throttling",
    "--disable-breakpad",
    "--disable-gpu-sandbox",
    "--disable-software-rasterizer",
    "--disable-extensions",
    "--disable-popup-blocking",
    "--disable-setuid-sandbox",
    "--disable-hang-monitor",
    "--disable-client-side-phishing-detection",
    "--disable-backgrounding-occluded-windows",
    "--disable-component-extensions-with-background-pages",
    "--disable-ipc-flooding-protection",
    "--disable-background-networking",
    "--disable-renderer-backgrounding",
    "--disable-field-trial-config",
    "--disable-back-forward-cache",
    "--enable-automation",
    "--metrics-recording-only",
    "--use-mock-keychain",
    "--force-color-profile=srgb",
    "--mute-audio",
    "--enable-logging=0",
    "--enable-features=NetworkService,NetworkServiceInProcess",
    "--no-service-autorun",
    "--password-store=basic",
    "--export-tagged-pdf",
    "--no-pings",
    "--disable-features=site-per-process,AutofillServerCommunication,ImprovedCookieControls,LazyFrameLoading,GlobalMediaControls,DialMediaRouteProvider,CertificateTransparencyComponentUpdater,AutoExpandDetailsElement,DestroyProfileOnBrowserClose,BackForwardCache,MediaRouter,OptimizationHints,AcceptCHFrame,AvoidUnnecessaryBeforeUnloadCheckSync,Translate,ScriptStreaming,InterestFeedContentSuggestions,BlinkGenPropertyTrees"
];

lazy_static! {
    static ref CHROME_INSTANCES: Mutex<HashSet<u32>> = Mutex::new(HashSet::new());
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
        chrome_args[3] = format!("--remote-debugging-address={}", &chrome_address.to_string());
    }

    match port {
        Some(port) => {
            chrome_args[4] = format!("--remote-debugging-port={}", &port.to_string());
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
async fn version_handler() -> Result<impl Reply> {
    let req = Request::builder()
        .method(Method::GET)
        .uri("http://127.0.0.1:9222/json/version")
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
    let chrome_path_1 = chrome_path.clone();

    let chrome_address = std::env::args().nth(2).unwrap_or("".to_string());
    let chrome_address_1 = chrome_address.clone();
    let chrome_address_2 = chrome_address.clone();

    let auto_start = std::env::args().nth(3).unwrap_or_default();

    // init chrome process
    if auto_start == "init" {
        fork(&chrome_path, &chrome_address_1, None);
    }

    let chrome_init = move || fork(&chrome_path, &chrome_address_1, None);

    let chrome_init_args = move |port: u32| fork(&chrome_path_1, &chrome_address_2, Some(port));

    let health_check = warp::path::end()
        .and_then(hc)
        .with(warp::cors().allow_any_origin());

    let fork = warp::path!("fork").map(chrome_init);
    let fork_with_port = warp::path!("fork" / u32).map(chrome_init_args);

    let version = warp::path!("json" / "version").and_then(version_handler);

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

    let routes = warp::get()
        .and(health_check)
        .or(shutdown)
        .or(version)
        .or(ctrls_fork)
        .or(ctrls);

    println!("Chrome server at {}:6000", if chrome_address.is_empty() { "localhost" } else { &chrome_address });
    warp::serve(routes).run(([0, 0, 0, 0], 6000)).await;
}