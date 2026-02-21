use anyhow::{Context, Result};
use chrono::Timelike;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::config::{Config, ScheduleEntry, config_dir};
use crate::device;
use crate::image;

#[derive(Debug, Default, Serialize, Deserialize)]
struct State {
    last_pushed: HashMap<String, String>,
}

fn state_path() -> PathBuf {
    config_dir().join("state.json")
}

fn hold_path() -> PathBuf {
    config_dir().join("hold")
}

pub fn is_held() -> bool {
    hold_path().exists()
}

pub fn set_hold() -> Result<()> {
    std::fs::create_dir_all(config_dir()).context("Failed to create config directory")?;
    std::fs::write(hold_path(), "").context("Failed to create hold file")?;
    eprintln!("Schedule held. Run `pixoo-ctl resume` to re-enable.");
    Ok(())
}

pub fn clear_hold() -> Result<()> {
    let path = hold_path();
    if path.exists() {
        std::fs::remove_file(&path).context("Failed to remove hold file")?;
        eprintln!("Schedule resumed.");
    } else {
        eprintln!("Schedule is not held.");
    }
    Ok(())
}

fn load_state() -> State {
    std::fs::read_to_string(state_path())
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn save_state(state: &State) -> Result<()> {
    std::fs::create_dir_all(config_dir()).context("Failed to create config directory")?;
    let json = serde_json::to_string_pretty(state)?;
    std::fs::write(state_path(), json).context("Failed to write state file")?;
    Ok(())
}

fn current_day_abbrev() -> String {
    let now = chrono::Local::now();
    now.format("%a").to_string()
}

fn current_time_minutes() -> u32 {
    let now = chrono::Local::now();
    now.hour() * 60 + now.minute()
}

fn parse_time(s: &str) -> Option<u32> {
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() != 2 {
        return None;
    }
    let h: u32 = parts[0].parse().ok()?;
    let m: u32 = parts[1].parse().ok()?;
    if h > 23 || m > 59 {
        return None;
    }
    Some(h * 60 + m)
}

fn find_active_entry<'a>(
    schedule: &'a [ScheduleEntry],
    day: &str,
    now_minutes: u32,
) -> Option<&'a ScheduleEntry> {
    let mut best: Option<(&ScheduleEntry, u32)> = None;

    for entry in schedule {
        let day_matches = entry.days.iter().any(|d| d.eq_ignore_ascii_case(day));
        if !day_matches {
            continue;
        }

        let Some(entry_minutes) = parse_time(&entry.time) else {
            continue;
        };

        if entry_minutes > now_minutes {
            continue;
        }

        match best {
            None => best = Some((entry, entry_minutes)),
            Some((_, best_minutes)) if entry_minutes > best_minutes => {
                best = Some((entry, entry_minutes));
            }
            _ => {}
        }
    }

    best.map(|(entry, _)| entry)
}

pub async fn update(config: &Config) -> Result<()> {
    if is_held() {
        eprintln!("Schedule is held. Run `,pixoo-ctl resume` to re-enable.");
        return Ok(());
    }

    if config.schedule.is_empty() {
        eprintln!("No schedule entries in config");
        return Ok(());
    }

    let day = current_day_abbrev();
    let now = current_time_minutes();

    let Some(entry) = find_active_entry(&config.schedule, &day, now) else {
        eprintln!(
            "No matching schedule entry for {} at {:02}:{:02}",
            day,
            now / 60,
            now % 60
        );
        return Ok(());
    };

    eprintln!(
        "Matched schedule entry: {} {}",
        entry.days.join(","),
        entry.time
    );

    let mut state = load_state();
    let mut changed = false;

    let device_images: Vec<(&str, Option<&String>)> = vec![
        ("stage-left", entry.stage_left.as_ref()),
        ("stage-right", entry.stage_right.as_ref()),
    ];

    for (device_name, image_path) in device_images {
        let Some(path) = image_path else { continue };
        let Some(dev) = config.devices.get(device_name) else {
            eprintln!("Device '{}' not found in config, skipping", device_name);
            continue;
        };

        let last = state.last_pushed.get(device_name);
        if last.is_some_and(|l| l == path) {
            eprintln!("{}: already showing {}", device_name, path);
            continue;
        }

        eprintln!("{}: pushing {}", device_name, path);
        let rgb_data = image::load_and_prepare(std::path::Path::new(path))?;
        device::push_image(&dev.ip, &rgb_data).await?;

        state
            .last_pushed
            .insert(device_name.to_string(), path.clone());
        changed = true;
        eprintln!("{}: done", device_name);
    }

    if changed {
        save_state(&state)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(days: &[&str], time: &str) -> ScheduleEntry {
        ScheduleEntry {
            days: days.iter().map(|d| d.to_string()).collect(),
            time: time.to_string(),
            stage_left: None,
            stage_right: None,
        }
    }

    #[test]
    fn parse_time_validates_ranges() {
        assert_eq!(parse_time("09:30"), Some(570));
        assert_eq!(parse_time("23:59"), Some(1439));
        assert_eq!(parse_time("24:00"), None);
        assert_eq!(parse_time("12:60"), None);
        assert_eq!(parse_time("nope"), None);
    }

    #[test]
    fn finds_latest_matching_entry_for_day() {
        let schedule = vec![
            entry(&["Mon", "Tue"], "08:00"),
            entry(&["Mon"], "09:00"),
            entry(&["Mon"], "12:00"),
        ];

        let selected = find_active_entry(&schedule, "Mon", 10 * 60).unwrap();
        assert_eq!(selected.time, "09:00");
    }

    #[test]
    fn ignores_entries_after_current_time() {
        let schedule = vec![entry(&["Tue"], "14:00")];
        assert!(find_active_entry(&schedule, "Tue", 13 * 60 + 59).is_none());
    }
}
