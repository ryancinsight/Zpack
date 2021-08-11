extern crate clap;
extern crate dirs;
extern crate mimalloc;
extern crate once_cell;
extern crate reqwest;
extern crate tar;
extern crate zstd;
extern crate static_vcruntime;
use mimalloc::MiMalloc;
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[cfg(target_family = "windows")]
extern crate winapi;

use clap::{App, AppSettings, Arg};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::{env, fs, io, process};


use zstd::stream::Encoder;

const APP_NAME: &str = env!("CARGO_PKG_NAME");
const AUTHOR: &str = env!("CARGO_PKG_AUTHORS");
const VERSION: &str = env!("CARGO_PKG_VERSION");

const RUNNER_MAGIC: &[u8] = b"tVQhhsFFlGGD3oWV4lEPST8I8FEPP54IM0q7daes4E1y3p2U2wlJRYmWmjPYfkhZ0PlT14Ls0j8fdDkoj33f2BlRJavLj3mWGibJsGt5uLAtrCDtvxikZ8UX2mQDCrgE\0";

static RUNNER_BY_ARCH: Lazy<HashMap<&'static str, &'static [u8]>> = Lazy::new(|| {
    let mut m = HashMap::new();

    const RUNNER: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/../../../Zrun.exe"));
    m.insert("application", RUNNER);

    m
});

/// Print a message to stderr and exit with error code 1
macro_rules! bail {
    () => (process::exit(1));
    ($($arg:tt)*) => ({
        eprint!("{}\n", format_args!($($arg)*));
        process::exit(1);
    })
}

#[inline(always)]
fn patch_runner(arch: &str, exec_name: &str) -> io::Result<Vec<u8>> {
    // Read runner executable in memory
    let runner_contents = RUNNER_BY_ARCH.get(arch).unwrap();
    let mut buf = runner_contents.to_vec();

    // Set the correct target executable name into the local magic buffer
    let magic_len = RUNNER_MAGIC.len();
    let mut new_magic = vec![0; magic_len];
    new_magic[..exec_name.len()].clone_from_slice(exec_name.as_bytes());

    // Find the magic buffer offset inside the runner executable
    let mut offs_opt = None;
    for (i, chunk) in buf.windows(magic_len).enumerate() {
        if chunk == RUNNER_MAGIC {
            offs_opt = Some(i);
            break;
        }
    }

    if offs_opt.is_none() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "no magic found inside runner",
        ));
    }

    // Replace the magic with the new one that points to the target executable
    let offs = offs_opt.unwrap();
    buf[offs..offs + magic_len].clone_from_slice(&new_magic);

    Ok(buf)
}

#[cfg(target_family = "unix")]
#[inline(always)]
fn create_app_file(out: &Path) -> io::Result<File> {
    use std::os::unix::fs::OpenOptionsExt;

    fs::OpenOptions::new()
        .create(true)
        .write(true)
        .mode(0o755)
        .open(out)
}

#[cfg(target_family = "windows")]
#[inline(always)]
fn create_app_file(out: &Path) -> io::Result<File> {
    use std::os::windows::fs::OpenOptionsExt;
    use winapi::um::winbase::SECURITY_IDENTIFICATION;
    fs::OpenOptions::new()
        .create(true)
        .write(true)
        .security_qos_flags(SECURITY_IDENTIFICATION)
        .open(out)
}

#[inline(always)]
fn create_app(dir: &Path,runner_buf: &[u8], out: &Path) -> io::Result<()> {
    let mut outf = create_app_file(out)?;
    let gz = Encoder::new(Vec::new(), 6)?;
    let mut tar = tar::Builder::new(gz);
    tar.follow_symlinks(true);  
    tar.append_dir_all(".", dir)?;
    let encoder_data: Encoder<Vec<u8>> = tar.into_inner()?;
    let compress_vec: &[u8] = &encoder_data.finish()?;
    outf.write_all(runner_buf)?;
    outf.write_all(compress_vec)?;
    Ok(())
}

#[inline(always)]
fn main() -> Result<(), Box<dyn Error>> {
    let args = App::new(APP_NAME)
        .settings(&[AppSettings::ArgRequiredElseHelp, AppSettings::ColoredHelp])
        .version(VERSION)
        .author(AUTHOR)
        .about("Create self-contained single binary application")
        .arg(
            Arg::with_name("input_dir")
                .short("i")
                .long("input_dir")
                .value_name("input_dir")
                .help("Sets the input directory containing the application and dependencies")
                .display_order(1)
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("exec")
                .short("e")
                .long("exec")
                .value_name("exec")
                .help("Sets the application executable file name")
                .display_order(2)
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("output")
                .short("o")
                .long("output")
                .value_name("output")
                .help("Sets the resulting self-contained application file name")
                .display_order(3)
                .takes_value(true)
                .required(true),
        )
        .get_matches();

    let arch = "application";

    let input_dir = Path::new(args.value_of("input_dir").unwrap());
    if fs::metadata(input_dir).is_err() {
        bail!("Cannot access specified input directory {:?}", input_dir);
    }

    let exec_name = args.value_of("exec").unwrap();
    if exec_name.len() >= RUNNER_MAGIC.len() {
        bail!("Executable name is too long, please consider using a shorter name");
    }

    let exec_path = Path::new(input_dir).join(exec_name);
    match fs::metadata(&exec_path) {
        Err(_) => {
            bail!("Cannot find file {:?}", exec_path);
        }
        Ok(metadata) => {
            if !metadata.is_file() {
                bail!("{:?} isn't a file", exec_path);
            }
        }
    }

    let runner_buf = patch_runner(arch, exec_name)?;

    println!("Compressing input directory {:?}...", input_dir);
    //let tmp_dir = TempDir::new(APP_NAME)?;
    //let tgz_path = tmp_dir.path().join("input.tgz");
    //create_tgz(input_dir, &tgz_path)?;

    let exec_name = Path::new(args.value_of("output").unwrap());
    println!(
        "Creating self-contained application binary {:?}...",
        exec_name
    );
    create_app(input_dir,&runner_buf, exec_name)?;

    println!("All done");
    Ok(())
}