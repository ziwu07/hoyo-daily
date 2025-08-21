use core::panic;
use log::{debug, error, warn};
use std::str::FromStr;
use tao::platform::unix::WindowExtUnix;
use wry::WebViewBuilderExtUnix;

// consts
const __USER_AGENT: &str = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36";

fn main() {
    let log_file = "./hoyo-daily.log";
    let _logger = log4rs::init_config(
        log4rs::Config::builder()
            .appender(
                log4rs::config::Appender::builder()
                    .filter(Box::new(log4rs::filter::threshold::ThresholdFilter::new(
                        log::LevelFilter::Info,
                    )))
                    .build(
                        "logfile",
                        Box::new(
                            log4rs::append::file::FileAppender::builder()
                                .encoder(Box::new(log4rs::encode::pattern::PatternEncoder::new(
                                    "{d(%Y-%m-%d %H:%M:%S):<20.20} {l:<5.5} {L:<4.4} -- {m}{n}",
                                )))
                                .build(log_file)
                                .unwrap(),
                        ),
                    ),
            )
            .build(
                log4rs::config::Root::builder()
                    .appender("logfile")
                    .build(log::LevelFilter::Info),
            )
            .unwrap(),
    )
    .unwrap();
    let cookie_url = "https://act.hoyolab.com/ys/event/signin-sea-v3/index.html?act_id=e202102251931481&hyl_auth_required=true&hyl_presentation_style=fullscreen&lang=en-us&bbs_theme=dark&bbs_theme_device=1https://www.google.com".to_string();
    let _cookies = __get_cookies(cookie_url);
    let url = "https://sg-hk4e-api.hoyolab.com/event/sol/sign".to_string();
    let query = [("lang", "en-us"), ("act_id", "e202102251931481")];
    let body = "".to_string();
    let origin = "https://act.hoyolab.com".to_string();
    let referer =
        "https://act.hoyolab.com/ys/event/signin-sea-v3/index.html?act_id=e202102251931481"
            .to_string();
    let response = __reqests(&_cookies, url, query, body, origin, referer);
    println!("{:?}", response.text().unwrap());
}
fn __get_cookies(url: String) -> std::sync::Arc<reqwest_cookie_store::CookieStoreMutex> {
    fn cookie_from_str(
        cookie_str: String,
    ) -> Result<cookie_store::Cookie<'static>, cookie_store::CookieError> {
        let mut _str = cookie_str.clone();
        let mut http_only = true;
        match _str.strip_prefix("#HttpOnly_") {
            Some(new_str) => _str = new_str.to_string(),
            None => http_only = false,
        };
        let mut list: Vec<&str> = _str.split('\t').collect();
        let domain: String = list.remove(0).to_string();
        // base site?
        let _unknown_bool0 = bool::from_str(&list.remove(0).to_lowercase()).unwrap_or_default();
        let path = list.remove(0).to_string();
        let is_secure = bool::from_str(&list.remove(0).to_lowercase()).unwrap_or_else(|_| true);
        let time: i64 = i64::from_str(list.remove(0)).unwrap_or_else(|_| i64::MAX);
        let name = list.remove(0).to_string();
        let value = list.remove(0).to_string();
        let samesite = match list.remove(0) {
            "Strict" => cookie::SameSite::Strict,
            "None" => cookie::SameSite::None,
            _ => cookie::SameSite::Lax,
        };
        let cookie = cookie::Cookie::build((name, value))
            .path(path)
            .expires(cookie::Expiration::from(
                match cookie::time::OffsetDateTime::from_unix_timestamp(time) {
                    Ok(time) => Some(time),
                    Err(_) => None,
                },
            ))
            .same_site(samesite)
            .secure(is_secure)
            .http_only(http_only)
            .domain(domain.clone())
            .build();
        let ret = cookie_store::Cookie::try_from_raw_cookie(
            &cookie,
            &reqwest::Url::from_str(&("https://".to_owned() + &domain)).unwrap_or_else(|_| {
                error!("invalid cookie url from webview {domain}");
                panic!("invalid cookie url from webview {domain}")
            }),
        );
        return ret;
    }
    let cookie_file = "./webview_storage/cookies.json";
    let webview_cookies_file = "./webview_storage/cookies";
    std::sync::Arc::new(reqwest_cookie_store::CookieStoreMutex::new({
        if let Ok(file) = std::fs::File::open(cookie_file).map(std::io::BufReader::new) {
            // use re-exported version of `CookieStore` for crate compatibility
            reqwest_cookie_store::CookieStore::load_json(file).unwrap_or_else(|_| {
                error!("failed to load cookies from json file");
                panic!("failed to load cookies from json file");
            })
        } else {
            let file = std::fs::File::open(webview_cookies_file)
                .map(std::io::BufReader::new)
                .unwrap_or_else(|_| {
                    __webview(url);
                    std::fs::File::open(webview_cookies_file)
                        .map(std::io::BufReader::new)
                        .unwrap_or_else(|err| {
                            error!("{err}");
                            panic!("{err}")
                        })
                });

            let _cookies = reqwest_cookie_store::CookieStore::load(file, |cookie_str| {
                cookie_from_str(cookie_str.to_string())
            })
            .unwrap_or_else(|err| {
                error!("failed to parse webview cookie file {err}");
                panic!("check log file")
            });
            let mut file = std::fs::File::create(cookie_file).unwrap_or_else(|err| {
                error!("failed to create json cookie file {err}");
                panic!("{err}")
            });
            _cookies.save_json(&mut file).unwrap_or_else(|err| {
                error!("failed to save json cookie file {err}");
                panic!("check log file")
            });
            _cookies
        }
    }))
}
fn __webview(url: String) {
    let data_dir = std::path::PathBuf::from("./webview_storage");
    let event_loop = tao::event_loop::EventLoop::new();
    let window = tao::window::WindowBuilder::new()
        .with_title("tmp name")
        .build(&event_loop)
        .unwrap_or_else(|err| {
            error!("failed to create window {err}");
            panic!("failed to create window {err}")
        });

    #[cfg(any(
        target_os = "windows",
        target_os = "macos",
        target_os = "ios",
        target_os = "android"
    ))]
    let builder = wry::WebViewBuilder::new(&window);

    #[cfg(not(any(
        target_os = "windows",
        target_os = "macos",
        target_os = "ios",
        target_os = "android"
    )))]
    let builder = {
        let vbox = window.default_vbox().unwrap_or_else(|| {
            error!("failed to create window");
            panic!("failed to create window")
        });
        wry::WebViewBuilder::new_gtk(vbox)
    };
    let mut webcontext = wry::WebContext::new(Some(data_dir));
    let script = include_str!("./script.js");
    let _webview: wry::WebView = builder
        .with_url(url)
        .with_web_context(&mut webcontext)
        .with_initialization_script(script)
        .with_user_agent(__USER_AGENT)
        .with_clipboard(true)
        .with_devtools(true)
        .build()
        .unwrap_or_else(|err| {
            error!("failed to create webview {err}");
            panic!("failed to create webview {err}");
        });
    event_loop.set_device_event_filter(tao::event_loop::DeviceEventFilter::Always);
    event_loop.run(move |event, _, control_flow| {
        *control_flow = tao::event_loop::ControlFlow::Wait;
        match event {
            tao::event::Event::WindowEvent {
                event: tao::event::WindowEvent::CloseRequested,
                ..
            } => *control_flow = tao::event_loop::ControlFlow::Exit,
            _ => (),
        };
    });
}
fn __reqests<T: serde::Serialize>(
    _cookie_store: &std::sync::Arc<reqwest_cookie_store::CookieStoreMutex>,
    url: String,
    query: T,
    body: String,
    origin: String,
    referer: String,
) -> reqwest::blocking::Response {
    let client = reqwest::blocking::Client::builder()
        .user_agent(USER_AGENT)
        .cookie_provider(std::sync::Arc::clone(&_cookie_store))
        .https_only(true)
        .build()
        .unwrap_or_else(|err| {
            error!("api client failed {err}");
            panic!("api client failed: {err}");
        });
    use reqwest::header::*;
    let mut headers = HeaderMap::new();
    headers.append(ACCEPT, "application/json".parse().unwrap());
    headers.append(ACCEPT_LANGUAGE, "en-US,en;q=0.9".parse().unwrap());
    headers.append(CONNECTION, "keep-alive".parse().unwrap());
    headers.append(
        CONTENT_TYPE,
        "application/json;charset=utf-8".parse().unwrap(),
    );
    // TODO: get user agent value from file with Option and default, also replace the unwrap with
    // defaults and waring.
    headers.append(USER_AGENT, __USER_AGENT.parse().unwrap());
    headers.append(reqwest::header::ORIGIN, origin.parse().unwrap());
    headers.append(reqwest::header::REFERER, referer.parse().unwrap());
    let response = client
        .post(url)
        .headers(headers)
        .query(&query)
        .body(body)
        .send()
        .unwrap_or_else(|err| {
            error!("api request failed {err}");
            panic!("api request failed: {err}")
        });
    response
}
