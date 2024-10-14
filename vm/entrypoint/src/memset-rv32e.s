    .file   "memset.s"
    .option nopic
    .attribute arch, "rv32em"
    .attribute unaligned_access, 0
    .attribute stack_align, 4
    .text
    .align  2
    .globl  memset
    .type   memset, @function

memset:
    # Check if length (a2) is zero, return immediately if so
    beqz    a2, .Ldone

    # Load the byte value to set, ensuring it's within 0-255 range
    andi    a1, a1, 0xFF

    # Handle small sizes directly (size <= 4 bytes)
    slti    t0, a2, 5        # Check if a2 < 5
    bnez    t0, .Lsmall      # If true, jump to handle small cases

    # Align the destination pointer to a 4-byte boundary if necessary
    # a5 = how many bytes to align to 4-byte boundary
    neg     a5, a0
    andi    a5, a5, 3
    beqz    a5, .Laligned      # Already aligned, skip

    # Adjust length and handle misaligned start
    sub     a2, a2, a5          # Reduce length by alignment bytes
    beqz    a2, .Ldone          # If exactly aligned, we are done

.Lunaligned:
    sb      a1, 0(a0)           # Write 1 byte
    addi    a0, a0, 1           # Increment destination pointer
    addi    a5, a5, -1          # Decrement alignment counter
    bnez    a5, .Lunaligned     # Loop until aligned
    beqz    a2, .Ldone          # If length is zero, exit

.Laligned:
    # Now a0 is aligned to 4-byte boundary, proceed with word-sized stores
    li      t0, 0x01010101      # Load pattern for 4 bytes
    mul     t0, t0, a1          # Fill each byte in the word with a1

    # Main loop to write in 4-byte blocks
.Lwordloop:
    slti    t0, a2, 4           # Check if remaining bytes are < 4
    bnez    t0, .Lfinalbytes    # If fewer than 4 bytes left, switch to byte store
    sw      t0, 0(a0)           # Store 4 bytes (word)
    addi    a0, a0, 4           # Increment pointer by 4
    addi    a2, a2, -4          # Decrement length by 4
    bnez    a2, .Lwordloop      # Repeat if there are more blocks

.Lfinalbytes:
    # Handle remaining 1-3 bytes
    beqz    a2, .Ldone          # If no bytes left, exit
.Lsmall:
    sb      a1, 0(a0)           # Write remaining bytes
    addi    a0, a0, 1
    addi    a2, a2, -1
    bnez    a2, .Lsmall

.Ldone:
    ret                         # Return from function

    .size   memset, .-memset
