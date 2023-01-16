; This is the basic assembler implementation for the Taleä Computer System
#once

SYS    = 0b000
MEM    = 0b001
BRANCH = 0b011
LOAD   = 0b100
ALUI   = 0b101
ALUR   = 0b110
STORE  = 0b111

BLANK5 = 0b00000
BLANK10 = 0b00000_00000
BLANK15 = 0b00000_00000_00000

LUI     = 0b010_0001
AUIPC   = 0b010_0010
JAL     = 0b010_0000
JALR    = 0b100_0001

BEQ     = BRANCH @ 0x0
BNE     = BRANCH @ 0x1
BLT     = BRANCH @ 0x2
BGE     = BRANCH @ 0x3
BLTU    = BRANCH @ 0x4
BGEU    = BRANCH @ 0x5

LB      = LOAD @ 0x2
LBU     = LOAD @ 0x3
LBD     = LOAD @ 0x4
LBUD    = LOAD @ 0x5
LH      = LOAD @ 0x6
LHU     = LOAD @ 0x7
LHD     = LOAD @ 0x8
LHUD    = LOAD @ 0x9
LW      = LOAD @ 0xa
LWD     = LOAD @ 0xb

MULI    = ALUI @ 0x0
MULIH   = ALUI @ 0x1
IDIVI   = ALUI @ 0x2
ADDI    = ALUI @ 0x3
SUBI    = ALUI @ 0x4

ORI     = ALUI @ 0x5
ANDI    = ALUI @ 0x6
XORI    = ALUI @ 0x7
SHIRA   = ALUI @ 0x8
SHIRL   = ALUI @ 0x9
SHILL   = ALUI @ 0xa

SLTI    = ALUI @ 0xb
SLTIU   = ALUI @ 0xc

ADD     = ALUR @ 0x0 
SUB     = ALUR @ 0x1 
IDIV    = ALUR @ 0x2
MUL     = ALUR @ 0x3

OR      = ALUR @ 0x4 
AND     = ALUR @ 0x5
XOR     = ALUR @ 0x6 

NOT     = ALUR @ 0x7
CTZ     = ALUR @ 0x8
CLZ     = ALUR @ 0x9
PCOUNT  = ALUR @ 0xa

SHRA    = ALUR @ 0xb 
SHRL    = ALUR @ 0xc 
SHLL    = ALUR @ 0xd 
ROR     = ALUR @ 0xe
ROL     = ALUR @ 0xf

SB      = STORE @ 0x0
SBD     = STORE @ 0x1
SH      = STORE @ 0x2
SHD     = STORE @ 0x3
SW      = STORE @ 0x4
SWD     = STORE @ 0x5

COPY    = MEM @ 0x0
SWAP    = MEM @ 0x1
FILL    = MEM @ 0x2
THRO    = MEM @ 0x3
FROM    = MEM @ 0x4

POPB    = MEM @ 0x5
POPH    = MEM @ 0x6
POP     = MEM @ 0x7
PUSHB   = MEM @ 0x8
PUSHH   = MEM @ 0x9
PUSH    = MEM @ 0xa

SAVE    = MEM @ 0xb
RESTORE = MEM @ 0xc
EXCH    = MEM @ 0xd
SLT     = MEM @ 0xe 
SLTU   = MEM @ 0xf 

SYSCALL  = SYS @ 0x2
GSREG    = SYS @ 0x3
SSREG    = SYS @ 0x4
SYSRET   = SYS @ 0x6


#ruledef reg {
    zero => 0b00000
    ra  =>  0b00001
    sp  =>  0b00010
    gp  =>  0b00011
    tp  =>  0b00100
    t0  =>  0b00101
    t1  =>  0b00110
    t2  =>  0b00111
    s1  =>  0b01001
    fp  =>  0b01000
    a0  =>  0b01010
    a1  =>  0b01011
    a2  =>  0b01100
    a3  =>  0b01101
    a4  =>  0b01110
    a5  =>  0b01111
    a6  =>  0b10000
    a7  =>  0b10001
    s2  =>  0b10010
    s3  =>  0b10011
    s4  =>  0b10100
    s5  =>  0b10101
    s6  =>  0b10110
    s7  =>  0b10111
    s8  =>  0b11000
    s9  =>  0b11001
    s10 =>  0b11010
    s11 =>  0b11011
    t3  =>  0b11100
    t4  =>  0b11101
    t5  =>  0b11110
    t6  =>  0b11111
    r {n: u5} => n
}

#ruledef {
    lui   {rd: reg}, {imm: u20}  => LUI   @ rd @ imm
    auipc {rd: reg}, {imm: u20}  => AUIPC @ rd @ imm
    jal   {rd: reg}, {label}  => {
        imm = label - $
        assert((imm % 4) == 0)
        ; NOTE OFFSETS FOR JUMPS MUST BE 4 BYTE ALIGNED,
        ; OTHERWISE IT WILL RESULT IN AN EXCEPTION
        imm20 = imm >> 2
        JAL @ rd @ imm20`20
    }
    jalr {rd: reg}, {imm: s15}({rs1: reg}) => JALR @ rd @ rs1 @ imm 

    beq  {rs1: reg}, {rs2: reg}, {label} => {
        imm = label - $
        assert((imm % 4) == 0)
        ; NOTE OFFSETS FOR JUMPS MUST BE 4 BYTE ALIGNED,
        ; OTHERWISE IT WILL RESULT IN AN EXCEPTION
        imm15 = imm >> 2
        BEQ @ rs1 @ rs2 @ imm15`15
    }
    bne  {rs1: reg}, {rs2: reg}, {label} => {
        imm = label - $
        assert((imm % 4) == 0)
        ; NOTE OFFSETS FOR JUMPS MUST BE 4 BYTE ALIGNED,
        ; OTHERWISE IT WILL RESULT IN AN EXCEPTION
        imm15 = imm >> 2
        BNE @ rs1 @ rs2 @ imm15`15
    }
    blt  {rs1: reg}, {rs2: reg}, {label} => {
        imm = label - $
        assert((imm % 4) == 0)
        ; NOTE OFFSETS FOR JUMPS MUST BE 4 BYTE ALIGNED,
        ; OTHERWISE IT WILL RESULT IN AN EXCEPTION
        imm15 = imm >> 2
        BLT @ rs1 @ rs2 @ imm15`15
    }
    bge  {rs1: reg}, {rs2: reg}, {label} => {
        imm = label - $
        assert((imm % 4) == 0)
        ; NOTE OFFSETS FOR JUMPS MUST BE 4 BYTE ALIGNED,
        ; OTHERWISE IT WILL RESULT IN AN EXCEPTION
        imm15 = imm >> 2
        BGE @ rs1 @ rs2 @ imm15`15
    }
    bltu {rs1: reg}, {rs2: reg}, {label} => {
        imm = label - $
        assert((imm % 4) == 0)
        ; NOTE OFFSETS FOR JUMPS MUST BE 4 BYTE ALIGNED,
        ; OTHERWISE IT WILL RESULT IN AN EXCEPTION
        imm15 = imm >> 2
        BLTU @ rs1 @ rs2 @ imm15`15
    }
    bgeu {rs1: reg}, {rs2: reg}, {label} => {
        imm = label - $
        assert((imm % 4) == 0)
        ; NOTE OFFSETS FOR JUMPS MUST BE 4 BYTE ALIGNED,
        ; OTHERWISE IT WILL RESULT IN AN EXCEPTION
        imm15 = imm >> 2
        BGEU @ rs1 @ rs2 @ imm15`15
    }

    lb   {rd: reg}, {imm: s15}({rs1: reg})  => LB @ rd @ rs1 @ imm
    lbu  {rd: reg}, {imm: s15}({rs1: reg})  => LBU @ rd @ rs1 @ imm
    lbd  {rd: reg}, {imm: s15}({rs1: reg})  => LBD @ rd @ rs1 @ imm
    lbud {rd: reg}, {imm: s15}({rs1: reg})  => LBUD @ rd @ rs1 @ imm
    lh   {rd: reg}, {imm: s15}({rs1: reg})  => LH @ rd @ rs1 @ imm
    lhu  {rd: reg}, {imm: s15}({rs1: reg})  => LHU @ rd @ rs1 @ imm
    lhd  {rd: reg}, {imm: s15}({rs1: reg})  => LHD @ rd @ rs1 @ imm
    lhud {rd: reg}, {imm: s15}({rs1: reg})  => LHUD @ rd @ rs1 @ imm
    lw   {rd: reg}, {imm: s15}({rs1: reg})  => LW @ rd @ rs1 @ imm
    lwd  {rd: reg}, {imm: s15}({rs1: reg})  => LWD @ rd @ rs1 @ imm

    muli  {rd: reg}, {rs1: reg}, {imm: i15} => MULI @ rd @ rs1 @ imm
    mulih {rd: reg}, {rs1: reg}, {imm: i15} => MULIH @ rd @ rs1 @ imm
    idivi {rd: reg}, {rs1: reg}, {imm: i15} => {
        assert(imm != 0)
        IDIVI @ rd @ rs1 @ imm
    }
    addi  {rd: reg}, {rs1: reg}, {imm: i15} => ADDI @ rd @ rs1 @ imm
    subi  {rd: reg}, {rs1: reg}, {imm: i15} => SUBI @ rd @ rs1 @ imm

    ori   {rd: reg}, {rs1: reg}, {imm: i15} => ORI @ rd @ rs1 @ imm
    andi  {rd: reg}, {rs1: reg}, {imm: i15} => ANDI @ rd @ rs1 @ imm
    xori  {rd: reg}, {rs1: reg}, {imm: i15} => XORI @ rd @ rs1 @ imm
    shira {rd: reg}, {rs1: reg}, {imm: i15} => SHIRA @ rd @ rs1 @ imm
    shirl {rd: reg}, {rs1: reg}, {imm: i15} => SHIRL @ rd @ rs1 @ imm
    shill {rd: reg}, {rs1: reg}, {imm: i15} => SHILL @ rd @ rs1 @ imm

    slti  {rd: reg}, {rs1: reg}, {imm: i15} => SLTI @ rd @ rs1 @ imm
    sltiu {rd: reg}, {rs1: reg}, {imm: u15} => SLTIU @ rd @ rs1 @ imm

    add    {rd: reg}, {rs1: reg}, {rs2: reg} => ADD @ rd @ rs1 @ rs2 @ BLANK10
    sub    {rd: reg}, {rs1: reg}, {rs2: reg} => SUB @ rd @ rs1 @ rs2 @ BLANK10
    idiv   {rd: reg}, {rd2: reg}, {rs1: reg}, {rs2: reg} => IDIV @ rd @ rd2 @ rs1 @ rs2 @ BLANK5
    mul    {rd: reg}, {rd2: reg}, {rs1: reg}, {rs2: reg} => MUL @ rd @ rd2 @ rs1 @ rs2  @ BLANK5

    or     {rd: reg}, {rs1: reg}, {rs2: reg} => OR @ rd @ rs1 @ rs2  @ BLANK10
    and    {rd: reg}, {rs1: reg}, {rs2: reg} => AND @ rd @ rs1 @ rs2 @ BLANK10
    xor    {rd: reg}, {rs1: reg}, {rs2: reg} => XOR @ rd @ rs1 @ rs2 @ BLANK10

    slt    {rd: reg}, {rs1: reg}, {rs2: reg} => SLT @ rd @ rs1 @ rs2 @ BLANK10
    sltu   {rd: reg}, {rs1: reg}, {rs2: reg} => SLTU @ rd @ rs1 @ rs2 @ BLANK10

    not    {rd: reg}, {rs1: reg} => NOT @ rd @ rs1 @ BLANK15
    ctz    {rd: reg}, {rs1: reg} => CTZ @ rd @ rs1 @ BLANK15
    clz    {rd: reg}, {rs1: reg} => CLZ @ rd @ rs1 @ BLANK15
    pcount {rd: reg}, {rs1: reg} => PCOUNT @ rd @ rs1 @ BLANK15

    shra   {rd: reg}, {rs1: reg}, {rs2: reg} => SHRA @ rd @ rs1 @ rs2 @ BLANK10
    shrl   {rd: reg}, {rs1: reg}, {rs2: reg} => SHRL @ rd @ rs1 @ rs2 @ BLANK10
    shll   {rd: reg}, {rs1: reg}, {rs2: reg} => SHLL @ rd @ rs1 @ rs2 @ BLANK10
    ror    {rd: reg}, {rs1: reg}, {rs2: reg} => ROR @ rd @ rs1 @ rs2  @ BLANK10
    rol    {rd: reg}, {rs1: reg}, {rs2: reg} => ROL @ rd @ rs1 @ rs2  @ BLANK10

    sb  {rs2: reg}, {imm: i15}({rs1: reg})   => SB @ rs2 @ rs1 @ imm
    sbd {rs2: reg}, {imm: i15}({rs1: reg})   => SBD @ rs2 @ rs1 @ imm
    sh  {rs2: reg}, {imm: i15}({rs1: reg})   => SH @ rs2 @ rs1 @ imm
    shd {rs2: reg}, {imm: i15}({rs1: reg})   => SHD @ rs2 @ rs1 @ imm
    sw  {rs2: reg}, {imm: i15}({rs1: reg})   => SW @ rs2 @ rs1 @ imm
    swd {rs2: reg}, {imm: i15}({rs1: reg})   => SWD @ rs2 @ rs1 @ imm

    copy    {rd: reg}, {rs1: reg}, {rs2: reg} => COPY @ rd @ rs1 @ rs2 @ BLANK10
    swap    {rd: reg}, {rs1: reg}, {rs2: reg} => SWAP @ rd @ rs1 @ rs2 @ BLANK10
    fill    {rd: reg}, {rs1: reg}, {rs2: reg} => FILL @ rd @ rs1 @ rs2 @ BLANK10
    thro    {rd: reg}, {rs1: reg} => THRO @ rd @ rs1 @ BLANK15
    from    {rd: reg}, {rs1: reg} => FROM @ rd @ rs1 @ BLANK15

    popb    {rd: reg}, {rs1: reg} => POPB @ rd @ rs1 @ BLANK15
    poph    {rd: reg}, {rs1: reg} => POPH @ rd @ rs1 @ BLANK15
    pop     {rd: reg}, {rs1: reg} => POP @ rd @ rs1  @ BLANK15
    pushb   {rd: reg}, {rs1: reg} => PUSHB @ rd @ rs1 @ BLANK15
    pushh   {rd: reg}, {rs1: reg} => PUSH @ rd @ rs1  @ BLANK15
    push    {rd: reg}, {rs1: reg} => PUSH @ rd @ rs1  @ BLANK15

    save    {rd: reg}, {rs1: reg}, {rs2: reg} => SAVE @ rd @ rs1 @ rs2 @ BLANK10
    restore {rd: reg}, {rs1: reg}, {rs2: reg} => RESTORE @ rd @ rs1 @ rs2 @ BLANK10
    exch    {rd: reg}, {rs1: reg}  => EXCH @ rd @ rs1 @ BLANK15
    
    syscall {rd: reg}, {vector: u8} => SYSCALL @ rd @ 0x000 @ vector 
    gsreg   {rd: reg} => GSREG @ rd @ BLANK10 @ BLANK10
    ssreg   {rs1: reg} => SSREG @ rs1 @ BLANK10 @ BLANK10
    sysret  => SYSRET @ BLANK15 @ BLANK10
}

#ruledef {
    li {rd: reg}, {const: i32} => asm {
        assert(const < 0x3fff)
        addi {rd}, zero, const`15
    }
    li {rd: reg}, {const: i32}  => asm {
        lui {rd}, (const >> 12)
        addi {rd}, {rd}, const`12
    }
    la {rd: reg}, {label} => asm {
        auipc    {rd}, ((label-$) >> 12)`20
        addi     {rd},{rd},((label-($-4))`12)
    }
    llb {rd: reg}, {label} => asm {
        auipc {rd}, ((label-$) >> 12)`20
        lb {rd}, ((label-($-4))`12)({rd})

    }
    llh {rd: reg}, {label} => asm {
        auipc {rd}, ((label-$) >> 12)`20
        lh {rd}, ((label-($-4))`12)({rd})
    }
    llw {rd: reg}, {label} => asm {
        auipc {rd}, ((label-$) >> 12)`20
        lw {rd}, (label-($-4))`12({rd})
    }
    ssb{rd: reg}, {label}, {rt: reg} => asm {
        auipc    {rt},((label-$) >> 12)`20
        sb {rd}, ((label-($-4))`12)({rt})
    }
    ssh {rd: reg}, {label}, {rt: reg} => asm {
        auipc    {rt},((label-$) >> 12)`20
        sh {rd}, ((label-($-4))`12)({rt})
    }
    ssw {rd: reg}, {label}, {rt: reg} => asm {
        auipc    {rt},((label-$) >> 12)`20
        sw {rd}, ((label-($-4))`12)({rt})
    }
    call {label} => asm {
        auipc    ra, ((label-$) >> 12)`20
        jalr     ra, ((label-($-4)))(ra)
    }
    tail {label}, {rt: reg} => asm {
        auipc    {rt}, ((label-$) >> 12)`20
        jalr     zero, ((label-($-4))`12)({rt})
    }

    mv {rd: reg}, {rs: reg} => asm {addi {rd}, {rs}, 0}
    j {label} => asm {jal zero, label}
    jr {rs: reg} => asm {jalr zero, 0({rs})}
    ret => asm {jalr zero, 0(ra)}
}