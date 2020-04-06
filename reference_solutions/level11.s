// Copyright 2020, Verizon Media
// Licensed under the terms of the MIT license. See LICENSE file in project root for terms.

.syntax unified
.code 16

// R7 = USART2 (user)
movw r7, #0x4400
movt r7, #0x4000

// R8 = USART6 (cpld)
movw r8, #0x1400
movt r8, #0x4001

movs r0, r7

adr r1, hello
bl putstr

movs r10, #0
bruteforceloop:
	# Debug output
	movs r0, r7
	adr r1, trying
	bl putstr
	movs r0, #1
	movs r1, r10
	svc 0
	movs r0, r7
	adr r1, crlf
	bl putstr

	# Send out
	movs r0, r8
	movs r1, r10
	bl putchar
	bl cpld_is_slow
	movs r0, r8
	movs r1, r10, lsr 8
	bl putchar
	bl cpld_is_slow

	movs r0, r8
	bl getchar
	cmp r0, #'Y'
	beq was_correct

	# Increment
	adds r10, #1

	ubfx r9, r10, #0, #4
	cmp r9, #10
	it eq
	addseq r10, #0x6

	ubfx r9, r10, #4, #4
	cmp r9, #10
	it eq
	addseq r10, #0x60

	ubfx r9, r10, #8, #4
	cmp r9, #10
	it eq
	addseq r10, #0x600

	ubfx r9, r10, #12, #4
	cmp r9, #10
	it eq
	addseq r10, #0x6000

	cmp r10, #0x10000
	bne bruteforceloop

movs r0, #0
svc 0

was_correct:
	movs r0, r7
	adr r1, correct
	bl putstr
	movs r0, #0
	svc 0

cpld_is_slow:
	movw r0, #1000
delayloop:
	subs r0, #1
	bne delayloop
	bx lr

# r0 = uart
# r1 = char
putchar:
	ldr r2, [r0]
	tst r2, 0x80
	beq putchar
	strb r1, [r0, 4]
	bx lr

# r0 = uart
getchar:
	ldr r1, [r0]
	tst r1, 0x20
	beq getchar
	ldrb r0, [r0, 4]
	bx lr

# r0 = uart
# r1 = string
putstr:
	push {r4, lr}
	movs r4, r1
loop:
	ldrb r1, [r4]
	orrs r1, r1
	it eq
	popeq {r4, pc}
	bl putchar
	adds r4, #1
	b loop

hello: .asciz "Hello World\r\nThis is the CPLD pin cracker payload.\r\n\r\n"
trying: .asciz "Trying "
crlf: .asciz "\r\n"
correct: .asciz "This guess was correct, but the digits need to be entered in reverse.\r\n"
