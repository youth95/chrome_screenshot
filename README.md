# Chrome Screenshot

Create Web Page Snapshot to a PNG file. it will use the local cookies in the same host.

Tips: Only supports MacOS Currently.

# Hot to use it ?

add it to your `Cargo.toml`
```
chrome_screenshot = {git = "https://github.com/youth95/chrome_screenshot", branch = "master"}
```

usage:
```rust
pub use chrome_screenshot::fetch_screenshot;

fn main(){
   let contents = fetch_screenshot(
        "https://google.com.hk", // url, the chrome headless will use all of cookies by the "google.com.hk" in your PC.
        1024, // width
        768,  // height
        "",   // wait for element selector. no wait when it's empty
        0,    // delay time after wait for element.
        false,// is it wait until navigated
    );
    std::fs::write("output.png", contents).unwrap();
}
```
