Testing IPFS Git Commit Hash / Link for DEFE {Check src/fetcher.rs}

IPFS installation and set up to test IPFS_GIT_FETCHER Binary in a controlled environment.
You will want to make sure the front end has a repo_init.json file made available first : 
```
REPO_URL="https://github.com/osmosis/osmosis-front-end.git"
COMMIT_HASH=$(git rev-parse HEAD)
echo "{\"repo_url\": \"$REPO_URL\", \"commit_hash\": \"$COMMIT_HASH\"}" > repo_info.json
```
1. Install IPFS on Ubuntu 22.04 LTS

Download and Extract IPFS:
```sh
wget https://dist.ipfs.io/go-ipfs/v0.28.0/go-ipfs_v0.28.0_linux-amd64.tar.gz
tar -xvzf go-ipfs_v0.28.0_linux-amd64.tar.gz
cd go-ipfs
```
Install IPFS as root:
```sh
sudo bash install.sh
```
Initialize IPFS in $HOME:
```sh
ipfs init --profile server
```
Start the IPFS Daemon in $HOME:
```sh
ipfs daemon
```

Then you’d want to pin the CID in a separate command line on the hosted server, as such ‘$ ipfs pin add <CID>’, where then you will want to announce the content to the network ‘$ ipfs routing provide <CID>’
Automated update For each dfe server, they should have a kernel running the ipfs gateway continuously to ensure the IPFS’s liveliness to the file that was pinned. Other IPFS nodes 
Add → `pin_and_announce.sh`
$ chmod +x pin_and_announce.sh
```sh
#!/bin/bash

# IPFS CID to be pinned
CID="<your-own-CID>"

# Check if IPFS is installed
if ! command -v ipfs &> /dev/null
then
    echo "IPFS could not be found, please install it first."
    exit
fi

# Start IPFS daemon if not already running
if ! pgrep -x "ipfs" > /dev/null
then
    echo "Starting IPFS daemon..."
    ipfs daemon &
    sleep 5
fi

# Pin the CID
echo "Pinning CID: $CID"
ipfs pin add $CID

# Announce the CID to the network
echo "Announcing CID to the network..."
ipfs routing provide $CID

echo "CID $CID pinned and announced successfully."
```


2. Manual Upload `repo_info.json` to IPFS

Ensure Your IPFS Daemon is Running:
```sh
ipfs daemon
```

In a New Terminal Window, Navigate to Your Repository Directory:
```sh
cd ~/ipfs-dfe
```

3. Add the chosen front end’s repo’s `repo_info.json` to ipfs-dfe/ directory and then add it to IPFS to Get the CID:
```sh
ipfs add -Q repo_info.json
```
Note the returned CID, e.g., `QmefZuBxoZef7kthX42XUijzBoZYC5iM1pEGeNQU8mvBWY`.

Pin the Content Locally:
```sh
ipfs pin add <CID>
```

Announce the Content to the Network
```sh
ipfs routing provide <CID>
```
3. Verify IPFS CID Accessibility

Check Accessibility via Curl:
```sh
curl https://ipfs.io/ipfs/<CID>
```

Check Accessibility via Web Browser:
Open a web browser and go to:
     ```
     https://ipfs.io/ipfs/<CID>
     ```


Implement and Run the Rust Program to test the binary operation as a program

Create a New Rust Project for the IPFS Fetching Program on a completely separate machine:
```sh
cd ~
cargo new ipfs_git_fetcher
cd ipfs_git_fetcher
```


Build and Run the Rust Program:
   ```sh
   cargo build
  cargo run
   ```

Enter the CID When Prompted:
   ```sh
   Enter the IPFS CID: <CID>
   ```

In the end you should see your binary run the IPFS fetcher to return the full library from github, which could then be used by your Javascript runtime binary to run or build this to serve locally or via the web server. The goal is to pull this code, and run it in the enclave using Deno. All operations will be turned into Binary operations that will be more useful an a Cargo Installation that will behave as the dfe engine for the server running a specific DFE. 
