**Look at Projects -> Feature releases for infos on new features**


**right now the code is being completelly refactored so there wont be commits because it will be taking longer when its finished commits here and hay os will be continued in active developing here**


welcome to hay os this project is in active development if you want to compile it on linux you need following packages

for being able to compile boot.asm:
sudo apt install build-essential nasm xorriso grub-pc-bin grub-common mtool

Quemu:
sudo apt install qemu-system-x86 qemu-utils

needed rust tools:
rustup default nightly
rustup component add rust-src

to compile run: make run