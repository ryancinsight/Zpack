extern crate dirs;
extern crate mimalloc;
extern crate winapi;
use mimalloc::MiMalloc;
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

extern crate remove_dir_all;
use remove_dir_all::remove_dir_all;
use std::env;
use std::error::Error;
use std::ffi::*;
use std::fs;
use std::io;
use std::path::*;
use std::process;
extern crate static_vcruntime;

mod executor;
mod extractor;

static TARGET_FILE_NAME_BUF: &[u8] = b"tVQhhsFFlGGD3oWV4lEPST8I8FEPP54IM0q7daes4E1y3p2U2wlJRYmWmjPYfkhZ0PlT14Ls0j8fdDkoj33f2BlRJavLj3mWGibJsGt5uLAtrCDtvxikZ8UX2mQDCrgE\0";

#[inline(always)]
fn target_file_name() -> &'static str {
    let nul_pos = TARGET_FILE_NAME_BUF
        .iter()
        .position(|elem| *elem == b'\0')
        .expect("TARGET_FILE_NAME_BUF has no NUL terminator");

    let slice = &TARGET_FILE_NAME_BUF[..(nul_pos + 1)];
    CStr::from_bytes_with_nul(slice)
        .expect("Can't convert TARGET_FILE_NAME_BUF slice to CStr")
        .to_str()
        .expect("Can't convert TARGET_FILE_NAME_BUF CStr to str")
}

#[inline(always)]
fn extract(exe_path: &Path, cache_path: &Path) -> io::Result<()> {
    remove_dir_all(cache_path).ok();
    println!("Installing new version");
    extractor::extract_to(exe_path, cache_path)?;
    println!("Install Finished, running ...");

    Ok(())
}

#[inline(always)]
fn main() -> Result<(), Box<dyn Error>> {
    let self_path = env::current_exe()?;
    let self_file_name = self_path.file_stem().unwrap();
    let path: String = self_file_name.to_str().unwrap().to_string();
    let mut cache_path = PathBuf::new();
    cache_path.push(&self_path);
    cache_path.push("../../../../Local/");
    //cache_path.push(dirs::data_local_dir().unwrap());
    //cache_path.push("FusWs/Shared/Tools");
    cache_path.push(path);

    let target_file_name = target_file_name();
    let target_path = cache_path.join(target_file_name);

    match fs::metadata(&cache_path) {
        Ok(cache) => {
            if cache.modified()? >= fs::metadata(&self_path)?.modified()? {
                println!("Install is up-to-date, Loading...");
            } else {
                println!("Install is Outdated");
                println!("Uninstalling Original...");
                extract(&self_path, &cache_path)?;
            }
        }
        Err(_) => {
            println!("Install not found");
            extract(&self_path, &cache_path)?;
        }
    }

    let exit_code = executor::execute(&target_path)?;
    if exit_code == 0 {
        process::exit(exit_code);
    } else if exit_code== 1 {
        process::exit(exit_code);
    } else if exit_code== 2 {
        process::exit(exit_code);
    } else {
        println!("Execute Issue {}, reinstalling ...",exit_code);
        extract(&self_path, &cache_path)?;
        let exit_code = executor::execute(&target_path)?;
        process::exit(exit_code);
    }
}
