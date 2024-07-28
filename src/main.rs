use std::str::FromStr;
use tao::platform::unix::WindowExtUnix;
use wry::WebViewBuilderExtUnix;

fn main() {}
fn __get_cookies() {
    let file = std::fs::File::open("./webview_storage/cookies")
        .map(std::io::BufReader::new)
        .unwrap();
    let cookies = reqwest_cookie_store::CookieStore::load(file, |cookie_str| {
        cookie_from_str(cookie_str.to_string())
    })
    .unwrap();
    for c in cookies.iter_any() {
        println!("{:?}\n", c);
    }

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
        let _unknown_bool0 = bool::from_str(&list.remove(0).to_lowercase()).unwrap();
        let path = list.remove(0).to_string();
        let is_secure = bool::from_str(&list.remove(0).to_lowercase()).unwrap();
        let time: i64 = i64::from_str(list.remove(0)).unwrap();
        let name = list.remove(0).to_string();
        let value = list.remove(0).to_string();
        let samesite = match list.remove(0) {
            "Lax" => cookie::SameSite::Lax,
            "Strict" => cookie::SameSite::Strict,
            _ => cookie::SameSite::None,
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
    let _webview = builder
        .with_url("https://www.google.com")
        .with_web_context(&mut webcontext)
        .with_on_page_load_handler(page_loaded)
        .build()
        .unwrap();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = tao::event_loop::ControlFlow::Wait;

        if let tao::event::Event::WindowEvent {
            event: tao::event::WindowEvent::CloseRequested,
            ..
        } = event
        {
            println!("webview exit");
            *control_flow = tao::event_loop::ControlFlow::Exit
        }
    });
    fn page_loaded(loadevent: wry::PageLoadEvent, eventstr: String) {
        match loadevent {
            wry::PageLoadEvent::Started => println!("load started"),
            wry::PageLoadEvent::Finished => println!("page loaded:"),
        };
        println!("{:?}", eventstr)
    }
}

fn __reqests() {
    let cookies = std::sync::Arc::new(reqwest_cookie_store::CookieStoreMutex::new(
        reqwest_cookie_store::CookieStore::new(None),
    ));
    let client = match reqwest::blocking::Client::builder()
        .user_agent("test")
        .cookie_provider(std::sync::Arc::clone(&cookies))
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
    for c in cookies.lock().unwrap().iter_any() {
        println!("{:?}", c)
    }
}
