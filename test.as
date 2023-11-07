.global _start
.section .text
	_start:
		mov	$1, %rax
		mov	$1, %rdi
		lea	L_5c911ff473bdf180(%rip), %rsi
		mov	L_b6e839f855c0f9c2, %rdx
		syscall
		mov	$60, %rax
		xor	%rdi, %rdi
		syscall

.section .data
	L_5c911ff473bdf180:
		.byte 0x48, 0x65, 0x6C, 0x6C, 0x6F, 0x2C, 0x20, 0x77, 0x6F, 0x72, 0x6C, 0x64, 0x21
	L_b6e839f855c0f9c2:
		.quad 13


