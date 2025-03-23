	.text
	.file	"wanco_aot"
	.globl	aot_main                        # -- Begin function aot_main
	.p2align	4, 0x90
	.type	aot_main,@function
aot_main:                               # @aot_main
	.cfi_startproc
# %bb.0:                                # %entry
	pushq	%rax
	.cfi_def_cfa_offset 16
	movl	12(%rdi), %eax
	callq	func_3@PLT
	popq	%rax
	.cfi_def_cfa_offset 8
	retq
.Lfunc_end0:
	.size	aot_main, .Lfunc_end0-aot_main
	.cfi_endproc
                                        # -- End function
	.globl	func_2                          # -- Begin function func_2
	.p2align	4, 0x90
	.type	func_2,@function
func_2:                                 # @func_2
	.cfi_startproc
# %bb.0:                                # %entry
	pushq	%rbp
	.cfi_def_cfa_offset 16
	.cfi_offset %rbp, -16
	movq	%rsp, %rbp
	.cfi_def_cfa_register %rbp
	pushq	%r14
	pushq	%rbx
	subq	$16, %rsp
	.cfi_offset %rbx, -32
	.cfi_offset %r14, -24
	movq	%rdi, %rbx
	movl	%esi, -20(%rbp)
	movl	%edx, -24(%rbp)
	movl	12(%rdi), %eax
	cmpl	$3, %eax
	jne	.LBB1_4
# %bb.1:                                # %restore.dispatch
	movq	%rbx, %rdi
	callq	get_pc_from_frame@PLT
	movq	%rbx, %rdi
	cmpl	$2, %eax
	je	.LBB1_10
# %bb.2:                                # %restore.dispatch
	cmpl	$5, %eax
	jne	.LBB1_9
# %bb.3:                                # %restore_op_5.start
	callq	pop_front_local_i32@PLT
	movl	%eax, %r14d
	movq	%rbx, %rdi
	callq	pop_front_local_i32@PLT
	movl	%r14d, -20(%rbp)
	movl	%eax, -24(%rbp)
	movq	%rbx, %rdi
	callq	pop_front_frame@PLT
	xorl	%esi, %esi
	jmp	.LBB1_7
.LBB1_4:                                # %main
	movq	28(%rbx), %rax
	movl	(%rax), %eax
.Ltmp0:
	jmp	.LBB1_5
	nopw	8(%rax,%rax)
.LBB1_10:                               # %restore_op_2.start
	callq	pop_front_local_i32@PLT
	movl	%eax, %r14d
	movq	%rbx, %rdi
	callq	pop_front_local_i32@PLT
	movl	%r14d, -20(%rbp)
	movl	%eax, -24(%rbp)
	movq	%rbx, %rdi
	callq	pop_front_frame@PLT
	xorl	%esi, %esi
	jmp	.LBB1_6
.LBB1_9:                                # %restore_op_7.start
	callq	pop_front_local_i32@PLT
	movl	%eax, %r14d
	movq	%rbx, %rdi
	callq	pop_front_local_i32@PLT
	movl	%r14d, -20(%rbp)
	movl	%eax, -24(%rbp)
	movq	%rbx, %rdi
	callq	pop_front_frame@PLT
	xorl	%esi, %esi
	jmp	.LBB1_8
.LBB1_5:                                # %loop.body
	movq	28(%rbx), %rax
	movl	(%rax), %eax
.Ltmp1:
	movl	-20(%rbp), %esi
	nopl	8(%rax,%rax)
.LBB1_6:                                # %non_leaf_op_2_restore.end
	movq	%rbx, %rdi
	callq	print_i32@PLT
.Ltmp2:
	movq	(%rbx), %rax
	movl	5(%rax), %esi
	xchgw	%ax, %ax
.LBB1_7:                                # %non_leaf_op_5_restore.end
	movq	%rbx, %rdi
	callq	print_i32@PLT
.Ltmp3:
	movl	$1000, %esi                     # imm = 0x3E8
	nopl	(%rax)
.LBB1_8:                                # %non_leaf_op_7_restore.end
	movq	%rbx, %rdi
	callq	sleep_msec@PLT
.Ltmp4:
	movl	-24(%rbp), %eax
	addl	%eax, -20(%rbp)
	jmp	.LBB1_5
.Lfunc_end1:
	.size	func_2, .Lfunc_end1-func_2
	.cfi_endproc
                                        # -- End function
	.globl	func_3                          # -- Begin function func_3
	.p2align	4, 0x90
	.type	func_3,@function
func_3:                                 # @func_3
	.cfi_startproc
# %bb.0:                                # %entry
	pushq	%rbp
	.cfi_def_cfa_offset 16
	.cfi_offset %rbp, -16
	movq	%rsp, %rbp
	.cfi_def_cfa_register %rbp
	pushq	%rbx
	pushq	%rax
	.cfi_offset %rbx, -24
	movq	%rdi, %rbx
	movl	12(%rdi), %eax
	cmpl	$3, %eax
	jne	.LBB2_2
# %bb.1:                                # %restore.dispatch
	movq	%rbx, %rdi
	callq	get_pc_from_frame@PLT
	movq	%rbx, %rdi
	callq	pop_front_frame@PLT
	xorl	%edx, %edx
	jmp	.LBB2_3
.LBB2_2:                                # %main
	movq	28(%rbx), %rax
	movl	(%rax), %eax
.Ltmp5:
	movq	(%rbx), %rax
	movl	$10, 5(%rax)
	movl	$1, %edx
.LBB2_3:                                # %non_leaf_op_5_restore.end
	movq	%rbx, %rdi
	xorl	%esi, %esi
	callq	func_2@PLT
.Ltmp6:
	addq	$8, %rsp
	popq	%rbx
	popq	%rbp
	.cfi_def_cfa %rsp, 8
	retq
	nop
.Lfunc_end2:
	.size	func_3, .Lfunc_end2-func_3
	.cfi_endproc
                                        # -- End function
	.globl	store_globals                   # -- Begin function store_globals
	.p2align	4, 0x90
	.type	store_globals,@function
store_globals:                          # @store_globals
	.cfi_startproc
# %bb.0:                                # %entry
	retq
.Lfunc_end3:
	.size	store_globals, .Lfunc_end3-store_globals
	.cfi_endproc
                                        # -- End function
	.globl	store_table                     # -- Begin function store_table
	.p2align	4, 0x90
	.type	store_table,@function
store_table:                            # @store_table
	.cfi_startproc
# %bb.0:                                # %entry
	retq
.Lfunc_end4:
	.size	store_table, .Lfunc_end4-store_table
	.cfi_endproc
                                        # -- End function
	.type	INIT_MEMORY_SIZE,@object        # @INIT_MEMORY_SIZE
	.section	.rodata,"a",@progbits
	.globl	INIT_MEMORY_SIZE
	.p2align	2, 0x0
INIT_MEMORY_SIZE:
	.long	1                               # 0x1
	.size	INIT_MEMORY_SIZE, 4

	.section	.llvm_stackmaps,"a",@progbits
__LLVM_StackMaps:
	.byte	3
	.byte	0
	.short	0
	.long	2
	.long	0
	.long	7
	.quad	func_2
	.quad	40
	.quad	5
	.quad	func_3
	.quad	24
	.quad	2
	.quad	12884901887
	.long	.Ltmp0-func_2
	.short	0
	.short	5
	.byte	4
	.byte	0
	.short	8
	.short	0
	.short	0
	.long	2
	.byte	4
	.byte	0
	.short	8
	.short	0
	.short	0
	.long	0
	.byte	2
	.byte	0
	.short	8
	.short	6
	.short	0
	.long	-20
	.byte	4
	.byte	0
	.short	8
	.short	0
	.short	0
	.long	0
	.byte	2
	.byte	0
	.short	8
	.short	6
	.short	0
	.long	-24
	.p2align	3, 0x0
	.short	0
	.short	0
	.p2align	3, 0x0
	.quad	8589934592
	.long	.Ltmp1-func_2
	.short	0
	.short	5
	.byte	4
	.byte	0
	.short	8
	.short	0
	.short	0
	.long	2
	.byte	4
	.byte	0
	.short	8
	.short	0
	.short	0
	.long	0
	.byte	2
	.byte	0
	.short	8
	.short	6
	.short	0
	.long	-20
	.byte	4
	.byte	0
	.short	8
	.short	0
	.short	0
	.long	0
	.byte	2
	.byte	0
	.short	8
	.short	6
	.short	0
	.long	-24
	.p2align	3, 0x0
	.short	0
	.short	0
	.p2align	3, 0x0
	.quad	8589934594
	.long	.Ltmp2-func_2
	.short	0
	.short	5
	.byte	4
	.byte	0
	.short	8
	.short	0
	.short	0
	.long	2
	.byte	4
	.byte	0
	.short	8
	.short	0
	.short	0
	.long	0
	.byte	2
	.byte	0
	.short	8
	.short	6
	.short	0
	.long	-20
	.byte	4
	.byte	0
	.short	8
	.short	0
	.short	0
	.long	0
	.byte	2
	.byte	0
	.short	8
	.short	6
	.short	0
	.long	-24
	.p2align	3, 0x0
	.short	0
	.short	0
	.p2align	3, 0x0
	.quad	8589934597
	.long	.Ltmp3-func_2
	.short	0
	.short	5
	.byte	4
	.byte	0
	.short	8
	.short	0
	.short	0
	.long	2
	.byte	4
	.byte	0
	.short	8
	.short	0
	.short	0
	.long	0
	.byte	2
	.byte	0
	.short	8
	.short	6
	.short	0
	.long	-20
	.byte	4
	.byte	0
	.short	8
	.short	0
	.short	0
	.long	0
	.byte	2
	.byte	0
	.short	8
	.short	6
	.short	0
	.long	-24
	.p2align	3, 0x0
	.short	0
	.short	0
	.p2align	3, 0x0
	.quad	8589934599
	.long	.Ltmp4-func_2
	.short	0
	.short	5
	.byte	4
	.byte	0
	.short	8
	.short	0
	.short	0
	.long	2
	.byte	4
	.byte	0
	.short	8
	.short	0
	.short	0
	.long	0
	.byte	2
	.byte	0
	.short	8
	.short	6
	.short	0
	.long	-20
	.byte	4
	.byte	0
	.short	8
	.short	0
	.short	0
	.long	0
	.byte	2
	.byte	0
	.short	8
	.short	6
	.short	0
	.long	-24
	.p2align	3, 0x0
	.short	0
	.short	0
	.p2align	3, 0x0
	.quad	17179869183
	.long	.Ltmp5-func_3
	.short	0
	.short	1
	.byte	4
	.byte	0
	.short	8
	.short	0
	.short	0
	.long	0
	.p2align	3, 0x0
	.short	0
	.short	0
	.p2align	3, 0x0
	.quad	12884901893
	.long	.Ltmp6-func_3
	.short	0
	.short	1
	.byte	4
	.byte	0
	.short	8
	.short	0
	.short	0
	.long	0
	.p2align	3, 0x0
	.short	0
	.short	0
	.p2align	3, 0x0

	.section	".note.GNU-stack","",@progbits
