mod cli;
mod tool;

use cli::{Cli, Command};
use tool::Tool;

use std::{
    fs::Permissions,
    io::{Read, Write},
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Result};
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::{blocking::ClientBuilder, redirect::Policy, StatusCode};
use structopt::StructOpt;

fn main() -> Result<()> {
    let cli = Cli::from_args();

    match cli.command {
        Command::Status { tool } => status(tool),
        Command::Use {
            force,
            tool,
            version,
        } => switch_or_download(tool, &version, force),
    }
}

fn status(_tool: Option<String>) -> Result<()> {
    let tools = vec![
        Tool::Kops.name(),
        Tool::Kubectl.name(),
        Tool::Terraform.name(),
    ];

    for tool in tools {
        let link_path = get_link_path(tool);
        if link_path.exists() {
            // look up bin and extract version
            let link_meta = std::fs::read_link(link_path)?;
            let filename = link_meta.file_name().unwrap().to_str().unwrap();
            let parts = filename.rsplitn(2, "-").collect::<Vec<&str>>();
            println!("{}: {}", parts[1], parts[0]);
        } else {
            println!("{}: not setup", tool);
        }
    }
    Ok(())
}

fn switch_or_download(tool: Tool, version: &str, force: bool) -> Result<()> {
    let versions_dir = get_versions_dir(tool.name());
    if !versions_dir.exists() {
        std::fs::create_dir_all(versions_dir)?;
    }

    let bin_path = get_bin_path(tool.name(), version);
    if !bin_path.exists() || force {
        if bin_path.exists() {
            println!("Redownloading {} {}", tool.name(), version);
        } else {
            println!("{} {} not found locally", tool.name(), version);
        }
        let (mut res, total_size, content_type) =
            download(tool.url(version, get_os(), get_arch()).as_ref())?;

        let (res, total_size): (Box<dyn Read>, u64) = match content_type.as_str() {
            "application/zip" => {
                let zipfile = zip::read::read_zipfile_from_stream(&mut res)?.unwrap();
                let total_size = zipfile.size();
                (Box::new(zipfile), total_size)
            }
            _ => (Box::new(res), total_size),
        };

        write_to_file(res, get_bin_path(tool.name(), version), total_size)?;
    } else {
        println!("Binary already downloaded. To redownload it, pass the --force flag or manually remove the file");
    }

    std::fs::set_permissions(&bin_path, Permissions::from_mode(0o700))?;

    link_binary(bin_path, get_link_path(tool.name()))?;

    println!("Done!");
    Ok(())
}

fn get_os() -> &'static str {
    match std::env::consts::OS {
        "macos" => "darwin",
        os => os,
    }
}

fn get_arch() -> &'static str {
    match std::env::consts::ARCH {
        "x86" => "i386",
        "x86_64" => "amd64",
        arch => arch,
    }
}

fn get_bin_path(tool: &str, version: &str) -> PathBuf {
    let mut path = PathBuf::new();
    path.push(dirs::executable_dir().unwrap());
    path.push(format!("{}-versions", tool));
    path.push(format!("{}-{}", tool, version));
    path
}

fn get_link_path(tool: &str) -> PathBuf {
    let mut path = PathBuf::new();
    path.push(dirs::executable_dir().unwrap());
    path.push(tool);
    path
}

fn get_versions_dir(tool: &str) -> PathBuf {
    let mut path = PathBuf::new();
    path.push(dirs::executable_dir().unwrap());
    path.push(format!("{}-versions", tool));
    path
}

fn link_binary<P: AsRef<Path>>(bin_path: P, link_path: P) -> Result<()> {
    println!("Updating symlink");
    if link_path.as_ref().exists() {
        std::fs::remove_file(link_path.as_ref())?;
    }

    Ok(std::os::unix::fs::symlink(bin_path, link_path)?)
}

fn download(url: &str) -> Result<(impl std::io::Read, u64, String)> {
    let policy = Policy::custom(|attempt| attempt.follow());
    let client = ClientBuilder::new().redirect(policy).build()?;

    let res = client.get(url).send().map_err(|e| anyhow!(e))?;

    match res.status() {
        StatusCode::OK => {}
        StatusCode::NOT_FOUND => return Err(anyhow!("Tool version not available for download")),
        s => {
            return Err(anyhow!(
                "Recieved a non-200 response when downloading the tool: {}",
                s
            ))
        }
    }

    let total_size: u64 = match res.headers().get("Content-Length") {
        Some(length) => length.to_str()?.parse().map_err(|_| {
            anyhow!(
                "Invalid Content-Length header: {}",
                length.to_str().unwrap()
            )
        })?,
        None => return Err(anyhow!("No Content-Length header")),
    };

    let content_type = match res.headers().get("Content-Type") {
        Some(value) => value.to_str()?.to_string(),
        None => return Err(anyhow!("No Content-Type header")),
    };

    Ok((res, total_size, content_type))
}

fn write_to_file(mut src: impl Read, path: impl AsRef<Path>, total_size: u64) -> Result<()> {
    let pb = ProgressBar::new(total_size);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("Downloading [{bar:40.cyan/blue} {percent}%] {bytes_per_sec}")
            .progress_chars("=>-"),
    );

    let mut dest = std::fs::File::create(path.as_ref())?;

    let mut buf = [0u8; 8096];

    loop {
        let n = src.read(buf.as_mut())?;
        if n == 0 {
            break;
        }

        dest.write_all(&buf[..n])?;
        pb.inc(n as u64);
    }

    pb.finish_with_message("Done");

    Ok(())
}
