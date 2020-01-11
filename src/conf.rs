extern crate ini;

use std::str::FromStr;
use self::ini::Ini;
use crate::Backlight;

pub fn parse_conf(path: &str) -> (Backlight, Backlight) {
    let ini = Ini::load_from_file(path)
        .expect(format!("Couldn't load configuration file ({})", path).as_str());

    let input = parse_backlight(&ini, "input");
    let output = parse_backlight(&ini, "output");

    (input, output)
}

fn parse_backlight(ini: &Ini, section_name: &str) -> Backlight {
    let section = ini.section(Some(section_name))
        .expect(format!("Configuration file doesn't contain section {}", section_name).as_str());

    let min = i32::from_str(section.get("min")
        .expect(format!("Backlight must have attribute min in section {}", section_name).as_str())
    ).expect(format!("Invalid number format in attribute min in section {}", section_name).as_str());

    let max = i32::from_str(section.get("max")
        .expect(format!("Backlight must have attribute max in section {}", section_name).as_str())
    ).expect(format!("Invalid number format in attribute max in section {}", section_name).as_str());

    let path = match section.get("path") {
        Some(string) => {
            Some(string.to_owned())
        }
        None => None
    };

    let command = match section.get("update_command") {
        Some(string) => {
            Some(string.to_owned())
        }
        None => None
    };

    Backlight {
        min,
        max,
        path,
        update_command: command
    }
}