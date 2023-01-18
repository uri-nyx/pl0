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
    .char:
        la a0, .msg
        push ra, sp
        jal ra, crt0.puts
        mv a0, zero ; wait for interrupt
        ..wait: beq a0, zero, ..wait
        lbud a0, T_RX(zero)
        pop ra, sp
        jalr zero, 0(ra)

    .int:
        la a0, .msg
        push ra, sp
        jal ra, crt0.puts
        pop ra, sp
        mv a0, zero ; wait for interrupt
        ..wait: beq a0, zero, ..wait

            
        ..convert:                  ; number = number + (char - 0x30 * step * 10)
            addi t5, zero, 1
            mv a0, zero
            ...loop:
                lbud t1, T_RX(zero)     ; char
                beq t1, zero, ...return ; NULL
                mv t3, t1
                subi t3, t3, 0x2d       ; minus sign
                beq t3, zero, ...return.negative
                mv t3, t1
                subi t3, t3, 0x2b       ; plus sign
                beq t3, zero, ...return
                subi t1, t1, 0x30       ; digit
                sltiu t4, t1, 10        ; is less than 10?
                beq t4, zero, ...err    ; Error converting, expected number
                mul zero, t1, t1, t5    ; digit * step (*10)
                add a0, a0, t1          ; add ponderated digit to a0
                muli t5, t5, 10
                addi t3, t3, 1
                j ...loop
            
            ...err:
                la a0, ...msg
                push ra, sp
                jal ra, crt0.puts
                pop ra, sp
                addi a0, a0, 0x2ead
            ...return:
                jalr zero, 0(ra)
                ....negative:
                    not a0, a0
                    addi a0, a0, 1
                    jalr zero, 0(ra)

            ...msg: #d"Error converting input, expected only numeric characters\n\0"
            #align 32
            

    .msg: #d "pl/0> \0"
    #align 32
        
PL0_OUTPUT:
    addi t1, zero, 11
    la t3, global.out_buff
    fill t3, t1, zero
    addi t3, t3, 10
    addi t1, zero, 10

    .check_for_sign:
        shirl t2, a0, 31    ; sign bit
        beq t2, zero, ..positive
        ..negative:
            addi t4, zero, 0x2d
            not a0, a0      ; turn to positive
            addi a0, a0, 1
            j PL0_OUTPUT.loop
        ..positive:
            addi t4, zero, 0x2b



    .loop:
        idiv a0, t2, a0, t1      ; a0 /= 10, t2 = a0 % 10
        addi t2, t2, 0x30
        sb t2, 0(t3)
        beq a0, zero, .end
        subi t3, t3, 1
        j .loop
    
    .end:
        sbd t4, T_TX(zero)
        mv a0, t3
        push ra, sp
        jal ra, crt0.puts
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
    addi a0, a0, 1
    sysret

HANDLER_KBD_CHARACTER:
    ;lbud a0, K_CHARACTER(zero)   ; Notice that PL/0 does not use yet the keyboard interface
    sysret

HANDLER_KBD_SCANCODE:
    lbud a0, K_CODE(zero)    ; Notice that PL/0 does not use yet the keyboard interface
    sysret

HANDLER_TPS_LOAD_FINISHED:
    sysret                       ; Does nothing in PL/0

HANDLER_DISK_LOAD_FINISHED:
    sysret                       ; Does nothing in PL/0



