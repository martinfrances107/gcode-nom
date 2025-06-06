; Round Box demonstrates G2/G2 commands
; a square from 0,0 to 100
; but with rounded boxes

; Bottom flat edge going from left to right
G0 X5 Y0
G1 X95 Y0 E1

; Convert to counter clockwise arc
;G1 X100 Y5 E1
G3 I0 J5 X100 Y5 E1 ; verified

; Right vertical edge going from bottom to top
G1 X100 Y95 E1

; Convert to counter clockwise arc
; G1 X95 Y100 E1
G3 I-5 J0 X95 Y100 E1

; TOP edge going from right to left
G1 X5 Y100 E1

; Convert to counter clockwise arc
; G1 X0 Y95 E1
G3 I0 J-5 X0 Y95 E1

; Left vertical edge - going down
G1 X0 Y5 E1

; Convert to counter clockwise arc
G1 X5 Y0 E1
;G3 I5 J0 X5 Y0 E1 ; BUG WHY DOES THIS NOT WORK
