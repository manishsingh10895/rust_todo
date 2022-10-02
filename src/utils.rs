use std::{
    error,
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
    str::FromStr,
};

use crate::config::API_URL;

/// Checks if the `Todo` server is already running
pub fn is_server_running(address: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let cmd_netstat = Command::new("netstat")
        .arg("-ano")
        //.arg("-n")
        //.arg("-o")
        .stdout(Stdio::piped())
        .spawn()
        .expect("error command");

    let cmd_netstat = cmd_netstat.wait_with_output()?;

    let output = String::from_utf8(cmd_netstat.stdout)?;

    if output.contains(address) {
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Get token saved to credentials
pub fn get_saved_token() -> Result<String, Box<dyn error::Error>> {
    let mut path = dirs::home_dir().unwrap();
    path.push("todo/credentials");

    if !path.exists() {}

    let contents = std::fs::read_to_string(path)?;

    let json = serde_json::Value::from_str(contents.as_str())?;

    let token = json.get("token").unwrap().as_str().unwrap();

    Ok(String::from(token))
}

/// Saves `Todo` login token at ~/todo/.credentials
pub fn save_token(token: &str) -> Result<(), Box<dyn error::Error>> {
    use serde_json::json;

    let mut path = dirs::home_dir().unwrap();

    path.push("todo");

    if !path.exists() {
        std::fs::create_dir(&path)?;
    }

    path.push("credentials");

    let mut file = std::fs::OpenOptions::new()
        .create(true) // Create new file if doesn't exist
        .write(true)
        .open(path)
        .unwrap();

    // Remove all contents of the file
    file.set_len(0)?;

    let data = json!({ "token": token });

    file.write(data.to_string().as_bytes())?;

    Ok(())
}

pub fn make_api_url(resource: &str) -> String {
    format!("http://{}/api/{}", API_URL.as_str(), resource)
}

#[cfg(test)]
mod utils_test {
    use super::{get_saved_token, make_api_url, save_token};
    use dirs::home_dir;
    use std::path::PathBuf;

    #[test]
    fn test_make_api_url() {
        let resource = "auth/login";

        let api_url = make_api_url(resource);

        assert_eq!(
            api_url,
            String::from("http://localhost:5900/api/auth/login")
        );
    }

    #[test]
    fn test_save_token() {
        let token = "randombytesisthe";

        let res = save_token(token);

        assert_eq!(res.is_ok(), true);

        let mut cred_path = PathBuf::new();
        cred_path.push(home_dir().unwrap());
        cred_path.push("todo");
        cred_path.push("credentials");

        let file_resp = std::fs::read_to_string(cred_path);

        assert_eq!(file_resp.is_ok(), true); //File exists

        let data = file_resp.unwrap();

        assert_eq!(data, "{\"token\":\"randombytesisthe\"}");
    }

    #[test]
    fn test_get_token() {
        let token = get_saved_token();

        assert_eq!(token.is_ok(), true);

        let token = token.unwrap();

        assert_eq!(token.len() > 1, true);
    }
}
