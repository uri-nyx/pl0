#include "master.asm"
#include "sys.asm"

# addr 0
crt0:
    li  a0, le(0b1_1_0_010_111110_11111111_000000000000)
    ssreg a0    ; supervisor, intterupt enabled, mmu disabled
                ; priority 2, ivt at 0xf800, pdt at 0xff00
    
    ; IVT initialization
    li t1, _IVT
    la a0, HANDLER_RESET
    swd a0, IVT_RESET(t1)
    la a0, HANDLER_BUS_ERROR
    swd a0, IVT_BUS_ERROR(t1)
    la a0, HANDLER_ADDRESS_ERROR
    swd a0, IVT_ADDRESS_ERROR(t1)
    la a0, HANDLER_ILLEGAL_INSTRUCTION
    swd a0, IVT_ILLEGAL_INSTRUCTION(t1)
    la a0, HANDLER_DIVISION_ZERO
    swd a0, IVT_DIVISION_ZERO(t1)
    la a0, HANDLER_PRIVILEGE_VIOLATION
    swd a0, IVT_PRIVILEGE_VIOLATION(t1)
    la a0, HANDLER_PAGE_FAULT
    swd a0, IVT_PAGE_FAULT(t1)
    la a0, HANDLER_ACCESS_VIOLATION
    swd a0, IVT_ACCESS_VIOLATION(t1)
    la a0, HANDLER_TTY_TRANSMIT
    swd a0, IVT_TTY_TRANSMIT(t1)
    la a0, HANDLER_KBD_CHARACTER
    swd a0, IVT_KBD_CHARACTER(t1)
    la a0, HANDLER_KBD_SCANCODE
    swd a0, IVT_KBD_SCANCODE(t1)
    la a0, HANDLER_TPS_LOAD_FINISHED
    swd a0, IVT_TPS_LOAD_FINISHED(t1)
    la a0, HANDLER_DISK_LOAD_FINISHED
    swd a0, IVT_DISK_LOAD_FINISHED(t1)

    ; Stack initialization
    la sp, global.stack
    jal ra, Start
    j .exit

    .putc:                          ; Put Character (a0: char)
        sbd a0, T_TX(zero)
        jalr zero, 0(ra)

    .puts:                          ; Put String (a0: char*)
        mv a1, a0
        ..loop:
            lbu a0, 0(a1)
            beq a0, zero, ...end
            sbd a0, T_TX(zero)
            addi a1, a1, 1
            j ..loop

            ...end:
                jalr zero, 0(ra)

    .exit:
        la a0, ..msg
        jal ra, .puts
        ..halt:
        j ..halt
        ..msg:
            #d "Execution Terminated. ^C to close the emulator.\0"
            #align 32

PL0_INPUT:
    la a0, .msg
    jal ra, crt0.puts

    .wait_for_input:
        lbu a0, T_RXLEN(zero)
        bne a0, zero, .end
        j .wait_for_input

    .end:
        lbu a0, T_RX(zero)
        pop ra, sp
        jalr zero, 0(ra)

    .msg:
        #d "pl/0> \0"
        #align 32

PL0_OUTPUT:
    addi t1, zero, 10
    la t3, global.out_buff
    fill t3, t1, zero
    addi t3, t3, 10

    .loop:
        idiv a0, t2, a0, t1      ; a0 /= 10, t2 = a0 % 10
        addi t2, t2, 0x30
        sb t2, 0(t3)
        beq a0, zero, .end
        subi t3, t3, 1
        j .loop
    
    .end:
        mv a0, t3
        push ra, sp
        jal ra, crt0.puts
        addi a0, zero, 10
        sbd a0, T_TX(zero)
        pop ra, sp
        jalr zero, 0(ra)

;---------------------------------------
;   Interrupt and Exception Handlers
;---------------------------------------
HANDLER_RESET:
    j crt0

HANDLER_BUS_ERROR:
    la a0, .msg
    jal ra, crt0.puts
    j HANDLER_RESET
    .msg:
        #d "Abort: Bus Error\n The system will restart\n\0"
        #align 32

HANDLER_ADDRESS_ERROR:
    la a0, .msg
    jal ra, crt0.puts
    j HANDLER_RESET
    .msg:
        #d "Abort: Address Error\n The system will restart\n\0"
        #align 32

HANDLER_ILLEGAL_INSTRUCTION:
    la a0, .msg
    jal ra, crt0.puts
    j crt0.exit
    .msg:
        #d "Trap: Illegal Instruction\n The process will be terminated\n\0"
        #align 32

HANDLER_DIVISION_ZERO:
    la a0, .msg
    jal ra, crt0.puts
    j crt0.exit
    .msg:
        #d "Trap: Division by zero\n The process will be terminated\n\0"
        #align 32

HANDLER_PRIVILEGE_VIOLATION:
    la a0, .msg
    jal ra, crt0.puts
    j crt0.exit
    .msg:
        #d "Trap: Privilege violation\n The process will be terminated\n\0"
        #align 32

HANDLER_PAGE_FAULT:
    la a0, .msg
    jal ra, crt0.puts
    j crt0.exit
    .msg:
        #d "Page Fault Unreachable: MMU should not be enabled for pl/0\n\0"
        #align 32

HANDLER_ACCESS_VIOLATION:
    la a0, .msg
    jal ra, crt0.puts
    j crt0.exit
    .msg:
        #d "Access Violation Unreachable: MMU should not be enabled for pl/0\n\0"
        #align 32

HANDLER_TTY_TRANSMIT:
    lbud a0, T_RX(zero)          ; Notice that PL/0 only can read one character
    sysret

HANDLER_KBD_CHARACTER:
    lbud a0, K_CHARACTER(zero)   ; Notice that PL/0 only can read one character
    sysret

HANDLER_KBD_SCANCODE:
    lbud a0, K_CODE(zero)    ; Notice that PL/0 only can read one character
    sysret

HANDLER_TPS_LOAD_FINISHED:
    sysret                       ; Does nothing in PL/0

HANDLER_DISK_LOAD_FINISHED:
    sysret                       ; Does nothing in PL/0



