/*
    Touch utility implementation.
    Copyright 2017 Sam Saint-Pettersen.

    Released under the MIT License.
*/

extern crate clioptions;
extern crate filetime;
extern crate time;
extern crate litepattern;
use clioptions::CliOptions;
use filetime::FileTime;
use litepattern::LPattern;
use std::fs;
use std::fs::File;
use std::path::Path;
use std::process::exit;

fn display_error(program: &str, err: &str) {
    println!("touch: {}.", err);
    println!("Try '{} -h | --help' for more information.", program);
    exit(-1);
}

fn display_version() {
    println!("touch v0.1.0.");
    exit(0);
}

fn display_help(program: &str) {
    println!("Touch utility implementation.");
    println!("Copyright 2017 Sam Saint-Pettersen.");
    println!("\nReleased under the MIT License.\n");
    println!("Usage: {} <file> [options] \n", program);
    println!("Options:\n");
    println!("-c | --no-create: Do not create file if it does not exist.");
    println!("-a | --access: Change the access time only.");
    println!("-m | --modification: Change the modification time only.");
    println!("-d | --date <iso8601>: Use ISO 8601 timestamp (e.g 2017-01-02T23:50:00+00:00).");
    println!("-u | --unix <timestamp>: Use Unix timestamp (e.g. 1483402603).");
    println!("-r | --reference <ref_file>: Use reference file's time instead of current time.");
    println!("-l | --log: Log used Unix timestamp to console for -d or -u option.");
    exit(0);
}

fn parse_unit(unit: &str) -> i32 {
    let n = unit.parse::<i32>().ok();
    let unit = match n {
        Some(unit) => unit as i32,
        None => 0 as i32,
    };
    unit
}

fn get_unix_time(timestamp: &str, unix: bool) -> i64 {
    if !unix {
        // Parse something like 2016-10-12T14:00:34...
        let p = LPattern::new("%dddd-%dd-%ddT%dd:%dd:%dd");
        let caps = p.apply_to(timestamp);
        if !p.is_match(caps.clone(), timestamp) {
            return -1 as i64;
        }
        let date = time::Tm {
            tm_sec: parse_unit(&caps[5][0..2]),
            tm_min: parse_unit(&caps[4][0..2]),
            tm_hour: parse_unit(&caps[3][0..2]),
            tm_mday: parse_unit(&caps[2][0..2]),
            tm_mon: parse_unit(&caps[1][0..2]) - 1,
            tm_year: parse_unit(&caps[0][0..4]) - 1900,
            tm_wday: 0,
            tm_yday: 0,
            tm_isdst: 0,
            tm_utcoff: 0,
            tm_nsec: 0,
        };
        return date.to_utc().to_timespec().sec as i64;
    } 
    let n = timestamp.parse::<i64>().ok();
    let timestamp = match n {
        Some(timestamp) => timestamp as i64,
        None => -1 as i64,
    };
    timestamp
}

fn touch(program: &str, file: &str, create: bool, access: bool, modify: bool,
rfile: String, timestamp: String, unix: bool, log: bool) {
    let touchf = "__touch_file__";
    let p = Path::new(file);
    if !p.exists() && create {
        let _ = File::create(file);
    }
    if p.exists() {
        let _ = File::create(touchf);
        let _ = File::open(file);
        let dt = fs::metadata(touchf).unwrap();
        let df = fs::metadata(file).unwrap();
        let mut tatime = FileTime::from_last_access_time(&dt);
        let mut tmtime = FileTime::from_last_modification_time(&dt);
        if !rfile.is_empty() {
            let rp = Path::new(&rfile);
            if rp.exists() {
                let drf = fs::metadata(&rfile).unwrap();
                tatime = FileTime::from_last_access_time(&drf);
                tmtime = FileTime::from_last_modification_time(&drf);
            } else {
                let _ = fs::remove_file(touchf);
                display_error(program, "cannot access reference file");
            }
        }
        else if !timestamp.is_empty() {
            let utime = get_unix_time(&timestamp, unix);
            if log {
                println!("Using timestamp: {}", utime);
            }
            if utime == -1 {
                let _ = fs::remove_file(touchf);
                if !unix {
                    display_error(&program, "not a valid ISO 8601 timestamp");
                } else {
                    display_error(&program, "not a valid Unix timestamp");
                }
            }
            tatime = FileTime::from_seconds_since_1970(utime as u64, 0);
            tmtime = tatime;
        }
        let fatime = FileTime::from_last_access_time(&df);
        let fmtime = FileTime::from_last_modification_time(&df);
        if access && !modify {
            let _ = filetime::set_file_times(file, tatime, fmtime);
        } else if modify && !access {
            let _ = filetime::set_file_times(file, fatime, tmtime);
        } else {
            let _ = filetime::set_file_times(file, tatime, tmtime);
        }
        let _ = fs::remove_file(touchf);
    }
    exit(0);
}

fn main() {
    let cli = CliOptions::new("touch");
    let program = cli.get_program();

    let file = cli.next_argument(0);
    let mut rfile = String::new();
    let mut date = String::new();
    let mut create = true;
    let mut access = true;
    let mut modify = true;
    let mut unix = false;
    let mut log = false;

    if cli.get_num() == 2 {
        if file.trim() == "-h" || file.trim() == "--help" {
            display_help(&program);
        } else if file.trim() == "-v" || file.trim() == "--version" {
            display_version();
        }
    } else if cli.get_num() > 2 {
        for (i, a) in cli.get_args().iter().enumerate() {
            match a.trim() {
                "-c" | "--no-create" => create = false,
                "-a" | "--access" => modify = false,
                "-m" | "--modification" => access = false,
                "-d" | "--date" => date = cli.next_argument(i),
                "-u" | "--unix" => {
                    date = cli.next_argument(i);
                    unix = true;
                },
                "-r" | "--reference" => rfile = cli.next_argument(i),
                "-l" | "--log" => log = true,
                _ => continue
            }
        }
    } else {
        display_error(&program, "missing file operand");
    }
    touch(&program, &file, create, access, modify, rfile, date, unix, log);
}
