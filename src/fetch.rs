use git2::build::{CheckoutBuilder, RepoBuilder};
use git2::{Error, FetchOptions, Progress, RemoteCallbacks, Repository};
use std::fs::create_dir_all;
use std::io::{self, Write};
use std::path::{Path, PathBuf};


const URI: &str = "https://github.com/CISecurity/OVALRepo.git";
const DEST: &str = "data/oval/";


pub struct Args {
    arg_url: String,
    arg_path: String,
}

struct State {
    progress: Option<Progress<'static>>,
    total: usize,
    current: usize,
    path: Option<PathBuf>,
    newline: bool,
}


// https://github.com/CISecurity/OVALRepo/blob/master/README.getting.repo.md#clone-for-read-only-access
pub async fn fetch() -> Result<(), git2::Error> {
    let p = Path::new(DEST);
    match create_dir_all(p) {
        Ok(s) => s,
        Err(e) => {
            println!("failed to create part of file path for oval data {}", DEST);
            let fs = git2::ErrorClass::Filesystem;
            let code = git2::ErrorCode::NotFound;
            let err = git2::Error::new(code, fs, e.to_string());
        }
    };



    match Repository::clone(URI, DEST) {
        Ok(repo) => repo,
        Err(e) => panic!("failed to clone: {}", e),
    };
    Ok(())
}
