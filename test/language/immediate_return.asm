.label __func_0
	push bp
	mov bp, sp
	movi r0, __func_1
	store [bp -20], r0
	load r0, [bp -20]
	callr r0
	store [bp -16], r0
	load r0, [bp -16]
	store [bp +24], r0
	lea r0, [bp +24]
	store [bp -12], r0
	movi r0, __func_2
	store [bp -8], r0
	load r0, [bp -8]
	callr r0
	store [bp -4], r0
	load r0, [bp -4]
	load r1, [bp -12]
	store [r1], r0
	movi r0, 0
	halt
	halt
.label __func_1
	push bp
	mov bp, sp
	movi r0, 4
	store [bp -4], r0
	load r0, [bp -4]
	jmp __func_1_exit
.label __func_1_exit
	mov sp, bp
	pop bp
	ret
.label __func_2
	push bp
	mov bp, sp
	movi r0, 2
	store [bp -4], r0
	load r0, [bp -4]
	jmp __func_2_exit
.label __func_2_exit
	mov sp, bp
	pop bp
	ret
.label __str_0 .data "Hello, World!", 0
.label __global_0 .word __str_0
