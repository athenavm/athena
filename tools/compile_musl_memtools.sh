# NOTE: ensure that https://github.com/riscv-collab/riscv-gnu-toolchain is installed and working.

# Download and extract musl source code (if not already done)
wget https://musl.libc.org/releases/musl-1.2.5.tar.gz
tar -xvf musl-1.2.5.tar.gz
cd musl-1.2.5

# Set up cross-compilation environment
export CC=riscv64-unknown-elf-gcc
export CFLAGS="-march=rv32em -mabi=ilp32e -O3 -nostdlib -fno-builtin -funroll-loops"

# Configure musl for cross-compilation, disabling shared libraries
./configure --target=riscv32 --host=riscv64-unknown-elf --disable-shared

# Navigate to the source directory
cd src/string

# Compile memset.c to assembly
$CC $CFLAGS -S memset.c -o memset.s

# Compile memcpy.c to assembly
$CC $CFLAGS -S memcpy.c -o memcpy.s

# Remove the unsupported attribute from the generated assembly files
sed -i 's/.attribute arch, "rv32e1p9_m2p0"/.attribute arch, "rv32em"/' memset.s
sed -i 's/.attribute arch, "rv32e1p9_m2p0"/.attribute arch, "rv32em"/' memcpy.s

# Modify labels
sed -i 's/\.\(L[0-9]\)/\.LBBmemset\1/g' memset.s
sed -i 's/\.\(L[0-9]\)/\.LBBmemcpy\1/g' memcpy.s
