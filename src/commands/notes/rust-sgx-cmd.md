Example CMD for operting rust-sgx

```sh
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