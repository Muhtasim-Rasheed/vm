.label _start
	movi r0, string
	push r0
	call puts
	pop r0
	jmp _start

.label advance_cursor
	load r0, [cursor_x]
	load r1, [cursor_y]
	addi r0, r0, 1

	cmpi r0, 80
	jl advance_cursor_end
	movi r0, 0
	addi r1, r1, 1

	cmpi r1, 25
	jl advance_cursor_end
	movi r1, 0
.label advance_cursor_end
	store [cursor_x], r0
	store [cursor_y], r1
	ret

.label putc
	load r0, [bp + 8]

	cmpi r0, 10
	je putc_newline

	load r1, [cursor_y]
	muli r1, r1, 80
	load r2, [cursor_x]
	add r1, r1, r2
	muli r1, r1, 2
	addi r1, r1, 0x001F0000

	load r2, [current_color]

	store [r1], r0
	store [r1 + 1], r2

	call advance_cursor
	jmp putc_end

.label putc_newline
	load r0, [cursor_y]
	addi r0, r0, 1
	store [cursor_y], r0

	movi r0, 0
	store [cursor_x], r0

.label putc_end
	ret

.label puts
	load r0, [bp + 8]
.label puts_loop
	load r1, [r0]
	cmpi r1, 0
	je puts_end
	push r0
	push r1
	call putc
	pop r1
	pop r0
	addi r0, r0, 1
	jmp puts_loop
.label puts_end
	ret

.label string .data "Hello, World! \0"
.label cursor_x .word 0
.label cursor_y .word 0
.label current_color .word 0xFF
