use tao::platform::unix::WindowExtUnix;
use wry::WebViewBuilderExtUnix;

fn main() {
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
    let _webview = builder.with_url("https://www.google.com").build().unwrap();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = tao::event_loop::ControlFlow::Wait;

        if let tao::event::Event::WindowEvent {
            event: tao::event::WindowEvent::CloseRequested,
            ..
        } = event
        {
            *control_flow = tao::event_loop::ControlFlow::Exit
        }
    });
}
fn thing() {
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
