#FN
add:
#ARG a
#ARG b
	LD a
	LD b
	OP +
	RET
#FN
__ctor__:
	LD print
	LD add
	INT 2
	INT 4
	CALL 2
	CALL 1
	RET
