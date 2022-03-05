use std::{thread::sleep, time::Duration};

use headless_chrome::{
    protocol::cdp::{Page::CaptureScreenshotFormatOption, Target::CreateTarget},
    Browser,
};

use url::Url;

use crate::cookies::make_cookies;

pub fn fetch_screenshot(
    url: &str,
    width: u16,
    height: u16,
    element: &str,
    delay: u64,
    wait_until_navigated: bool,
) -> Vec<u8> {
    let url_parsed = Url::parse(url).unwrap();
    let host = url_parsed.host().unwrap().to_string();
    tracing::debug!("Parse host_key {}", host);
    let browser = Browser::default().unwrap();
    let tab = browser
        .new_tab_with_options(CreateTarget {
            url: "".to_string(),
            width: Some(width.into()),
            height: Some(height.into()),
            browser_context_id: None,
            enable_begin_frame_control: None,
            new_window: None,
            background: None,
        })
        .unwrap();
    tracing::debug!("Tab Created");
    let cs = make_cookies(host.as_str());
    tracing::debug!("Set Cookies: {}", &cs);
    tab.set_default_timeout(std::time::Duration::from_secs(60));
    tracing::debug!("Set default timeout: 60s");
    tab.set_cookies(cs.into()).unwrap();
    tab.navigate_to(url).unwrap();
    if wait_until_navigated {
        tracing::debug!("Wait for until navigated");
        tab.wait_until_navigated().unwrap();
    }

    if element.is_empty() != true {
        tracing::debug!("Wait for element {}", element);
        tab.wait_for_element(element).unwrap();
        match tab.wait_for_element(element) {
            Ok(_) => (()),
            Err(_) => {
                tracing::error!("Wait for element time out");
                tab.close(true).unwrap();
            }
        }
    }

    tracing::debug!("Delay {}s", delay);
    sleep(Duration::from_secs(delay));

    let content = tab
        .capture_screenshot(CaptureScreenshotFormatOption::Png, None, None, true)
        .unwrap();
    tab.close(true).unwrap();
    content
}
