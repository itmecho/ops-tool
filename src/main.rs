mod error;

use std::fs::Permissions;
use std::os::unix::fs::PermissionsExt;

use dirs;
use reqwest;
use semver::Version;
use structopt::StructOpt;
use zip;

type OpsResult<T> = Result<T, error::Error>;

#[derive(StructOpt, Debug)]
struct Cli {
    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(StructOpt, Debug)]
enum Command {
    Use {
        /// Download binary regardless of if it already exists
        #[structopt(short, long)]
        force: bool,

        #[structopt(subcommand)]
        tool: Tool,
    },
}

#[derive(StructOpt, Debug)]
enum Tool {
    Kops {
        #[structopt(name = "VERSION")]
        version: String,
    },
    Terraform {
        #[structopt(name = "VERSION")]
        version: String,
    },
}

impl Tool {
    pub fn name(&self) -> &str {
        match self {
            Self::Kops { .. } => "kops",
            Self::Terraform { .. } => "terraform",
        }
    }

    pub fn bin_path(&self, v: &Version) -> OpsResult<std::path::PathBuf> {
        let mut path = get_home_dir()?;

        path.push("bin");
        path.push(format!("{}-versions", self.name()));
        path.push(format!("{}-{}", self.name(), v));
        Ok(path)
    }

    pub fn link_path(&self) -> OpsResult<std::path::PathBuf> {
        let mut path = get_home_dir()?;

        path.push("bin");
        path.push(self.name());
        Ok(path)
    }

    pub fn download(&self, v: &Version, out: &mut impl std::io::Write) -> OpsResult<()> {
        match self {
            Self::Kops { .. } => {
                let url = format!(
                    "https://github.com/kubernetes/kops/releases/download/{}{}/kops-linux-amd64",
                    // Kops changed it's version naming after 1.15.0, thanks for that
                    if v > &Version::parse("1.15.0").unwrap() {
                        "v"
                    } else {
                        ""
                    },
                    v,
                );

                let mut resp = download_file(url.as_ref())?;

                std::io::copy(&mut resp, out)?;
            }
            Self::Terraform { .. } => {
                let url =
                    format!(
                    "https://releases.hashicorp.com/terraform/{version}/terraform_{version}_linux_amd64.zip",
                    version = v,
                );

                let mut resp = download_file(url.as_ref())?;

                let a = zip::read::read_zipfile_from_stream(&mut resp)?;
                let mut f = a.unwrap();

                std::io::copy(&mut f, out)?;
            }
        }
        Ok(())
    }
}

fn main() -> Result<(), error::Error> {
    let cli = Cli::from_args();
    let cmd = cli.cmd;

    match cmd {
        Command::Use { force, ref tool } => match tool {
            Tool::Kops { ref version } => use_tool(tool, &Version::parse(version)?, force)?,
            Tool::Terraform { ref version } => use_tool(tool, &Version::parse(version)?, force)?,
        },
    };

    Ok(())
}

fn use_tool(t: &Tool, v: &Version, force: bool) -> OpsResult<()> {
    let bin_path = t.bin_path(v)?;
    if !bin_path.exists() || force {
        println!(
            "Downloading {} version {} to {}",
            t.name(),
            v,
            bin_path.to_string_lossy()
        );
        let mut f = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open(&bin_path)?;

        t.download(v, &mut f)?;
    }

    let link_path = t.link_path()?;
    println!("Setting permissions for {}", bin_path.to_string_lossy());
    std::fs::set_permissions(&bin_path, Permissions::from_mode(0o700))?;
    if let Ok(_) = link_path.symlink_metadata() {
        println!(
            "Removing previous link path {}",
            link_path.to_string_lossy()
        );
        std::fs::remove_file(&link_path)?;
    }

    println!(
        "Linking {} to {}",
        bin_path.to_string_lossy(),
        link_path.to_string_lossy()
    );
    std::os::unix::fs::symlink(bin_path, link_path)?;

    println!("Done!");
    Ok(())
}

fn get_home_dir() -> OpsResult<std::path::PathBuf> {
    match dirs::home_dir() {
        Some(d) => Ok(d),
        None => return Err(error::Error::HomeDir),
    }
}

fn download_file(src: &str) -> OpsResult<reqwest::blocking::Response> {
    let resp = reqwest::blocking::get(src)?;
    if !resp.status().is_success() {
        return Err(error::Error::Http(resp.status()));
    };

    Ok(resp)
}
