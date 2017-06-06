use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;
use std::os::unix::net::UnixStream;

fn get_value_from_file(path: &str) -> i64 {
    let mut file = match File::open(path) {
        Ok(val) => val,
        Err(_) => return -1
    };
    let mut buffer = String::with_capacity(4096);
    let _ = file.read_to_string(&mut buffer);
    let value = match buffer.trim_right().parse::<i64>() {
        Ok(val) => val,
        Err(_) => return -1,
    };
    value
}

fn main() {
    let socket = match UnixStream::connect("/var/run/acpid.socket") {
        Ok(sock) => sock,
        Err(e) => {
            println!("Couldn't listen: {:?}", e);
            return
        }
    };

    // Allows buffering of the socket data since we need to read a line, not until the socket closes
    let mut reader = BufReader::new(&socket);
    
    let mut response = String::with_capacity(64);

    loop {
        match reader.read_line(&mut response) {
            Ok(_) => (),
            Err(e) => {
                println!("Couldn't read: {:?}", e);
                return;
            }
        }

        {
            let responses: Vec<&str> = response.trim().split_whitespace().collect();
            let acpi = responses[1];
            match acpi {
                "BRTUP" | "BRTDN" => {
                    let current_brightness: i64 = get_value_from_file("/sys/class/backlight/radeon_bl0/brightness");
                    let max_brightness: i64 = get_value_from_file("/sys/class/backlight/radeon_bl0/max_brightness");
                    let percent: i64 = max_brightness * 5 / 100;
                    //let new_brightness: i64 = current_brightness + percent;
                    let new_brightness: i64 = if acpi == "BRTUP" {
                        current_brightness + percent
                    } else {
                        current_brightness - percent
                    };
                    let mut file = match File::create("/sys/class/backlight/radeon_bl0/brightness") {
                        Ok(file) => file,
                        Err(_) => return (),
                    };
                    match write!(file, "{}", new_brightness) {
                        Ok(_) => (),
                        Err(e) => println!("Error writing new brightness value: {}", e)
                    }
                },
                _ => {
                    println!("{}", acpi);
                },
            }
        }

        response.clear();
    }
}
