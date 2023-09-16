factorial:
	LD 0
	INT 1
	OP <=
	JPF @lbl_1
	INT 1
	JP @lbl_0
@lbl_1:
	LD 0
	PTR factorial
	LD 0
	INT 1
	OP -
	CALL 1
	OP *
@lbl_0:
	RET
__ctor__:
	PTR print
	PTR factorial
	INT 6
	CALL 1
	CALL 1
	RET
