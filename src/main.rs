mod error;
mod tool;

use std::{fs::Permissions, os::unix::fs::PermissionsExt, path::PathBuf};

use clap::{App, Arg, SubCommand};
use semver::Version;

type OpsResult<T> = Result<T, error::Error>;

fn main() -> OpsResult<()> {
    let matches = App::new("Ops Tool")
        .version(clap::crate_version!())
        .author(clap::crate_authors!())
        .about("CLI for managing operational tools")
        .subcommand(
            SubCommand::with_name("use")
                .about("Use a specific version of a tool")
                .arg(
                    Arg::with_name("force")
                        .short("f")
                        .long("force")
                        .help("Redownload the binary if it already exists"),
                )
                .arg(
                    Arg::with_name("TOOL")
                        .help("The tool to manage")
                        .index(1)
                        .required(true)
                        .possible_values(&["kops", "kubectl", "terraform"]),
                )
                .arg(
                    Arg::with_name("VERSION")
                        .help("The version to use")
                        .index(2)
                        .required(true),
                ),
        )
        .subcommand(SubCommand::with_name("status").about("Print the current version of each tool"))
        .get_matches();

    if let Some(ref matches) = matches.subcommand_matches("use") {
        let tool = tool::Tool::from(matches.value_of("TOOL").unwrap())?;
        let version = Version::parse(matches.value_of("VERSION").unwrap())?;
        let force = matches.is_present("force");

        use_tool(tool, &version, force)?
    }

    if let Some(_) = matches.subcommand_matches("status") {
        status()?
    }

    Ok(())
}

fn use_tool<T: tool::Named + tool::Download>(t: T, v: &Version, force: bool) -> OpsResult<()> {
    let bin_path = bin_path(t.name(), v)?;
    let mut bin_dir = bin_path.clone();
    bin_dir.pop();
    if !bin_dir.exists() {
        std::fs::create_dir_all(bin_dir)?;
    }

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

    let link_path = link_path(t.name())?;
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

fn status() -> OpsResult<()> {
    print_version(tool::Tool::Kops)?;
    print_version(tool::Tool::Kubectl)?;
    print_version(tool::Tool::Terraform)?;
    Ok(())
}

fn print_version(t: impl tool::Named) -> OpsResult<()> {
    let p = link_path(t.name())?;
    let p = std::fs::read_link(p)?;
    println!("{}", p.file_name().unwrap().to_str().unwrap());
    Ok(())
}

fn get_home_dir() -> OpsResult<PathBuf> {
    match dirs::home_dir() {
        Some(d) => Ok(d),
        None => return Err(error::Error::HomeDir),
    }
}

fn link_path(name: &str) -> OpsResult<PathBuf> {
    let mut path = get_home_dir()?;
    path.push("bin");
    path.push(name);
    Ok(path)
}

fn bin_path(name: &str, v: &Version) -> OpsResult<PathBuf> {
    let mut path = get_home_dir()?;
    path.push("bin");
    path.push(format!("{}-versions", name));
    path.push(format!("{}-{}", name, v));
    Ok(path)
}
