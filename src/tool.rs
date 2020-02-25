use semver::{Version, VersionReq};
use std::io::Write;

use zip::read::read_zipfile_from_stream;

pub mod error {
    use std::{
        error::Error as StdError,
        fmt::{Display, Formatter, Result as FmtResult},
        io::Error as IoError,
    };
    use zip::result::ZipError;

    #[derive(Debug)]
    pub enum Error {
        Http(reqwest::StatusCode),
        Io(IoError),
        Reqwest(reqwest::Error),
        UnsupportedTool(String),
        Zip(ZipError),
    }

    impl Display for Error {
        fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
            match self {
                Self::Http(s) => write!(f, "got a non-2xx response code: {}", s),
                Self::Io(e) => write!(f, "io error: {}", e),
                Self::Reqwest(e) => write!(f, "request error: {}", e),
                Self::UnsupportedTool(t) => write!(f, "unsupported tool: {}", t),
                Self::Zip(e) => write!(f, "zip: {}", e),
            }
        }
    }

    impl StdError for Error {}

    impl From<IoError> for Error {
        fn from(e: IoError) -> Self {
            Self::Io(e)
        }
    }

    impl From<reqwest::Error> for Error {
        fn from(e: reqwest::Error) -> Self {
            Self::Reqwest(e)
        }
    }

    impl From<ZipError> for Error {
        fn from(e: ZipError) -> Self {
            Self::Zip(e)
        }
    }
}

pub type ToolResult<T> = Result<T, error::Error>;

pub trait Named {
    fn name(&self) -> &'static str;

    fn name_versioned(&self, v: &Version) -> String {
        format!("{}-{}", self.name(), v)
    }
}

pub trait Download {
    fn download(&self, v: &Version, dest: &mut dyn Write) -> ToolResult<()>;
}

pub enum Tool {
    Kops,
    Kubectl,
    Terraform,
}

impl Tool {
    pub fn from(name: &str) -> ToolResult<Self> {
        Ok(match name {
            "kops" => Self::Kops,
            "kubectl" => Self::Kubectl,
            "terraform" => Self::Terraform,
            _ => return Err(error::Error::UnsupportedTool(name.to_string())),
        })
    }
}

impl Named for Tool {
    fn name(&self) -> &'static str {
        match self {
            Self::Kops => "kops",
            Self::Kubectl => "kubectl",
            Self::Terraform => "terraform",
        }
    }
}

impl Download for Tool {
    fn download(&self, v: &Version, dest: &mut dyn Write) -> ToolResult<()> {
        match self {
            Self::Kops => download_kops(v, dest),
            Self::Kubectl => download_kubectl(v, dest),
            Self::Terraform => download_terraform(v, dest),
        }
    }
}

fn download_kops(v: &Version, dest: &mut dyn Write) -> ToolResult<()> {
    let url = format!(
        "https://github.com/kubernetes/kops/releases/download/{}{}/kops-linux-amd64",
        // Kops changed it's version naming after 1.15.0, thanks for that
        if VersionReq::parse("> 1.15.0").unwrap().matches(v) {
            "v"
        } else {
            ""
        },
        v,
    );

    let mut resp = download_file(url.as_ref())?;

    std::io::copy(&mut resp, dest)?;
    Ok(())
}

fn download_kubectl(v: &Version, dest: &mut dyn Write) -> ToolResult<()> {
    let url = format!(
        "https://storage.googleapis.com/kubernetes-release/release/v{}/bin/linux/amd64/kubectl",
        v,
    );

    let mut resp = download_file(url.as_ref())?;

    std::io::copy(&mut resp, dest)?;
    Ok(())
}

fn download_terraform(v: &Version, dest: &mut dyn Write) -> ToolResult<()> {
    let url = format!(
        "https://releases.hashicorp.com/terraform/{version}/terraform_{version}_linux_amd64.zip",
        version = v,
    );

    let mut resp = download_file(url.as_ref())?;

    let a = read_zipfile_from_stream(&mut resp)?;
    let mut f = a.unwrap();

    std::io::copy(&mut f, dest)?;
    Ok(())
}

fn download_file(src: &str) -> ToolResult<reqwest::blocking::Response> {
    let resp = reqwest::blocking::get(src)?;
    if !resp.status().is_success() {
        return Err(error::Error::Http(resp.status()));
    };

    Ok(resp)
}
