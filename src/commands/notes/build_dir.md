
1. **Install essential packages and tools:**
   ```sh
   sudo apt-get install build-essential cmake clang libclang-dev libssl-dev pkg-config protobuf-compiler
   ```

2. **Link missing headers if required:**
   ```sh
   sudo ln -s /usr/include/x86_64-linux-gnu/bits/libc-header-start.h /usr/include/bits/libc-header-start.h
   ```

3. **Set environment variables for the compiler and flags:**
   ```sh
   export CC=/usr/bin/gcc
   export CXX=/usr/bin/g++
   export CFLAGS="-I/usr/include -I/usr/include/x86_64-linux-gnu"
   export CXXFLAGS="-I/usr/include -I/usr/include/x86_64-linux-gnu"
   export CPPFLAGS="-I/usr/include -I/usr/include/x86_64-linux-gnu"
   export CMAKE_TOOLCHAIN_FILE=/home/dylankawalec/dev/sgx-test/Toolchain-Fortanix.cmake
   export CMAKE_GENERATOR="Unix Makefiles"
   export BINDGEN_EXTRA_CLANG_ARGS="--target=x86_64-fortanix-unknown-sgx"
   ```

4. **Create and configure the toolchain file for CMake:**
   ```sh
   echo "set(CMAKE_SYSTEM_NAME Linux)" > Toolchain-Fortanix.cmake
   echo "set(CMAKE_SYSTEM_PROCESSOR x86_64)" >> Toolchain-Fortanix.cmake
   ```

5. **Install and configure Rust toolchain:**
   ```sh
   rustup update
   rustup default nightly
   rustup target add x86_64-fortanix-unknown-sgx --toolchain nightly
   ```

6. **Install Intel SGX packages and services:**
   ```sh
   echo "deb https://download.01.org/intel-sgx/sgx_repo/ubuntu $(lsb_release -cs) main" | sudo tee -a /etc/apt/sources.list.d/intel-sgx.list >/dev/null
   curl -sSL "https://download.01.org/intel-sgx/sgx_repo/ubuntu/intel-sgx-deb.key" | sudo -E apt-key add -
   sudo apt-get update
   sudo apt-get install sgx-aesm-service libsgx-aesm-launch-plugin
   sudo apt-get install pkg-config libssl-dev protobuf-compiler
   ```

7. **Detect SGX capabilities:**
   ```sh
   sgx-detect
   ```

8. **Build the project:**
   ```sh
   cargo clean
   cargo build --target=x86_64-fortanix-unknown-sgx
   ```

9. **Set up and run CMake for additional configuration if needed:**
   ```sh
   mkdir build_directory
   cd build_directory
   cmake -DCMAKE_TOOLCHAIN_FILE=/home/dylankawalec/dev/sgx-test/Toolchain-Fortanix.cmake ..
   cmake --build .
   ```

10. **Testing and running the SGX project:**
    ```sh
    cargo test --target=x86_64-fortanix-unknown-sgx
    ftxsgx-elf2sgxs target/x86_64-fortanix-unknown-sgx/debug/sgx-test --heap-size 0x20000 --stack-size 0x20000 --threads 10 --debug
    sgxs-sign --key my_key.pem target/x86_64-fortanix-unknown-sgx/debug/sgx-test.sgxs sgx-test.sig --xfrm 7/0 --isvprodid 0 --isvsvn 0 --debug
    ftxsgx-runner target/x86_64-fortanix-unknown-sgx/debug/sgx-test.sgxs
    ```

