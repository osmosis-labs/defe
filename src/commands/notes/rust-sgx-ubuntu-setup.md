# Server Setup for Rust-SGX Development with Docker on Ubuntu 22.04 LTS

## Using Intel SGX (R) EDP Driver and Fortanix EDP

### Building Confidential Servers for Decentralized Front-End Applications

## Introduction

This guide will walk you through configuring an Ubuntu 22.04 LTS server for secure application development using Rust-SGX (Fortanix EDP), Docker containers, and additional security-enhancing tools like Trust-DNS, Rustls, and Certbot. The purpose of this guide is to get our Decentralized Front End Demonstration Server for secure application development ready to be used to deploy and fully decentralize the Osmosis Front End and Light Client software, as we believe this method of development achieves true App-&-Chain decentralization. This guide will help you get the server ready to begin your TEE development journey using Open Source Software and the bare bone requirements for getting involved in the development process. We welcome feedback! Thank you for your contribution and sanity checks!

## Design for Achieving DFE (Decentralized Front-End)

Replace the suggested tools with your preferred toolkits, or follow this guide to use the recommended systems to get your validator ready for DFE deployment and serviceability.

## Prerequisites

- Operating System: Ubuntu 22.04 LTS (Trusted Launch - Gen 2 x64)
- SGX-Compatible Hardware: A server with Intel SGX enabled in BIOS and supported by the Linux kernel
- User Credentials:  Replace placeholders (username, `dfekey`, `00.00.000.000`) with your actual credentials
- SSH Key Pair: A strong SSH key pair (e.g., generated with `ssh-keygen` or using the server on deployment)

## 1. Secure SSH Access

Place SSH Key:

```bash
mv ~/Desktop/dfekey.pem ~/.ssh/
chmod 600 ~/.ssh/dfekey.pem
```

SSH Agent:

```bash
eval "$(ssh-agent -s)"
ssh-add ~/.ssh/dfekey.pem
```

SSH Config (`nano ~/.ssh/config`):

```
Host your_server_alias
  Hostname 00.00.000.000
  User username
  IdentityFile ~/.ssh/dfekey.pem
```

User Password (Optional, but recommended):

```bash
sudo passwd username
```

## 2. Connect to Server

- SSH: `ssh your_server_alias`
- VS Code: Use the "Remote - SSH" extension

## 3. Verify SGX Supports

```bash
lscpu | grep sgx
sudo dmesg | grep sgx -- verbose
```

If both commands show output related to SGX, your hardware is ready.

## 4. Prepare System Environment

Download the list of dependencies from here:
`apt-requirements.txt`

Then install them all at once using the below command:

```bash
sudo apt update && sudo apt upgrade -y && xargs -a apt-requirements.txt sudo apt install -y
```

Then check for the sgx:

```bash
sudo journalctl -k | grep -i sgx
```

## 5. Install Rust & Fortanix SGX Toolkit

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
rustup upgrade
rustup default nightly
rustup update nightly
rustup target add x86_64-fortanix-unknown-sgx --toolchain nightly
echo "deb https://download.01.org/intel-sgx/sgx_repo/ubuntu $(lsb_release -cs) main" | sudo tee -a /etc/apt/sources.list.d/intel-sgx.list >/dev/null
curl -sSL "https://download.01.org/intel-sgx/sgx_repo/ubuntu/intel-sgx-deb.key" | sudo -E apt-key add -
```

If `curl -sSL "https://download.01.org/intel-sgx/sgx_repo/ubuntu/intel-sgx-deb.key" | sudo -E apt-key add - ` does not work, then try this:

```bash
sudo apt-get update
sudo apt-get install intel-sgx-dkms
cat /var/lib/dkms/intel-sgx/2.11/build/make.log
sudo apt install build-essential dkms -y
sudo apt-get remove --purge intel-sgx-dkms -y
sudo apt-get autoremove -y
sudo apt-get clean
sudo apt-get update
sudo apt-get install build-essential dkms linux-headers-$(uname -r) -y
```

Then try this again:

```bash
echo "deb https://download.01.org/intel-sgx/sgx_repo/ubuntu $(lsb_release -cs) main" | sudo tee -a /etc/apt/sources.list.d/intel-sgx.list >/dev/null
curl -sSL "https://download.01.org/intel-sgx/sgx_repo/ubuntu/intel-sgx-deb.key" | sudo -E apt-key add -
```

Once that works, we have to make sure now that the EDP Fortanix tools are installed next:

```bash
sudo apt-get update
sudo apt-get install sgx-aesm-service libsgx-aesm-launch-plugin
sudo apt-get install pkg-config libssl-dev protobuf-compiler
cargo install fortanix-sgx-tools sgxs-tools
rustup target add x86_64-fortanix-unknown-sgx
mkdir -p ~/.cargo
echo '[target.x86_64-fortanix-unknown-sgx]
runner = "ftxsgx-runner-cargo"' >> ~/.cargo/config
ls /dev/isgx
systemctl status aesmd
sudo systemctl start aesmd
sudo apt-get install linux-headers-$(uname -r)
dpkg-query -s linux-headers-$(uname -r)
```

Double Check everything is installed and reboot if you want to at this point:

```bash
sudo apt-get install build-essential ocaml ocamlbuild automake autoconf libtool wget python-is-python3 libssl-dev git cmake perl

sudo apt-get install libssl-dev libcurl4-openssl-dev protobuf-compiler libprotobuf-dev debhelper cmake reprepro unzip pkgconf libboost-dev libboost-system-dev libboost-thread-dev lsb-release libsystemd0

sudo reboot
```

Connect again and continue to check your systems health

## 6. Install Node.js, Docker, and Security Tools

```bash
curl -fsSL https://deb.nodesource.com/setup_lts.x | sudo -E bash -
sudo apt-get install -y nodejs

# Docker (follow official instructions for the latest version)
curl -fsSL https://get.docker.com -o get-docker.sh
sudo sh get-docker.sh

sudo usermod -aG docker $USER

cargo install trust-dns rustls 
sudo apt-get install -y certbot
curl https://bun.sh/install | bash
```

## 7. Re-verify SGX

```bash
sgx-detect --verbose
```

## 8. Get intel's SGX (R) driver installed and ready for Fortanix Execution

```bash 
sudo apt-get update
git clone https://github.com/intel/linux-sgx-driver.git
cd linux-sgx-driver
git checkout sgx_driver_2.14
# checkout the latest version
```

Edit the C/C++ JSON Config in VSCode →

```json
{
  "configurations": [
    {
      "name": "Linux",
      "includePath": [
        "${workspaceFolder}/**",
        "/usr/include",
        "/usr/local/include",
        "/usr/src/linux-headers-6.5.0-1021-azure/include",
        "/usr/src/linux-headers-6.5.0-1021-azure/arch/x86/include",
        "/usr/src/linux-headers-6.5.0-1021-azure/arch/x86/include/generated"
      ],
      "defines": [],
      "compilerPath": "/usr/bin/gcc",
      "cStandard": "c17",
      "cppStandard": "c++17",
      "intelliSenseMode": "linux-gcc-x64",
      "browse": {
        "path": [
          "${workspaceFolder}/**",
          "/usr/include",
          "/usr/local/include",
          "/usr/src/linux-headers-6.5.0-1021-azure/include",
          "/usr/src/linux-headers-6.5.0-1021-azure/arch/x86/include",
          "/usr/src/linux-headers-6.5.0-1021-azure/arch/x86/include/generated"
        ],
        "limitSymbolsToIncludedHeaders": true,
        "databaseFilename": ""
      }
    }
  ],
  "version": 4
}
```

Modify the sgx_main.c file to use an indirect method for modifying the vm_flags field. Here's the corrected code snippet:

```c
static int sgx_mmap(struct file *file, struct vm_area_struct *vma)
{
    vma->vm_ops = &sgx_vm_ops;
    unsigned long new_flags = vma->vm_flags | VM_PFNMAP | VM_DONTEXPAND | VM_DONTDUMP | VM_IO | VM_DONTCOPY;
    *(unsigned long *)&vma->vm_flags = new_flags;
    return 0;
}
```

Then build the driver in the /linux-sgx-driver root directory:

```bash 
make
```

Set the driver to the module that was built in the HOME directory:

```bash
sudo mkdir -p "/lib/modules/"`uname -r`"/kernel/drivers/intel/sgx"
sudo cp isgx.ko "/lib/modules/"`uname -r`"/kernel/drivers/intel/sgx"
sudo sh -c "cat /etc/modules | grep -Fxq isgx || echo isgx >> /etc/modules"
sudo /sbin/depmod
sudo /sbin/modprobe isgx
```

Then make a new directory /dev, and cargo new in that directory your project, like `cargo new sgx-test`

```bash
cd ~/dev/sgx-test
mkdir -p .cargo
```

Whereby the .cargo config file has the following if it doesn't already:

```toml
[build]
target = "x86_64-fortanix-unknown-sgx"
```

You want to be sure the project has the configured Cargo.toml ready:

```toml
[package]
name = "sgx-test"
version = "0.1.0"
edition = "2021"

[dependencies]
sgx = "0.6.1"

[package.metadata.fortanix-sgx]
# stack size (in bytes) for each thread, the default stack size is 0x20000.
stack-size=0x20000
# heap size (in bytes), the default heap size is 0x2000000.
heap-size=0x2000000
# the default number of threads is equal to the number of available CPUs of
# the current system.
# Gotcha: Don't forget to count the main thread when counting number of
# threads.
threads=1
# SSA frame size (in pages) for each thread, the default SSA frame size is 1.
# You normally don't need to change the SSA frame size.
ssaframesize=1
# whether to enable EDP debugging features in the enclave, debugging is
# enabled by default.
debug=true
```

Build and Run, sign and verify the project:

```bash
echo '[build] target = "x86_64-fortanix-unknown-sgx"' > .cargo/config
cargo build --target x86_64-fortanix-unknown-sgx
cargo run --target x86_64-fortanix-unknown-sgx
cargo test --target x86_64-fortanix-unknown-sgx
ftxsgx-elf2sgxs --help
ftxsgx-elf2sgxs target/x86_64-fortanix-unknown-sgx/debug/sgx-test --heap-size 0x20000 --stack-size 0x20000 --threads 10 --debug
openssl genrsa -3 3072 > my_key.pem
sgxs-sign --key my_key.pem target/x86_64-fortanix-unknown-sgx/debug/sgx-test.sgxs sgx-test.sig --xfrm 7/0 --isvprodid 0 --isvsvn 0 --debug
ftxsgx-runner target/x86_64-fortanix-unknown-sgx/debug/sgx-test.sgxs
```

## Additional Notes

You will want to reboot the system now that the server is set up, with Secure Boot and vTPM enabled. These must be turned off when setting up the server to get it to load correctly. However, once the setup is complete, reboot with Secure Boot and vTPM active, and everything will work.

## Security Measures

To check the security of the server's setup, you may find it problematic that two drivers are installed but only one out-of-tree driver (iSGX) is active. This is fine because the in-kernel driver remains dormant. With Secure Boot enabled, it cannot tamper with the enclave's runtime execution, as the permissions of the system would require a reinstallation using a separate SDK for the enclave to work. Since the target SDK cannot work with the in-kernel SDK, you can be assured that the enclave's execution is safe because the in-kernel driver is not active. You may remove the in-kernel driver if you want, but it may be useful for future installations and upgrades. Go to `Configuration` on your Cloud Portal to find your Secure Boot and vTMP.

…Running these commands will help assess the security status of the sgx fortanix server, the integrity of the set up and the presence of any potential vulnerabilities.

### Secure Boot Status

```bash
sudo mokutil --sb-state
sudo dmesg | grep -i secureboot
```

### SGX Driver and Enclave Status

```bash
sudo dmesg | grep sgx
lsmod | grep sgx
ls /dev/sgx*
```

### Verifying the Integrity of Key System Files

Using `AIDE` (Advanced Intrusion Detection Environment):

The `aideinit` process can take a considerable amount of time depending on the size of the existing filesystem, the number of files, and the resources available on the server. It builds a database of all system files to detect any unauthorized changes later. Typically, it can range from a few minutes to over an hour for large systems.

```bash
sudo apt-get install aide
sudo aideinit
sudo cp /var/lib/aide/aide.db.new /var/lib/aide/aide.db
sudo aide --check
```

### Checking Kernel Module Signing

List loaded modules and check for unsigned ones:

```bash
sudo dmesg | grep 'module verification failed'
```

### Checking the Firewall Status and Rules

But this is flexible to check since our system will likely require the firewall's to be turned off because for a DFE we'll have a POW on the client's side to reduce DDOS.

```bash
sudo ufw status
sudo iptables -L
```

### Check Installed Packages for Known Vulnerabilities

Using `lynis`:

```bash
sudo apt-get install lynis
sudo lynis audit system
```

### Verify Running Services and Open Ports

```bash
sudo netstat -tuln
sudo systemctl list-units --type=service --state=running
```

### Check for Rootkits

Using `chkrootkit`:

```bash
sudo apt-get install chkrootkit
sudo chkrootkit
```

### Verify System Logs for Unusual Activity

```bash
sudo journalctl -xe
```

## Secure Boot Permissions may cause issues

The BIOS on the server when using a confidential or trusted launch server come with a secure launch each time the server reboots. You can have these set to on or off, but when they are on, it's hard to get the sgx to run right. Try turning them off in the configuration to test for now, and then configure them later after you have the sgx working right.

## Authors
- Dylan Kawalec 
- Dev Ojha

## Contributors
- Moe Muhhouk
- Bart Hofkin