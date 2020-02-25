# Ops Tool

A simple cli to manage versions of ops tools. It works by downloading the binaries to `$HOME/bin/{name}-versions` and then linking `$HOME/bin/{name}` to the downloaded binary. This means that you need to have `$HOME/bin` in your `PATH`.

## Supported Tools
* [Kops](https://github.com/kubernetes/kops)
* [Kubectl](https://github.com/kubernetes/kubernetes)
* [Terraform](https://github.com/hashicorp/terraform)

## TODO
- [ ] Support other platforms/architectures
- [ ] List available versions
- [ ] Add other build targets with static linking
