
:i compile :asm "__ARG__:\n" ;
:i push8 :asm "    lda #__ARG__
    pha
" ;

:i dup :asm "    pla
    pha
    pha" ;
:i call :asm "    jsr __ARG__" ;
:i return :asm "\n    rts\n" ;
:i + :asm "    pla
    sta $00
    pla
    clc
    adc $00
    pha" ;

:i forever :asm "forever:\n    jmp forever\n" ;
