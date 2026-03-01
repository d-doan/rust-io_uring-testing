### environment
- ran on windows 11 desktop

## Downloads
- [VMWare Workstation 17.6.4 hypervisor](https://www.techspot.com/downloads/downloadnow/189/?evp=f14a48a23bc560f5fbe81b8d83387b41&file=241)
- [ubuntu-22.04.5-desktop-amd64.iso](https://releases.ubuntu.com/jammy/ubuntu-22.04.5-desktop-amd64.iso)
## VM Setup
- install downloads above
- open VMWare Workstation
	- create a new virtual machine
	- select ubuntu iso that we downloaded earlier
	- allocate resources
		- right click on VM on left sidebar and adjust # of processors and memory
		- personally gave it 8GB ram and 6 processors to compile kernel faster
		- also made max size 80GB i don't think it'll actually use this much but KASAN kernel is significantly larger than base ubuntu

## Building KASAN-Enabled Kernel
- KASAN isn't enabled by default prob bc of performance overhead, need to build it ourself
- run following commands to set-up Linux tooling and build kernel
```bash

# install random dependencies
sudo apt update && sudo apt upgrade -y
sudo apt install build-essential libncurses-dev bison flex libssl-dev libelf-dev dwarves -y
sudo apt install git


# create dir and download kernel
mkdir ~/kernel-research && cd ~/kernel-research
git clone --depth 1 --branch v6.1 https://github.com/torvalds/linux.git 
cd linux

# configure for KASAN
cp /boot/config-$(uname -r) .config
make menuconfig

# enable KASAN in menuconfig by navigating through these pages
# Kernel hacking -> Memory Debugging -> KASAN: runtime memory debugger 
# check the KASAN box by pressing Y
# save config and exit menuconfig

# config stuff
scripts/config --set-str SYSTEM_TRUSTED_KEYS ""
scripts/config --set-str SYSTEM_REVOCATION_KEYS ""
scripts/config --set-val FRAME_WARN 2048

# compiling and installing kernel (takes a long time)
make -j$(nproc)
sudo make modules_install
sudo make install
sudo reboot


# HOLD SHIFT UPON REBOOT then go into advanced settings and select 6.1.0
# then run upon reboot to check if it worked
grep CONFIG_KASAN /boot/config-$(uname -r)
'''
# should output
CONFIG_KASAN_SHADOW_OFFSET=0xdffffc0000000000
CONFIG_KASAN=y
CONFIG_KASAN_GENERIC=y
CONFIG_KASAN_OUTLINE=y
# CONFIG_KASAN_INLINE is not set
CONFIG_KASAN_STACK=y
# CONFIG_KASAN_VMALLOC is not set
# CONFIG_KASAN_MODULE_TEST is not set
'''

```
- We can check if KASAN is installed and running through the `kasan_c_test` program I put in the repo, instructions for running it are in that README

## Installing Rust
```bash
sudo apt install curl
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup default nightly
```

## SSH setup from computer
```bash
# in vm
sudo apt install openssh-server -y
sudo systemctl enable --now ssh

# note down inet 192.168.x.x
# we'll use this for config later
ip addr

# open vscode on pc and install remote - ssh extension (microsoft)
# open command pallet Connect to Host... -> Add New SSH Host
# enter ssh your_username@192.168.xxx.xxx
# command pallet -> connect to ... 
# enter vm password and should be good to go
```