//! Helper for system utilities like users and sessions
use std::collections::HashMap;
use std::fs::read;
use std::io::Result as IOResult;
use std::str::from_utf8;

use glob::glob;
use log::{debug, info, warn};
use pwd::Passwd;
use regex::Regex;

use crate::constants::{LOGIN_FILE, SESSION_DIRS};

/// Default minimum UID for `useradd` (a/c to my system)
const DEFAULT_UID_MIN: u32 = 1000;
/// Default maximum UID for `useradd` (a/c to my system)
const DEFAULT_UID_MAX: u32 = 60000;

type UserMap = HashMap<String, String>;
type SessionMap = HashMap<String, Vec<String>>;

/// Stores info of all regular users and sessions
pub struct SysUtil {
    /// Maps a user's full name to their system username
    users: UserMap,
    /// Maps a session's full name to its command
    sessions: SessionMap,
}

impl SysUtil {
    pub fn new() -> IOResult<Self> {
        Ok(Self {
            users: Self::init_users()?,
            sessions: Self::init_sessions()?,
        })
    }

    /// Get the list of regular users
    ///
    /// These are defined as a list of users with UID between UID_MIN and UID_MAX.
    fn init_users() -> IOResult<UserMap> {
        let contents = read(LOGIN_FILE)?;
        let text = from_utf8(contents.as_slice())
            .unwrap_or_else(|err| panic!("Login file '{}' is not UTF-8: {}", LOGIN_FILE, err));

        // UID_MIN/MAX are limits to a UID for a regular user i.e. a user created with `useradd`.
        // Thus, to find regular users, we filter the list of users with these UID limits.
        let min_uid_regex = Regex::new(r"UID_MIN\s+([0-9]+)").expect("Invalid regex for UID_MIN");
        let max_uid_regex = Regex::new(r"UID_MAX\s+([0-9]+)").expect("Invalid regex for UID_MAX");

        // Get UID_MIN
        let min_uid = if let Some(capture) = min_uid_regex.captures(text) {
            if let Some(num) = capture.get(1) {
                num.as_str()
                    .parse()
                    .expect("UID_MIN regex didn't capture an integer")
            } else {
                warn!("Failed to find UID_MIN in login file: {}", LOGIN_FILE);
                DEFAULT_UID_MIN
            }
        } else {
            warn!("Failed to find UID_MIN in login file: {}", LOGIN_FILE);
            DEFAULT_UID_MIN
        };

        // Get UID_MAX
        let max_uid = if let Some(capture) = max_uid_regex.captures(text) {
            if let Some(num) = capture.get(1) {
                num.as_str()
                    .parse()
                    .expect("UID_MAX regex didn't capture an integer")
            } else {
                warn!("Failed to find UID_MAX in login file: {}", LOGIN_FILE);
                DEFAULT_UID_MAX
            }
        } else {
            warn!("Failed to find UID_MAX in login file: {}", LOGIN_FILE);
            DEFAULT_UID_MAX
        };

        debug!("UID_MIN: {}, UID_MAX: {}", min_uid, max_uid);

        // Iterate over all users in /etc/passwd
        let mut users = HashMap::new();
        for entry in Passwd::iter() {
            if entry.uid > max_uid || entry.uid < min_uid {
                // Non-standard user, eg. git or root
                continue;
            };

            // Use the actual system username if the "full name" is not available
            let full_name = if let Some(gecos) = entry.gecos {
                if gecos.is_empty() {
                    debug!(
                        "Found user '{}' with UID '{}' and empty full name",
                        entry.name, entry.uid
                    );
                    entry.name.clone()
                } else {
                    debug!(
                        "Found user '{}' with UID '{}' and full name: {}",
                        entry.name, entry.uid, gecos
                    );
                    gecos
                }
            } else {
                debug!(
                    "Found user '{}' with UID '{}' and missing full name",
                    entry.name, entry.uid
                );
                entry.name.clone()
            };
            users.insert(full_name, entry.name);
        }
        Ok(users)
    }

    /// Get available X11 and Wayland sessions
    ///
    /// These are defined as either X11 or Wayland session desktop files stored in specific
    /// directories.
    fn init_sessions() -> IOResult<SessionMap> {
        let mut sessions = HashMap::new();

        for sess_dir in SESSION_DIRS.split(':') {
            // Iterate over all '.desktop' files
            for glob_path in glob(&format!("{}/*.desktop", sess_dir))
                .expect("Invalid glob pattern for session desktop files")
            {
                let path = match glob_path {
                    Ok(path) => path,
                    Err(err) => {
                        warn!("Error when globbing: {}", err);
                        continue;
                    }
                };
                info!("Now scanning session file: {}", path.display());

                let contents = read(&path)?;
                let text = from_utf8(contents.as_slice()).unwrap_or_else(|err| {
                    panic!("Session file '{}' is not UTF-8: {}", path.display(), err)
                });

                // The session launch command is specified as: Exec=command arg1 arg2...
                let cmd_regex =
                    Regex::new(r"Exec=(.*)").expect("Invalid regex for session command");
                // The session name is specified as: Name=My Session
                let name_regex = Regex::new(r"Name=(.*)").expect("Invalid regex for session name");

                // Parse the desktop file to get the session command
                let cmd = if let Some(capture) = cmd_regex.captures(text) {
                    if let Some(cmd_str) = capture.get(1) {
                        if let Some(cmd) = shlex::split(cmd_str.as_str()) {
                            cmd
                        } else {
                            warn!(
                                "Couldn't split command of '{}' into arguments: {}",
                                path.display(),
                                cmd_str.as_str()
                            );
                            // Skip the desktop file, since a missing command means that we can't
                            // use it
                            continue;
                        }
                    } else {
                        warn!("Empty command found for session: {}", path.display());
                        // Skip the desktop file, since a missing command means that we can't use
                        // it
                        continue;
                    }
                } else {
                    warn!("No command found for session: {}", path.display());
                    // Skip the desktop file, since a missing command means that we can't use it
                    continue;
                };

                // Get the full name of this session
                let name = if let Some(capture) = name_regex.captures(text) {
                    if let Some(name) = capture.get(1) {
                        debug!(
                            "Found name '{}' for session: {}",
                            name.as_str(),
                            path.display()
                        );
                        Some(name.as_str())
                    } else {
                        debug!("No name found for session: {}", path.display());
                        None
                    }
                } else {
                    debug!("No name found for session: {}", path.display());
                    None
                };

                let name = if let Some(name) = name {
                    name
                } else {
                    // Get the stem of the filename of this desktop file.
                    // This is used as backup, in case the file name doesn't exist.
                    if let Some(stem) = path.file_stem() {
                        stem.to_str().unwrap_or_else(|| {
                            panic!("Non-UTF-8 file stem in session file: {}", path.display())
                        })
                    } else {
                        warn!("No file stem found for session: {}", path.display());
                        // No file stem implies no file name, which shouldn't happen.
                        // Since there's no full name nor file stem, just skip this anomalous
                        // session.
                        continue;
                    }
                };

                sessions.insert(name.to_string(), cmd);
            }
        }

        Ok(sessions)
    }

    /// Get the mapping of a user's full name to their system username
    ///
    /// If the full name is not available, their system username is used.
    pub fn get_users(&self) -> &UserMap {
        &self.users
    }

    /// Get the mapping of a session's full name to its command
    ///
    /// If the full name is not available, the filename stem is used.
    pub fn get_sessions(&self) -> &SessionMap {
        &self.sessions
    }
}
