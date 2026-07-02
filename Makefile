# Compiler and tools
ASM = nasm
CARGO = cargo
LD = ld
QEMU = qemu-system-x86_64
VOLUME_MKR = grub-mkrescue

# Flags
ASMFLAGS = -f elf64
LDFLAGS = -n -T linker.ld

# Target and paths
TARGET = x86_64-unknown-none
RUST_LIB = target/$(TARGET)/release/libkernel.a

# Directory definitions
BUILD_DIR = build
ISO_DIR = $(BUILD_DIR)/isofiles
GRUB_DIR = $(ISO_DIR)/boot/grub

# Output files
KERNEL_BIN = $(BUILD_DIR)/hay_os.bin
ISO_OUT = $(BUILD_DIR)/hay_os.iso

.PHONY: all run clean FORCE

all: $(ISO_OUT)

# Rule to create the final ISO file
$(ISO_OUT): $(KERNEL_BIN) $(BUILD_DIR)/grub.cfg
	@echo "Creating ISO structure..."
	mkdir -p $(GRUB_DIR)
	cp $(KERNEL_BIN) $(ISO_DIR)/boot/hay_os.bin
	cp $(BUILD_DIR)/grub.cfg $(GRUB_DIR)/grub.cfg
	@echo "Generating ISO image..."
	$(VOLUME_MKR) -o $(ISO_OUT) $(ISO_DIR)

# Rule to link the kernel binary into the build directory
$(KERNEL_BIN): $(BUILD_DIR)/boot.o $(RUST_LIB)
	@echo "Linking kernel..."
	$(LD) $(LDFLAGS) -o $(KERNEL_BIN) $(BUILD_DIR)/boot.o $(RUST_LIB)

# Rule to assemble bootloader code into the build directory
$(BUILD_DIR)/boot.o: boot/boot.asm
	@echo "Assembling bootloader..."
	mkdir -p $(BUILD_DIR)
	$(ASM) $(ASMFLAGS) boot/boot.asm -o $(BUILD_DIR)/boot.o

$(BUILD_DIR)/grub.cfg:
	@echo "Generating grub.cfg..."
	mkdir -p $(BUILD_DIR)
	@echo 'set timeout=0' > $(BUILD_DIR)/grub.cfg
	@echo 'set default=0' >> $(BUILD_DIR)/grub.cfg
	@echo '' >> $(BUILD_DIR)/grub.cfg
	@echo 'menuentry "Hay OS" {' >> $(BUILD_DIR)/grub.cfg
	@echo '    multiboot /boot/hay_os.bin' >> $(BUILD_DIR)/grub.cfg  
	@echo '    boot' >> $(BUILD_DIR)/grub.cfg
	@echo '}' >> $(BUILD_DIR)/grub.cfg

# Rule to compile the Rust library
$(RUST_LIB): FORCE
	@echo "Building Rust library..."
	$(CARGO) build --release --target $(TARGET)

FORCE:

# Rule to run the OS image using QEMU
run: all
	$(QEMU) -cdrom $(ISO_OUT)

# Rule to clean up all build artifacts
clean:
	@echo "Cleaning up..."
	rm -rf $(BUILD_DIR)
	$(CARGO) clean