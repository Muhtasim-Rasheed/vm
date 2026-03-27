.label _start
	movi r0, 0
.label loop
	addi r0, r0, 1

	movi r1, 0
	addi r1, r0, 47
	store [0x001F8000], r1
	movi r1, 10
	store [0x001F8000], r1

	cmpi r0, 10
	jl loop
.label loop_end
	jmp loop_end
