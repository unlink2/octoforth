:i compile :asm "__ARG__:\n" ;

:i initfth :asm "   ldx FTHPTR\n" ;

:i push_i8im :asm "    sta FTHSTACK,x
    inx\n" ;

:i push_i8 :asm "    lda #__ARG__\n" push_i8im ;

:i push_default push_i8 ;

:i pull_i8 :asm "    dex
    lda FTHSTACK,x\n" ;

:i dup pull_i8 push_i8im push_i8im ;
:i call :asm "    jsr __ARG__" ;
:i return :asm "\n    rts\n" ;

:i start :asm "start:\n" ;

:i +
    pull_i8
    :asm "    dex
    clc
    adc FTHSTACK,x\n"
    push_i8im ;

:i +f
    :asm "    dex
    dex
    clc
    adc FTHSTACK,x\n"
    push_i8im ;


:i __loop
    :asm "@loop:\n" ;

:i __until
    pull_i8
    :asm "    bne @loop" ;

:i run :asm "@loop:" ;
:i forever
    :asm "    jmp @loop" ;
