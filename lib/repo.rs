extern crate git2;
extern crate rpf;

use std::io;
use std::io::prelude::*;
use std::str;
use std::cell::RefCell;

use git2::*;
use error::BuildError;
use rpf::*;

struct RepoState {
    progress: Option<Progress<'static>>,
    total: usize,
    current: usize,
    newline: bool,
}

fn repo_clone_print(state: &mut RepoState) {
    let stats = state.progress.as_ref().unwrap();
    let received_obj = (100 * stats.received_objects()) / stats.total_objects();
    let index_obj = (100 * stats.indexed_objects()) / stats.total_objects();

    let checkout_cnt = if state.total > 0 {
        (100 * state.current) / state.total
    } else {
        0
    };

    let kb = stats.received_objects() / 1024;
    let total_kb = stats.total_objects() / 1024;
    if stats.received_objects() == stats.total_objects() && state.newline {
        if !state.newline {
            println!("");
            state.newline = true;
        }
    } else {
        print!("Progress: {:3}% ({:4} kb / {} kb, {:5}/{:5}) / Index {:3}% ({:5}/{:5}) Check \
                {:3}% ({:4}/{:4})\r",
               received_obj,
               kb,
               total_kb,
               stats.received_objects(),
               stats.total_objects(),
               index_obj,
               stats.indexed_objects(),
               stats.total_objects(),
               checkout_cnt,
               state.current,
               state.total);
    }
    if let Err(_) = io::stdout().flush() {
        ()
    }
}

pub fn is_repo(path: &str) -> bool {
    if let Ok(_) = Repository::open(path) {
        return true;
    } else {
        return false
    }
}

pub fn clone_repo(url: &str) -> Result<(), BuildError> {
    if "src".as_path().exists() && is_repo("src") {
        println!("Updating {} repository...", url.bold());
        return Ok(try!(fetch_origin("src")));
    }
    let state = RefCell::new(RepoState {
        progress: None,
        total: 0,
        current: 0,
        newline: false,
    });

    let mut callbacks = RemoteCallbacks::new();
    callbacks.transfer_progress(|stats| {
        let mut state = state.borrow_mut();
        state.progress = Some(stats.to_owned());
        repo_clone_print(&mut *state);
        true
    });

    let mut checkbuilder = build::CheckoutBuilder::new();
    checkbuilder.progress(|_, cur, total| {
        let mut state = state.borrow_mut();
        state.current = cur;
        state.total = total;
        repo_clone_print(&mut *state);
    });

    let mut fetchopts = FetchOptions::new();
    fetchopts.remote_callbacks(callbacks);
    try!(build::RepoBuilder::new()
             .fetch_options(fetchopts)
             .with_checkout(checkbuilder)
             .clone(url, "src".as_path()));
    println!("");
    Ok(())
}

fn fetch_origin(path: &str) -> Result<(), BuildError> {
    let repo = try!(Repository::open(path));
    let mut callbacks = RemoteCallbacks::new();
    let mut remote = try!(repo.find_remote("origin").or_else(|_| repo.remote_anonymous("origin")));
    callbacks.sideband_progress(|data| {
        print!("Remote: {}", str::from_utf8(data).unwrap());
        if let Err(_) = io::stdout().flush() {
            return true;
        }
        true
    });

    callbacks.update_tips(|refname, a, b| {
        if a.is_zero() {
            println!("[New]\t{:20} {}", b, refname);
        } else {
            println!("[Update]\t{:10}..{:10} {}", a, b, refname);
        }
        true
    });

    callbacks.transfer_progress(|stats| {
        if stats.received_objects() == stats.total_objects() {
            print!("Resolving deltas {}/{}\r",
                   stats.indexed_deltas(),
                   stats.total_deltas());
        } else if stats.total_objects() > 0 {
            print!("Received {}/{} objects ({}) in {} bytes\r",
                   stats.received_objects(),
                   stats.total_objects(),
                   stats.indexed_objects(),
                   stats.received_bytes());
        }
        if let Err(_) = io::stdout().flush() {
            return true;
        }
        true
    });
    try!(remote.connect(Direction::Fetch));
    let mut fetchopts = FetchOptions::new();
    fetchopts.remote_callbacks(callbacks);
    try!(remote.download(&[], Some(&mut fetchopts)));
    remote.disconnect();
    Ok(())
}
