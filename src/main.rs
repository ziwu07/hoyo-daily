use log::{debug, error, info, trace, warn};
use std::{str::FromStr, sync::atomic::Ordering};
use tao::platform::unix::WindowExtUnix;
use wry::WebViewBuilderExtUnix;

// consts
const USER_AGENT: &str = "test";
const WEBVIEW_DATA_DIR: &str = "./webview_storage";
const WEBVIEW_COOKIES_FILE: &str = "./webview_storage/cookies";

fn main() {
    __webview("https://act.hoyolab.com/ys/event/signin-sea-v3/index.html?act_id=e202102251931481&hyl_auth_required=true&hyl_presentation_style=fullscreen&lang=en-us&bbs_theme=dark&bbs_theme_device=1https://www.google.com".to_string());
    // let log_file = "./hoyo-daily.log";
    // let _logger = log4rs::init_config(
    //     log4rs::Config::builder()
    //         .appender(
    //             log4rs::config::Appender::builder()
    //                 .filter(Box::new(log4rs::filter::threshold::ThresholdFilter::new(
    //                     log::LevelFilter::Debug,
    //                 )))
    //                 .build(
    //                     "logfile",
    //                     Box::new(
    //                         log4rs::append::file::FileAppender::builder()
    //                             .encoder(Box::new(log4rs::encode::pattern::PatternEncoder::new(
    //                                 "{d(%Y-%m-%d %H:%M:%S):<20.20} {l:<5.5} {L:<4.4} -- {m}{n}",
    //                             )))
    //                             .build(log_file)
    //                             .unwrap(),
    //                     ),
    //                 ),
    //         )
    //         .build(
    //             log4rs::config::Root::builder()
    //                 .appender("logfile")
    //                 .build(log::LevelFilter::Debug),
    //         )
    //         .unwrap(),
    // )
    // .unwrap();
    // let url = "https://".to_string();
    // let _cookies = __get_cookies(url);
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
        let _unknown_bool0 = bool::from_str(&list.remove(0).to_lowercase()).unwrap();
        let path = list.remove(0).to_string();
        let is_secure = bool::from_str(&list.remove(0).to_lowercase()).unwrap();
        let time: i64 = i64::from_str(list.remove(0)).unwrap();
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
                cookie::time::OffsetDateTime::from_unix_timestamp(time).unwrap(),
            ))
            .same_site(samesite)
            .secure(is_secure)
            .http_only(http_only)
            .domain(domain.clone())
            .build();
        let ret = cookie_store::Cookie::try_from_raw_cookie(
            &cookie,
            &reqwest::Url::from_str(&("https://".to_owned() + &domain)).unwrap(),
        )
        .unwrap();
        return Ok(ret);
    }
    let cookie_file = "cookies.json";
    std::sync::Arc::new(reqwest_cookie_store::CookieStoreMutex::new({
        if let Ok(file) = std::fs::File::open(cookie_file).map(std::io::BufReader::new) {
            // use re-exported version of `CookieStore` for crate compatibility
            reqwest_cookie_store::CookieStore::load_json(file).unwrap()
        } else {
            let file = match std::fs::File::open(WEBVIEW_COOKIES_FILE).map(std::io::BufReader::new)
            {
                Ok(file) => file,
                Err(_) => {
                    __webview(url);
                    std::fs::File::open(WEBVIEW_COOKIES_FILE)
                        .map(std::io::BufReader::new)
                        .unwrap()
                }
            };

            let _cookies = reqwest_cookie_store::CookieStore::load(file, |cookie_str| {
                cookie_from_str(cookie_str.to_string())
            })
            .unwrap();
            let mut file = std::fs::File::create(cookie_file).unwrap();
            _cookies.save_json(&mut file).unwrap();
            _cookies
        }
    }))
}
fn __webview(url: String) {
    let data_dir = std::path::PathBuf::from(WEBVIEW_DATA_DIR);
    let event_loop = tao::event_loop::EventLoop::new();
    let window = tao::window::WindowBuilder::new()
        .with_title("tmp name")
        .build(&event_loop)
        .unwrap();

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
        let vbox = window.default_vbox().unwrap();
        wry::WebViewBuilder::new_gtk(vbox)
    };
    let mut webcontext = wry::WebContext::new(Some(data_dir));
    let script = include_str!("./script.js");
    let _webview: wry::WebView = builder
        .with_url(url)
        .with_web_context(&mut webcontext)
        .with_headers(headers())
        .with_initialization_script(script)
        .with_user_agent(USER_AGENT)
        .with_clipboard(true)
        .with_devtools(true)
        .build()
        .unwrap();
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
fn __reqests(_cookie_store: &std::sync::Arc<reqwest_cookie_store::CookieStoreMutex>) {
    let client = match reqwest::blocking::Client::builder()
        .user_agent(USER_AGENT)
        .cookie_provider(std::sync::Arc::clone(&_cookie_store))
        .https_only(true)
        .build()
    {
        Ok(client) => client,
        Err(err) => panic!("error {err}"),
    };
    for c in _cookie_store.lock().unwrap().iter_any() {
        println!("{:?}", c)
    }
    let response = match client
        .post("")
        .headers(headers())
        // TODO: get from def file
        .query("")
        .body("")
        .send()
    {
        Ok(response) => response,
        Err(err) => {
            panic!("error {err}")
        }
    };
    for (key, value) in response.headers().iter() {
        println!("{:?} = {:?}", key, value)
    }
    let text = match response.text() {
        Ok(text) => text,
        Err(err) => panic!("error {err}"),
    };
    // println!("{0}\n\n", text);
}
fn headers() -> reqwest::header::HeaderMap {
    use reqwest::header::*;
    let mut headers = HeaderMap::new();
    headers.append(ACCEPT, "application/json".parse().unwrap());
    headers.append(ACCEPT_LANGUAGE, "en-US,en;q=0.9".parse().unwrap());
    headers.append(CONNECTION, "keep-alive".parse().unwrap());
    headers.append(
        CONTENT_TYPE,
        "application/json;charset=utf-8".parse().unwrap(),
    );
    headers.append(USER_AGENT, self::USER_AGENT.parse().unwrap());
    // TODO: parse from definition file
    headers.append(ORIGIN, "https://act.hoyolab.com".parse().unwrap());
    headers.append(REFERER, "".parse().unwrap());
    headers
}
