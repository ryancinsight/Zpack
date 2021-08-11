use std::env;
#[cfg(target_family = "unix")]
use std::fs;
#[cfg(target_family = "unix")]
use std::fs::Permissions;
use std::io;
#[cfg(target_family = "unix")]
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;
use std::process::Stdio;

#[inline(always)]
pub fn execute(target: &Path) -> io::Result<i32> {
    let args: Vec<String> = env::args().skip(1).collect();

    do_execute(target, &args)
}

#[cfg(target_family = "unix")]
#[inline(always)]
fn ensure_executable(target: &Path) {
    let perms = Permissions::from_mode(0o770);
    fs::set_permissions(target, perms).unwrap();
}

#[cfg(target_family = "unix")]
#[inline(always)]
fn do_execute(target: &Path, args: &[String]) -> io::Result<i32> {
    ensure_executable(target);

    Ok(Command::new(target)
        .args(args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?
        .wait()?
        .code()
        .unwrap_or(1))
}

#[cfg(target_family = "windows")]
#[inline(always)]
fn is_script(target: &Path) -> bool {
    const SCRIPT_EXTENSIONS: &[&str] = &["bat", "cmd"];
    SCRIPT_EXTENSIONS.contains(
        &target
            .extension()
            .unwrap_or_default()
            .to_string_lossy()
            .to_lowercase()
            .as_str(),
    )
}

#[cfg(target_family = "windows")]
#[inline(always)]
fn do_execute(target: &Path, args: &[String]) -> io::Result<i32> {
    if is_script(target) {
        let cmd_args = vec![
            "/c".to_string(),
            target.as_os_str().to_str().unwrap().to_string(),
            args.to_owned().iter().map(String::as_str).collect(),
        ];

        Ok(Command::new("cmd")
            .args(cmd_args)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()?
            .wait()?
            .code()
            .unwrap_or(1))
    } else {
        Ok(Command::new(target)
            .args(args)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()?
            .wait()?
            .code()
            .unwrap_or(1))
    }
}
