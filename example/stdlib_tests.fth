
:i compile :asm "__ARG__:\n" ;
:i push8 :asm "lda #__ARG__\npha";

:i dup :asm "pla\npha\npha\n" ;
:i call :asm "jsr __ARG__" ;
:i return :asm "rts\n" ;
:i + :asm "pla\nsta $00\npla\nclc\nadc $00\npha" ;

:i forever :asm "forever:\njmp forever\n" ;
