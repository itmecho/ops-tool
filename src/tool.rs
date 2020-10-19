use anyhow::{anyhow, Result};
use url::Url;

pub const TOOL_NAMES: &[&str] = &[
    Tool::Kops.name(),
    Tool::Kubectl.name(),
    Tool::Terraform.name(),
];

#[derive(Debug)]
pub enum Tool {
    Kops,
    Kubectl,
    Terraform,
}

impl std::str::FromStr for Tool {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "kops" => Ok(Self::Kops),
            "kubectl" => Ok(Self::Kubectl),
            "terraform" => Ok(Self::Terraform),
            _ => Err(anyhow!("Invalid tool {}", s)),
        }
    }
}

impl Tool {
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Kops => "kops",
            Self::Kubectl => "kubectl",
            Self::Terraform => "terraform",
        }
    }

    pub fn url(&self, version: &str, os: &str, arch: &str) -> Url {
        match self {
            Self::Terraform => Url::parse(&format!(
                    "https://releases.hashicorp.com/terraform/{version}/terraform_{version}_{os}_{arch}.zip",
                    version = version,
                    os = os,
                    arch = arch
                )).unwrap(),
            Self::Kops => Url::parse(&format!(
                    "https://github.com/kubernetes/kops/releases/download/v{version}/kops-{os}-{arch}",
                    version = version,
                    os = os,
                    arch = arch
                )).unwrap(),
            Self::Kubectl => Url::parse(&format!(
                    "https://storage.googleapis.com/kubernetes-release/release/v{version}/bin/{os}/{arch}/kubectl",
                    version = version,
                    os = os,
                    arch = arch
                )).unwrap()
        }
    }
}
