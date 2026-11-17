//! The shared secret between the mascot app and Claude Code's hooks,
//! stored at `~/.claude-mascot/token` (mode 0600).

use std::path::{Path, PathBuf};

pub fn mascot_dir(home: &Path) -> PathBuf {
    home.join(".claude-mascot")
}

pub fn load_or_create_token(home: &Path) -> std::io::Result<String> {
    use std::io::Read;
    use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};

    let dir = mascot_dir(home);
    let path = dir.join("token");
    if let Ok(existing) = std::fs::read_to_string(&path) {
        let t = existing.trim();
        if t.len() == 32 && t.chars().all(|c| c.is_ascii_hexdigit()) {
            return Ok(t.to_owned());
        }
    }
    std::fs::create_dir_all(&dir)?;
    std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o700))?;

    let mut bytes = [0u8; 16];
    std::fs::File::open("/dev/urandom")?.read_exact(&mut bytes)?;
    let token: String = bytes.iter().map(|b| format!("{b:02x}")).collect();

    let mut f = std::fs::OpenOptions::new().write(true).create(true).truncate(true).mode(0o600).open(&path)?;
    std::io::Write::write_all(&mut f, token.as_bytes())?;
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))?;
    Ok(token)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    fn temp_home(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("puppet-token-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn creates_a_32_hex_char_private_token() {
        let home = temp_home("create");
        let t = load_or_create_token(&home).unwrap();
        assert_eq!(t.len(), 32);
        assert!(t.chars().all(|c| c.is_ascii_hexdigit()));
        let mode = std::fs::metadata(mascot_dir(&home).join("token")).unwrap().permissions().mode();
        assert_eq!(mode & 0o777, 0o600);
    }

    #[test]
    fn reuses_the_existing_token() {
        let home = temp_home("reuse");
        let a = load_or_create_token(&home).unwrap();
        assert_eq!(load_or_create_token(&home).unwrap(), a);
    }

    #[test]
    fn different_homes_get_different_tokens() {
        assert_ne!(load_or_create_token(&temp_home("a")).unwrap(), load_or_create_token(&temp_home("b")).unwrap());
    }
}
