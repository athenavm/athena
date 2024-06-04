# Instruction Set

### RV32I Base Integer Instruction Set

| Instruction | Description                   |
|-------------|-------------------------------|
| ADD         | Add                           |
| SUB         | Subtract                      |
| XOR         | Bitwise XOR                   |
| OR          | Bitwise OR                    |
| AND         | Bitwise AND                   |
| SLL         | Shift Left Logical            |
| SRL         | Shift Right Logical           |
| SRA         | Shift Right Arithmetic        |
| SLT         | Set Less Than                 |
| SLTU        | Set Less Than Unsigned        |
| ADDI        | Add Immediate                 |
| XORI        | XOR Immediate                 |
| ORI         | OR Immediate                  |
| ANDI        | AND Immediate                 |
| SLLI        | Shift Left Logical Imm.       |
| SRLI        | Shift Right Logical Imm.      |
| SRAI        | Shift Right Arithmetic Imm.   |
| SLTI        | Set Less Than Immediate       |
| SLTIU       | Set Less Than Unsigned Imm.   |
| LB          | Load Byte                     |
| LH          | Load Halfword                 |
| LW          | Load Word                     |
| LBU         | Load Byte Unsigned            |
| LHU         | Load Halfword Unsigned        |
| SB          | Store Byte                    |
| SH          | Store Halfword                |
| SW          | Store Word                    |
| BEQ         | Branch if Equal               |
| BNE         | Branch if Not Equal           |
| BLT         | Branch if Less Than           |
| BGE         | Branch if Greater or Equal    |
| BLTU        | Branch if Less Than Unsigned  |
| BGEU        | Branch if Greater or Equal Unsigned |
| JAL         | Jump and Link                 |
| JALR        | Jump and Link Register        |
| LUI         | Load Upper Immediate          |
| AUIPC       | Add Upper Immediate to PC     |
| ECALL       | Environment Call              |
| EBREAK      | Environment Break             |

### RV32M Standard Extension for Integer Multiply and Divide

| Instruction | Description                   |
|-------------|-------------------------------|
| MUL         | Multiply                      |
| MULH        | Multiply High                 |
| MULHSU      | Multiply High Signed-Unsigned |
| MULHU       | Multiply High Unsigned        |
| DIV         | Divide                        |
| DIVU        | Divide Unsigned               |
| REM         | Remainder                     |
| REMU        | Remainder Unsigned            |

## Reference

See https://www.cs.sfu.ca/~ashriram/Courses/CS295/assets/notebooks/RISCV/RISCV_CARD.pdf,
https://five-embeddev.com/riscv-user-isa-manual/Priv-v1.12/preface.html,
https://michaeljclark.github.io/isa.html,
https://riscv.org/technical/specifications/
