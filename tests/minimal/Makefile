# Ensure that the RISC-V toolchain is installed and in the PATH
# See https://github.com/riscv-collab/riscv-gnu-toolchain
AS=riscv32-unknown-elf-as
LD=riscv32-unknown-elf-ld
OBJCOPY=riscv32-unknown-elf-objcopy

TARGET=getbalance
ASM_SOURCE=$(TARGET).s
OBJECT=$(TARGET).o
ELF=$(TARGET).elf
BIN=$(TARGET).bin
TEMP_BIN=$(TARGET)_temp.bin

.PHONY: all clean

all: $(BIN)

$(OBJECT): $(ASM_SOURCE)
	$(AS) -march=rv32em -o $@ $(ASM_SOURCE)

$(ELF): $(OBJECT)
	$(LD) -o $@ $(OBJECT)

# Strip the ELF file to only include the .text section
# Also add the Athena magic number to the beginning of the file
$(BIN): $(ELF)
	$(OBJCOPY) -O binary --only-section=.text $< $(TEMP_BIN)
	echo '7f415448' | xxd -r -p > $@
	cat $(TEMP_BIN) >> $@
	rm $(TEMP_BIN)

clean:
	rm -f $(OBJECT) $(ELF) $(BIN) $(TEMP_BIN)
