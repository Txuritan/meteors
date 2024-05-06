# eaves virtual machine

A simple register based virtual machine.

## Instructions

| instruction |  byte  | description                        |
| :---------- | :----: | :--------------------------------- |
| stop        | `0x00` | stops the virtual machine          |
| nop         | `0x01` | stops the virtual machine          |
| load        | `0x10` | loads memory from register         |
| move        | `0x11` | moves memory to or from a register |
| copy        | `0x12` | copies memory from a register      |
| jump        | `0x13` | jump to a offset in memory         |
| add         | `0x20` |                                    |
| sub         | `0x21` |                                    |
| mul         | `0x22` |                                    |
| div         | `0x23` |                                    |
| mod         | `0x24` |                                    |
| exp         | `0x25` |                                    |
| and         | `0x30` | preform a bitwise operation        |
| or          | `0x31` | preform a bitwise operation        |
| shift left  | `0x32` |                                    |
| shift right | `0x33` |                                    |
| compare     | `0x40` | preform a comparison               |
