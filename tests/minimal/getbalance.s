.section .text
.global _start

_start:
    addi x10, x0, 0x100  # load address to write result
    addi x5, x0, 0xa3    # load host getbalance syscall number
    ecall
    addi x10, x0, 3      # load fd (3)
    addi x11, x0, 0x100  # load writebuf address (result address)
    addi x12, x0, 32     # load nbytes (32)
    addi x5, x0, 2       # load write syscall number
    ecall
