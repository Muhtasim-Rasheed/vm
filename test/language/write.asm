.label __func_0
	push bp
	mov bp, sp
	movi r0, 100
	store [bp-60], r0
	not r0
	addi r0, r0, 1
	store [bp-64], r0
	store [bp+68], r0
.label __func_0L0
	load r0, [bp+68]
	store [bp-52], r0
	movi r0, 101
	store [bp-48], r0
	load r0, [bp-52]
	load r1, [bp-48]
	cmp r0, r1
	setl r0
	store [bp-56], r0
	cmpi r0, 0
	je __func_0L1
	movi r0, __func_1
	store [bp-44], r0
	load r0, [bp+68]
	store [bp-40], r0
	push r0
	load r0, [bp-44]
	callr r0
	addi sp, sp, 4
	load r0, [__global_0]
	store [bp-36], r0
	movi r0, 10
	store [bp-28], r0
	store [bp-32], r0
	load r1, [bp-36]
	storeb [r1], r0
	lea r0, [bp+68]
	store [bp-24], r0
	load r0, [bp+68]
	store [bp-16], r0
	movi r0, 1
	store [bp-12], r0
	load r0, [bp-16]
	load r1, [bp-12]
	add r0, r0, r1
	store [bp-20], r0
	load r1, [bp-24]
	store [r1], r0
	jmp __func_0L0
.label __func_0L1
	movi r0, __func_2
	store [bp-8], r0
	movi r0, __str_0
	store [bp-4], r0
	push r0
	load r0, [bp-8]
	callr r0
	addi sp, sp, 4
	movi r0, 0
	halt
	halt
.label __func_1
	push bp
	mov bp, sp
	load r0, [bp+12]
	store [bp-84], r0
	movi r0, 0
	store [bp-80], r0
	load r0, [bp-84]
	load r1, [bp-80]
	cmp r0, r1
	setl r0
	store [bp-88], r0
	cmpi r0, 0
	je __func_1L0
	load r0, [__global_0]
	store [bp-76], r0
	movi r0, 45
	store [bp-72], r0
	load r1, [bp-76]
	storeb [r1], r0
	lea r0, [bp+12]
	store [bp-68], r0
	load r0, [bp+12]
	store [bp-60], r0
	not r0
	addi r0, r0, 1
	store [bp-64], r0
	load r1, [bp-68]
	store [r1], r0
	jmp __func_1L1
.label __func_1L0
.label __func_1L1
	load r0, [bp+12]
	store [bp-52], r0
	movi r0, 10
	store [bp-48], r0
	load r0, [bp-52]
	load r1, [bp-48]
	cmp r0, r1
	setge r0
	store [bp-56], r0
	cmpi r0, 0
	je __func_1L2
	movi r0, __func_1
	store [bp-44], r0
	load r0, [bp+12]
	store [bp-36], r0
	movi r0, 10
	store [bp-32], r0
	load r0, [bp-36]
	load r1, [bp-32]
	div r0, r0, r1
	store [bp-40], r0
	push r0
	load r0, [bp-44]
	callr r0
	addi sp, sp, 4
	jmp __func_1L3
.label __func_1L2
.label __func_1L3
	load r0, [__global_0]
	store [bp-28], r0
	movi r0, 48
	store [bp-16], r0
	load r0, [bp+12]
	store [bp-8], r0
	movi r0, 10
	store [bp-4], r0
	load r0, [bp-8]
	load r1, [bp-4]
	mod r0, r0, r1
	store [bp-12], r0
	load r0, [bp-16]
	load r1, [bp-12]
	add r0, r0, r1
	store [bp-20], r0
	store [bp-24], r0
	load r1, [bp-28]
	storeb [r1], r0
.label __func_1_exit
	mov sp, bp
	pop bp
	ret
.label __func_2
	push bp
	mov bp, sp
.label __func_2L0
	load r0, [bp+12]
	store [bp-48], r0
	loadb r0, [r0]
	store [bp-52], r0
	movi r0, 0
	store [bp-40], r0
	store [bp-44], r0
	load r0, [bp-52]
	load r1, [bp-44]
	cmp r0, r1
	setne r0
	store [bp-56], r0
	cmpi r0, 0
	je __func_2L1
	load r0, [__global_0]
	store [bp-36], r0
	load r0, [bp+12]
	store [bp-28], r0
	loadb r0, [r0]
	store [bp-32], r0
	load r1, [bp-36]
	storeb [r1], r0
	lea r0, [bp+12]
	store [bp-24], r0
	load r0, [bp+12]
	store [bp-16], r0
	movi r0, 1
	store [bp-12], r0
	movi r0, 1
	store [bp-8], r0
	load r0, [bp-12]
	load r1, [bp-8]
	mul r0, r0, r1
	store [bp-4], r0
	load r0, [bp-16]
	load r1, [bp-4]
	add r0, r0, r1
	store [bp-20], r0
	load r1, [bp-24]
	store [r1], r0
	jmp __func_2L0
.label __func_2L1
	movi r0, 0
	jmp __func_2_exit
.label __func_2_exit
	mov sp, bp
	pop bp
	ret
.label __str_0 .data "stringy
", 0
.label __global_0 .word 2064384
