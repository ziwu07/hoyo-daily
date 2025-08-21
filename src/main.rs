use std::{str::FromStr, sync::atomic::Ordering};
use tao::platform::unix::WindowExtUnix;
use wry::WebViewBuilderExtUnix;

fn main() {
    __webview()
}
fn __get_cookies() {
    let _cookie_store = std::sync::Arc::new(reqwest_cookie_store::CookieStoreMutex::new({
        if let Ok(file) = std::fs::File::open("cookies.json").map(std::io::BufReader::new) {
            // use re-exported version of `CookieStore` for crate compatibility
            reqwest_cookie_store::CookieStore::load_json(file).unwrap()
        } else {
            let file = match std::fs::File::open("./webview_storage/cookies")
                .map(std::io::BufReader::new)
            {
                Ok(file) => file,
                Err(_) => {
                    __webview();
                    std::fs::File::open("./webview_storage/cookies")
                        .map(std::io::BufReader::new)
                        .unwrap()
                }
            };

            let _cookies = reqwest_cookie_store::CookieStore::load(file, |cookie_str| {
                cookie_from_str(cookie_str.to_string())
            })
            .unwrap();
            let mut file = std::fs::File::create("cookies.json").unwrap();
            _cookies.save_json(&mut file).unwrap();
            _cookies
        }
    }));

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
}
static _WEBVIEW_LOADED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
fn __webview() {
    let data_dir = std::path::PathBuf::from(r"./webview_storage");
    let event_loop = tao::event_loop::EventLoop::new();
    let window = tao::window::WindowBuilder::new()
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
    let _webview: wry::WebView = builder
        .with_url("https://www.google.com")
        .with_web_context(&mut webcontext)
        .with_on_page_load_handler(|e, _| {
            match e {
                wry::PageLoadEvent::Finished => _WEBVIEW_LOADED.store(true, Ordering::Release),
                _ => {}
            };
        })
        .build()
        .unwrap();
    event_loop.run(move |event, _, control_flow| {
        *control_flow = tao::event_loop::ControlFlow::Wait;
        if _WEBVIEW_LOADED.load(Ordering::Relaxed) {
            println!("webview_loaded: {:?}", _WEBVIEW_LOADED);
            _WEBVIEW_LOADED.store(false, Ordering::Relaxed)
        }
        if let tao::event::Event::WindowEvent {
            event: tao::event::WindowEvent::CloseRequested,
            ..
        } = event
        {
            println!("{:?}", _WEBVIEW_LOADED);
            *control_flow = tao::event_loop::ControlFlow::Exit
        }
    });
}
fn __reqests(_cookie_store: std::sync::Arc<reqwest_cookie_store::CookieStoreMutex>) {
    let client = match reqwest::blocking::Client::builder()
        .user_agent("test")
        // how to get cookies back? dont want to copy. read example from doc?
        .cookie_provider(_cookie_store)
        .https_only(true)
        .build()
    {
        Ok(client) => client,
        Err(err) => panic!("error {err}"),
    };
    let mut response = match client
        .get("https://www.google.com")
        // .body("the body of the post request")
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
    println!("{0}", text);
}
