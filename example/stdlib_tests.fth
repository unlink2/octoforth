:i compile :asm "__ARG__:\n" ;

:i initfth :asm "   ldx FTHPTR\n" ;

:i push8im :asm "    sta FTHSTACK,x
    inx\n" ;

:i push8 :asm "    lda #__ARG__\n" push8im ;

:i pull8 :asm "    dex
    lda FTHSTACK,x\n" ;

:i dup pull8 push8im push8im ;
:i call :asm "    jsr __ARG__" ;
:i return :asm "\n    rts\n" ;

:i start :asm "start:\n" ;

:i +
    pull8
    :asm "    dex
    clc
    adc FTHSTACK,x\n"
    push8im ;

:i +f
    :asm "    dex
    dex
    clc
    adc FTHSTACK,x\n"
    push8im ;


:i __loop
    :asm "@loop:\n" ;

:i __until
    pull8
    :asm "    bne @loop" ;

:i run :asm "@loop:" ;
:i forever
    :asm "    jmp @loop" ;
