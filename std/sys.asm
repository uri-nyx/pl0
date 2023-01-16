; These are the system constants for the Taleä Computer System

; DEVICES

TTY = 0x0000
KBD = 0x0006
VDO = 0x000a
TPS = 0x001a
DSK = 0x0020
_IVT = 0xf800
_PDT = 0xff00

; TTY
T_RX = TTY + 0x00
T_RXLEN = TTY + 0x01
T_TX = TTY + 0x02
T_STAT = TTY + 0x03
T_CTRL = TTY + 0x04

; VIDEO
V_COMMAND = VDO + 0x0
V_DATAH   = VDO + 0x1
V_DATAM   = VDO + 0x2
V_DATAL   = VDO + 0x3
V_GPU0    = VDO + 0x4
V_GPU1    = VDO + 0x5
V_GPU2    = VDO + 0x6
V_GPU3    = VDO + 0x7
V_GPU4    = VDO + 0x8
V_GPU5    = VDO + 0x9
V_GPU6    = VDO + 0xa
V_GPU7    = VDO + 0xb
V_STATUS0 = VDO + 0xc
V_STATUS1 = VDO + 0xd
V_STATUS2 = VDO + 0xe
V_STATUS3 = VDO + 0xf

; commands
V_nop       = 0x0
V_clear     = 0x1
V_setmode   = 0x2
V_setfont   = 0x4
V_blit      = 0x6

; KEYBOARD
K_CHARACTER = KBD + 0x00
K_CODE      = KBD + 0x01
K_MODIFIERS = KBD + 0x02
K_MODE      = KBD + 0x03

; modes
K_mchar = 0
K_mscan = 1

; TPS
TPS_COMMAND = TPS + 0x00
TPS_DATA    = TPS + 0x01
TPS_POINTH  = TPS + 0x02
TPS_POINTL  = TPS + 0x03
TPS_STATUSH = TPS + 0x04
TPS_STATUSL = TPS + 0x05

;commands
TPS_nop      = 0x00
TPS_bootable = 0x01
TPS_present  = 0x02
TPS_open     = 0x03
TPS_close    = 0x04
TPS_store    = 0x05
TPS_load     = 0x06

TPS_0 = 0
TPS_1 = 1

; DISK

DISK_COMMAND = 0x00
DISK_DATA    = 0x01
DISK_SECTORH = 0x02
DISK_SECTORL = 0x03
DISK_POINTH  = 0x04
DISK_POINTL  = 0x05
DISK_STATUS0 = 0x06
DISK_STATUS1 = 0x07

;commands

DISK_nop    = 0x0
DISK_store  = 0x1
DISK_load   = 0x2

; Exceptions
IVT_RESET               = 0x00 * 4
IVT_BUS_ERROR           = 0x02 * 4
IVT_ADDRESS_ERROR       = 0x03 * 4
IVT_ILLEGAL_INSTRUCTION = 0x04 * 4
IVT_DIVISION_ZERO       = 0x05 * 4
IVT_PRIVILEGE_VIOLATION = 0x06 * 4
IVT_PAGE_FAULT          = 0x07 * 4
IVT_ACCESS_VIOLATION    = 0x08 * 4
IVT_TTY_TRANSMIT       = 0x0a * 4
IVT_KBD_CHARACTER      = 0x0b * 4
IVT_KBD_SCANCODE       = 0x0c * 4
IVT_TPS_LOAD_FINISHED  = 0x0d * 4
IVT_DISK_LOAD_FINISHED = 0x0e * 4