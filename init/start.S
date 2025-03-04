.globl _start
.globl _syscall

.text

_start:
	call main

_syscall:
	movq %rdi, %rax         # syscall number
	movq %rsi, %rdi         # arg0
	movq %rdx, %rsi         # arg1
	movq %rcx, %rdx         # arg2
	movq %r8, %r10          # arg3
	movq %r9, %r8           # arg4
	movq 8(%rsp), %r9       # arg5
	syscall
	ret
