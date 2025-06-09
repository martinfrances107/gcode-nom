; Mix of G2 and G3 Arc commands.
;
; Leading zero test
; Parsing check G00/G01 must be correctly decoded.G00 X.5 Y3.5 Z.5 E1

; horizontal and square corner
G00X5 Y35
G01X22.5 E2
G02X27.5 Y30 I0 J-5 E1
G01Y7.5 E1

; triangular indent
G01X14.51 Y0 E1
G01X27.5 Y-7.5 E1

; straight section and corner
G01Y-30 E1
G02X22.5 Y-35 I-5 J0 E1

; indent ending in a curve
G01X5 E1
G01Y-20  E1
G03X-5 Y-20 I-5 J0  E1

G01Y-35 E1
G01X-22.5 E1
G02X-27.5 Y-30 I0 J5 E1

; triangular indent
G01Y-7.5 E1
G01X-14.1 Y0 E1
G01X-27.5 Y7.5 E1

G01Y30 E1
G02X-22.5 Y35 I5 J0 E1
G01X-5 E1

G01Y25 E1
G03X5 Y25 I5 J0 E1
G01Y35 E1
G01X22.5 E1
G00Z5 E1