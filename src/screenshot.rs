use std::{str::FromStr, thread::sleep, time::Duration};

use headless_chrome::{
    protocol::cdp::{Network, Page::CaptureScreenshotFormatOption, Target::CreateTarget},
    Browser,
};

use url::Url;

use crate::{cookies::make_cookies, parse_cookies};

pub struct FetchScreenshotConfig {
    pub url: String,
    pub width: u16,
    pub height: u16,
    pub element: String,
    pub delay: u64,
    pub wait_until_navigated: bool,
    pub cookies: String,
}

impl Default for FetchScreenshotConfig {
    fn default() -> Self {
        Self {
            url: Default::default(),
            width: Default::default(),
            height: Default::default(),
            element: Default::default(),
            delay: Default::default(),
            wait_until_navigated: Default::default(),
            cookies: Default::default(),
        }
    }
}

pub fn fetch_screenshot(
    FetchScreenshotConfig {
        url,
        cookies,
        delay,
        element,
        height,
        wait_until_navigated,
        width,
    }: FetchScreenshotConfig,
) -> Vec<u8> {
    let url_parsed = Url::parse(url.as_str()).unwrap();
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

    let cs = match cookies.is_empty() {
        true => make_cookies(host.as_str()),
        false => crate::cookies::Cookies::new(parse_cookies(&cookies, &host)),
    };
    tracing::debug!("Tab set Cookies by {}: {}", host, &cs);

    tab.set_default_timeout(std::time::Duration::from_secs(60));
    tracing::debug!("Tab set default timeout: 60s");

    tab.set_cookies(cs.into()).unwrap();
    tab.navigate_to(url.as_str()).unwrap();
    tab.enable_log().unwrap();
    if wait_until_navigated {
        tracing::debug!("Wait for until navigated");
        tab.wait_until_navigated().unwrap();
    }

    if element.is_empty() != true {
        tracing::debug!("Wait for element {}", element);
        match tab.wait_for_element(element.as_str()) {
            Ok(_) => (()),
            Err(_) => {
                tracing::error!("Wait for element time out");
                tab.close(true).unwrap();
                panic!("Wait for element time out");
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
