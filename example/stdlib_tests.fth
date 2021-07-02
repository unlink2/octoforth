:i compile :asm "__ARG__:\n" ;

:i initfth :asm "   ldx FTHPTR\n" ;

:i push8im :asm "    sta FTHSTACK,x
    inx
" ;

:i push8 :asm "    lda #__ARG__\n" push8im ;

:i pull8 :asm "    dex
    lda FTHSTACK,x
" ;

:i dup pull8 push8im push8im ;
:i call :asm "    jsr __ARG__" ;
:i return :asm "\n    rts\n" ;

:i start :asm "start:\n" ;

:i +
    pull8
    :asm "    sta $80
    "
    pull8
    :asm "    clc
    adc $80
    "
    push8im ;

:i loopstart
    :asm "@loop:\n" ;

:i untilcheck
    pull8
    :asm "    bne @loop" ;
