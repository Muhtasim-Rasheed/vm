.label __func_4
	push bp
	mov bp, sp
	movi r0, __global_6
	store [bp-52], r0
	movi r0, 20
	store [bp-48], r0
	load r1, [bp-52]
	store [r1], r0
.label __func_4L0
	movi r0, 1
	store [bp-44], r0
	cmpi r0, 0
	je __func_4L1
	movi r0, __func_0
	store [bp-40], r0
	load r0, [__global_7]
	store [bp-36], r0
	push r0
	load r0, [bp-40]
	callr r0
	addi sp, sp, 4
	movi r0, __global_5
	store [bp-32], r0
	load r0, [__global_5]
	store [bp-12], r0
	store [bp-16], r0
	movi r0, 41
	store [bp-8], r0
	load r0, [bp-16]
	load r1, [bp-8]
	mul r0, r0, r1
	store [bp-20], r0
	movi r0, 23
	store [bp-4], r0
	load r0, [bp-20]
	load r1, [bp-4]
	add r0, r0, r1
	store [bp-24], r0
	store [bp-28], r0
	load r1, [bp-32]
	storeb [r1], r0
	jmp __func_4L0
.label __func_4L1
	movi r0, 0
	halt
	halt
.label __func_0
	push bp
	mov bp, sp
	movi r0, __global_3
	store [bp-60], r0
	load r0, [__global_6]
	store [bp-52], r0
	load r0, [__global_1]
	store [bp-48], r0
	load r0, [bp-52]
	load r1, [bp-48]
	mod r0, r0, r1
	store [bp-56], r0
	load r1, [bp-60]
	store [r1], r0
.label __func_0L0
	load r0, [bp+12]
	store [bp-40], r0
	loadb r0, [r0]
	store [bp-44], r0
	cmpi r0, 0
	je __func_0L1
	movi r0, __func_1
	store [bp-36], r0
	load r0, [bp+12]
	store [bp-28], r0
	loadb r0, [r0]
	store [bp-32], r0
	push r0
	load r0, [bp-36]
	callr r0
	addi sp, sp, 4
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
	jmp __func_0L0
.label __func_0L1
	movi r0, 0
	jmp __func_0_exit
.label __func_0_exit
	mov sp, bp
	pop bp
	ret
.label __func_1
	push bp
	mov bp, sp
	load r0, [bp+12]
	store [bp-92], r0
	movi r0, 10
	storeb [bp-88], r0
	load r0, [bp-92]
	load r1, [bp-88]
	cmp r0, r1
	sete r0
	store [bp-96], r0
	cmpi r0, 0
	je __func_1L0
	movi r0, __func_3
	store [bp-84], r0
	callr r0
	movi r0, 0
	jmp __func_1_exit
	jmp __func_1L1
.label __func_1L0
.label __func_1L1
	load r0, [__global_0]
	store [bp-76], r0
	load r0, [__global_3]
	store [bp-64], r0
	load r0, [__global_4]
	store [bp-56], r0
	load r0, [__global_1]
	store [bp-52], r0
	load r0, [bp-56]
	load r1, [bp-52]
	mul r0, r0, r1
	store [bp-60], r0
	load r0, [bp-64]
	load r1, [bp-60]
	add r0, r0, r1
	store [bp-68], r0
	movi r0, 2
	store [bp-48], r0
	load r0, [bp-68]
	load r1, [bp-48]
	mul r0, r0, r1
	store [bp-72], r0
	movi r0, 1
	store [bp-44], r0
	load r0, [bp-72]
	load r1, [bp-44]
	mul r0, r0, r1
	store [bp-40], r0
	load r0, [bp-76]
	load r1, [bp-40]
	add r0, r0, r1
	store [bp-80], r0
	store [bp+104], r0
	store [bp-36], r0
	load r0, [bp+12]
	store [bp-32], r0
	load r1, [bp-36]
	storeb [r1], r0
	load r0, [bp+104]
	store [bp-24], r0
	movi r0, 1
	store [bp-20], r0
	movi r0, 1
	store [bp-16], r0
	load r0, [bp-20]
	load r1, [bp-16]
	mul r0, r0, r1
	store [bp-12], r0
	load r0, [bp-24]
	load r1, [bp-12]
	add r0, r0, r1
	store [bp-28], r0
	load r0, [__global_5]
	store [bp-8], r0
	load r1, [bp-28]
	storeb [r1], r0
	movi r0, __func_2
	store [bp-4], r0
	callr r0
	movi r0, 0
	jmp __func_1_exit
.label __func_1_exit
	mov sp, bp
	pop bp
	ret
.label __func_2
	push bp
	mov bp, sp
	movi r0, __global_3
	store [bp-72], r0
	load r0, [__global_3]
	store [bp-64], r0
	movi r0, 1
	store [bp-60], r0
	load r0, [bp-64]
	load r1, [bp-60]
	add r0, r0, r1
	store [bp-68], r0
	load r1, [bp-72]
	store [r1], r0
	load r0, [__global_3]
	store [bp-52], r0
	load r0, [__global_1]
	store [bp-48], r0
	load r0, [bp-52]
	load r1, [bp-48]
	cmp r0, r1
	setl r0
	store [bp-56], r0
	cmpi r0, 0
	je __func_2L0
	movi r0, 0
	jmp __func_2_exit
	jmp __func_2L1
.label __func_2L0
.label __func_2L1
	movi r0, __global_4
	store [bp-44], r0
	load r0, [__global_4]
	store [bp-36], r0
	movi r0, 1
	store [bp-32], r0
	load r0, [bp-36]
	load r1, [bp-32]
	add r0, r0, r1
	store [bp-40], r0
	load r1, [bp-44]
	store [r1], r0
	movi r0, __global_3
	store [bp-28], r0
	movi r0, 0
	store [bp-24], r0
	load r1, [bp-28]
	store [r1], r0
	load r0, [__global_4]
	store [bp-16], r0
	load r0, [__global_2]
	store [bp-12], r0
	load r0, [bp-16]
	load r1, [bp-12]
	cmp r0, r1
	setl r0
	store [bp-20], r0
	cmpi r0, 0
	je __func_2L2
	movi r0, 0
	jmp __func_2_exit
	jmp __func_2L3
.label __func_2L2
.label __func_2L3
	movi r0, __global_4
	store [bp-8], r0
	movi r0, 0
	store [bp-4], r0
	load r1, [bp-8]
	store [r1], r0
	movi r0, 0
	jmp __func_2_exit
.label __func_2_exit
	mov sp, bp
	pop bp
	ret
.label __func_3
	push bp
	mov bp, sp
	movi r0, __global_3
	store [bp-52], r0
	load r0, [__global_6]
	store [bp-44], r0
	load r0, [__global_1]
	store [bp-40], r0
	load r0, [bp-44]
	load r1, [bp-40]
	mod r0, r0, r1
	store [bp-48], r0
	load r1, [bp-52]
	store [r1], r0
	movi r0, __global_4
	store [bp-36], r0
	load r0, [__global_4]
	store [bp-28], r0
	movi r0, 1
	store [bp-24], r0
	load r0, [bp-28]
	load r1, [bp-24]
	add r0, r0, r1
	store [bp-32], r0
	load r1, [bp-36]
	store [r1], r0
	load r0, [__global_4]
	store [bp-16], r0
	load r0, [__global_2]
	store [bp-12], r0
	load r0, [bp-16]
	load r1, [bp-12]
	cmp r0, r1
	setl r0
	store [bp-20], r0
	cmpi r0, 0
	je __func_3L0
	movi r0, 0
	jmp __func_3_exit
	jmp __func_3L1
.label __func_3L0
.label __func_3L1
	movi r0, __global_4
	store [bp-8], r0
	movi r0, 0
	store [bp-4], r0
	load r1, [bp-8]
	store [r1], r0
	movi r0, 0
	jmp __func_3_exit
.label __func_3_exit
	mov sp, bp
	pop bp
	ret
.label __str_0 .data "Hello, World!
", 0
.label __global_0 .word 2031616
.label __global_1 .word 80
.label __global_2 .word 25
.label __global_3 .word 0
.label __global_4 .word 0
.label __global_5 .data 255
.label __global_6 .word 0
.label __global_7 .word __str_0
