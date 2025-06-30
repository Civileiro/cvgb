
SECTION "Main", ROM0[$0]
    ld a, 10
    ld [$8000], a
    ld a, 3
    ld [$8001], a
    ld a, [$8000]
    ld b, a
    ld a, [$8000+1]
    ld c, a
    ld a, 0
    ld hl, $8001
    scf
    ccf
loop:
    adc a, [hl]
    dec b
    jr nz, loop
    ld [$8002], a
    nop
    nop
    nop

