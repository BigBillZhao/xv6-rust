# Instruction on how to run the code
We have test on Mac & Ubuntu(Windows WSL)
### Install Rust
```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Install qemu
we have the qemu version 5.2.0, 
please refer to https://pdos.csail.mit.edu/6.828/2020/tools.html for more detail.

## Windows comments
We suggest you to use wsl2, how to start wsl2 can refer to this site:
https://docs.microsoft.com/zh-cn/windows/wsl/install-win10
after setting the wsl2 , please follow the instructions in the Ubuntu section.

## Run
first clone this repo, then cd this folder, and run the following command. 
```
cargo run
```
the first time you run the code, you need to wait a moment, since the rust are pulling corresponding packages and loading a nightly-build rust.