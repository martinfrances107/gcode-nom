; Round Box demonstrates G2 ARC anti-clockwise commands
; a square from 0,0 to 100
; but with 4 bites take out.

; Bottom flat edge going from left to right
G0 X5 Y0
G1 X95 Y0 E1

; (1) Convert to anti-clockwise arc
G2 I5 J0 X100 Y5 E1

; Right vertical edge going from bottom to top
G1 X100 Y95 E1

; (2) Convert to anti-clockwise arc
G2 I0 J5 X95 Y100 E1

; TOP edge going from right to left
G1 X5 Y100 E1

; (3) Convert to anti-clockwise arc
G2 I-5 J0 X0 Y95 E1

; Left vertical edge - going down
G1 X0 Y5 E1

; (4) Convert to counter clockwise arc
G2 J-5 J0 X5 Y0 E1