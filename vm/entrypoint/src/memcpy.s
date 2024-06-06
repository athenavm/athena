// This is musl-libc commit 0784374d561435f7c787a555aeab8ede699ed298
//
// src/string/memcpy.c
//
// This was compiled into assembly with:
//
// riscv64-unknown-elf-gcc -march=rv32em -mabi=ilp32e -O3 -nostdlib -fno-builtin -funroll-loops -I../../obj/include -I../../include -S memcpy.c -o memcpy.s
//
// and labels manually updated to not conflict.
//
// musl as a whole is licensed under the following standard MIT license:
//
// ----------------------------------------------------------------------
// Copyright © 2005-2020 Rich Felker, et al.
//
// Permission is hereby granted, free of charge, to any person obtaining
// a copy of this software and associated documentation files (the
// "Software"), to deal in the Software without restriction, including
// without limitation the rights to use, copy, modify, merge, publish,
// distribute, sublicense, and/or sell copies of the Software, and to
// permit persons to whom the Software is furnished to do so, subject to
// the following conditions:
//
// The above copyright notice and this permission notice shall be
// included in all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
// EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
// MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT.
// IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY
// CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT,
// TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE
// SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
// ----------------------------------------------------------------------
//
// Authors/contributors include:
//
// A. Wilcox
// Ada Worcester
// Alex Dowad
// Alex Suykov
// Alexander Monakov
// Andre McCurdy
// Andrew Kelley
// Anthony G. Basile
// Aric Belsito
// Arvid Picciani
// Bartosz Brachaczek
// Benjamin Peterson
// Bobby Bingham
// Boris Brezillon
// Brent Cook
// Chris Spiegel
// Clément Vasseur
// Daniel Micay
// Daniel Sabogal
// Daurnimator
// David Carlier
// David Edelsohn
// Denys Vlasenko
// Dmitry Ivanov
// Dmitry V. Levin
// Drew DeVault
// Emil Renner Berthing
// Fangrui Song
// Felix Fietkau
// Felix Janda
// Gianluca Anzolin
// Hauke Mehrtens
// He X
// Hiltjo Posthuma
// Isaac Dunham
// Jaydeep Patil
// Jens Gustedt
// Jeremy Huntwork
// Jo-Philipp Wich
// Joakim Sindholt
// John Spencer
// Julien Ramseier
// Justin Cormack
// Kaarle Ritvanen
// Khem Raj
// Kylie McClain
// Leah Neukirchen
// Luca Barbato
// Luka Perkov
// M Farkas-Dyck (Strake)
// Mahesh Bodapati
// Markus Wichmann
// Masanori Ogino
// Michael Clark
// Michael Forney
// Mikhail Kremnyov
// Natanael Copa
// Nicholas J. Kain
// orc
// Pascal Cuoq
// Patrick Oppenlander
// Petr Hosek
// Petr Skocik
// Pierre Carrier
// Reini Urban
// Rich Felker
// Richard Pennington
// Ryan Fairfax
// Samuel Holland
// Segev Finer
// Shiz
// sin
// Solar Designer
// Stefan Kristiansson
// Stefan O'Rear
// Szabolcs Nagy
// Timo Teräs
// Trutz Behn
// Valentin Ochs
// Will Dietz
// William Haddon
// William Pitcock
//
// Portions of this software are derived from third-party works licensed
// under terms compatible with the above MIT license:
//
// The TRE regular expression implementation (src/regex/reg* and
// src/regex/tre*) is Copyright © 2001-2008 Ville Laurikari and licensed
// under a 2-clause BSD license (license text in the source files). The
// included version has been heavily modified by Rich Felker in 2012, in
// the interests of size, simplicity, and namespace cleanliness.
//
// Much of the math library code (src/math/* and src/complex/*) is
// Copyright © 1993,2004 Sun Microsystems or
// Copyright © 2003-2011 David Schultz or
// Copyright © 2003-2009 Steven G. Kargl or
// Copyright © 2003-2009 Bruce D. Evans or
// Copyright © 2008 Stephen L. Moshier or
// Copyright © 2017-2018 Arm Limited
// and labelled as such in comments in the individual source files. All
// have been licensed under extremely permissive terms.
//
// The ARM memcpy code (src/string/arm/memcpy.S) is Copyright © 2008
// The Android Open Source Project and is licensed under a two-clause BSD
// license. It was taken from Bionic libc, used on Android.
//
// The AArch64 memcpy and memset code (src/string/aarch64/*) are
// Copyright © 1999-2019, Arm Limited.
//
// The implementation of DES for crypt (src/crypt/crypt_des.c) is
// Copyright © 1994 David Burren. It is licensed under a BSD license.
//
// The implementation of blowfish crypt (src/crypt/crypt_blowfish.c) was
// originally written by Solar Designer and placed into the public
// domain. The code also comes with a fallback permissive license for use
// in jurisdictions that may not recognize the public domain.
//
// The smoothsort implementation (src/stdlib/qsort.c) is Copyright © 2011
// Valentin Ochs and is licensed under an MIT-style license.
//
// The x86_64 port was written by Nicholas J. Kain and is licensed under
// the standard MIT terms.
//
// The mips and microblaze ports were originally written by Richard
// Pennington for use in the ellcc project. The original code was adapted
// by Rich Felker for build system and code conventions during upstream
// integration. It is licensed under the standard MIT terms.
//
// The mips64 port was contributed by Imagination Technologies and is
// licensed under the standard MIT terms.
//
// The powerpc port was also originally written by Richard Pennington,
// and later supplemented and integrated by John Spencer. It is licensed
// under the standard MIT terms.
//
// All other files which have no copyright comments are original works
// produced specifically for use as part of this library, written either
// by Rich Felker, the main author of the library, or by one or more
// contibutors listed above. Details on authorship of individual files
// can be found in the git version control history of the project. The
// omission of copyright and license comments in each file is in the
// interest of source tree size.
//
// In addition, permission is hereby granted for all public header files
// (include/* and arch/* /bits/* ) and crt files intended to be linked into
// applications (crt/*, ldso/dlstart.c, and arch/* /crt_arch.h) to omit
// the copyright notice and permission notice otherwise required by the
// license, and to use these files without any requirement of
// attribution. These files include substantial contributions from:
//
// Bobby Bingham
// John Spencer
// Nicholas J. Kain
// Rich Felker
// Richard Pennington
// Stefan Kristiansson
// Szabolcs Nagy
//
// all of whom have explicitly granted such permission.
//
// This file previously contained text expressing a belief that most of
// the files covered by the above exception were sufficiently trivial not
// to be subject to copyright, resulting in confusion over whether it
// negated the permissions granted in the license. In the spirit of
// permissive licensing, and of not having licensing issues being an
// obstacle to adoption, that text has been removed.
	.file	"memcpy.c"
	.option nopic
	.attribute arch, "rv32em"
	.attribute unaligned_access, 0
	.attribute stack_align, 4
	.text
	.align	2
	.globl	memcpy
	.type	memcpy, @function
memcpy:
	addi	sp,sp,-44
	sw	s1,36(sp)
	sw	s0,40(sp)
	andi	a5,a1,3
	sw	a0,4(sp)
	mv	s1,a2
	beq	a5,zero,.LBBmemcpyL24
	beq	a2,zero,.LBBmemcpyL12
	andi	a4,a2,7
	mv	a5,a0
	addi	a2,a2,-1
	beq	a4,zero,.LBBmemcpyL4
	li	a3,1
	beq	a4,a3,.LBBmemcpyL85
	li	t0,2
	beq	a4,t0,.LBBmemcpyL86
	li	t1,3
	beq	a4,t1,.LBBmemcpyL87
	li	t2,4
	beq	a4,t2,.LBBmemcpyL88
	li	s0,5
	beq	a4,s0,.LBBmemcpyL89
	li	a3,6
	bne	a4,a3,.LBBmemcpyL145
.LBBmemcpyL90:
	lbu	a2,0(a1)
	addi	a1,a1,1
	andi	t0,a1,3
	sb	a2,0(a5)
	addi	s1,s1,-1
	addi	a5,a5,1
	beq	t0,zero,.LBBmemcpyL2
.LBBmemcpyL89:
	lbu	t1,0(a1)
	addi	a1,a1,1
	andi	t2,a1,3
	sb	t1,0(a5)
	addi	s1,s1,-1
	addi	a5,a5,1
	beq	t2,zero,.LBBmemcpyL2
.LBBmemcpyL88:
	lbu	s0,0(a1)
	addi	a1,a1,1
	andi	a3,a1,3
	sb	s0,0(a5)
	addi	s1,s1,-1
	addi	a5,a5,1
	beq	a3,zero,.LBBmemcpyL2
.LBBmemcpyL87:
	lbu	a0,0(a1)
	addi	a1,a1,1
	andi	a4,a1,3
	sb	a0,0(a5)
	addi	s1,s1,-1
	addi	a5,a5,1
	beq	a4,zero,.LBBmemcpyL2
.LBBmemcpyL86:
	lbu	a2,0(a1)
	addi	a1,a1,1
	andi	t0,a1,3
	sb	a2,0(a5)
	addi	s1,s1,-1
	addi	a5,a5,1
	beq	t0,zero,.LBBmemcpyL2
.LBBmemcpyL85:
	lbu	t1,0(a1)
	addi	a1,a1,1
	andi	t2,a1,3
	sb	t1,0(a5)
	addi	s1,s1,-1
	addi	a5,a5,1
	beq	t2,zero,.LBBmemcpyL2
.LBBmemcpyL142:
	beq	s1,zero,.LBBmemcpyL12
.LBBmemcpyL4:
	lbu	s0,0(a1)
	addi	a1,a1,1
	addi	a5,a5,1
	addi	s1,s1,-1
	andi	a0,a1,3
	sb	s0,-1(a5)
	mv	a3,a1
	mv	a4,a5
	mv	a2,s1
	beq	a0,zero,.LBBmemcpyL2
	lbu	t0,0(a1)
	addi	a1,a1,1
	andi	t1,a1,3
	sb	t0,0(a5)
	addi	s1,s1,-1
	addi	a5,a5,1
	beq	t1,zero,.LBBmemcpyL2
	lbu	a5,1(a3)
	addi	a1,a3,2
	andi	t2,a1,3
	sb	a5,1(a4)
	addi	s1,a2,-2
	addi	a5,a4,2
	beq	t2,zero,.LBBmemcpyL2
	lbu	s1,2(a3)
	addi	a1,a3,3
	andi	s0,a1,3
	sb	s1,2(a4)
	addi	a5,a4,3
	addi	s1,a2,-3
	beq	s0,zero,.LBBmemcpyL2
	lbu	t0,3(a3)
	addi	a1,a3,4
	andi	a0,a1,3
	sb	t0,3(a4)
	addi	a5,a4,4
	addi	s1,a2,-4
	beq	a0,zero,.LBBmemcpyL2
	lbu	t1,4(a3)
	addi	a1,a3,5
	andi	t2,a1,3
	sb	t1,4(a4)
	addi	a5,a4,5
	addi	s1,a2,-5
	beq	t2,zero,.LBBmemcpyL2
	lbu	a5,5(a3)
	addi	a1,a3,6
	andi	s0,a1,3
	sb	a5,5(a4)
	addi	s1,a2,-6
	addi	a5,a4,6
	beq	s0,zero,.LBBmemcpyL2
	lbu	s1,6(a3)
	addi	a1,a3,7
	andi	a3,a1,3
	sb	s1,6(a4)
	addi	a5,a4,7
	addi	s1,a2,-7
	bne	a3,zero,.LBBmemcpyL142
.LBBmemcpyL2:
	andi	t0,a5,3
	beq	t0,zero,.LBBmemcpyL146
	li	a4,31
	bleu	s1,a4,.LBBmemcpyL13
	lw	a0,0(a1)
	li	a2,2
	lbu	s0,0(a1)
	sw	a0,0(sp)
	beq	t0,a2,.LBBmemcpyL14
	li	t2,3
	beq	t0,t2,.LBBmemcpyL15
	addi	a3,s1,-20
	andi	t2,a3,-16
	addi	a2,t2,19
	add	a4,a5,a2
	addi	t2,a5,3
	sub	t0,a4,t2
	lbu	a0,1(a1)
	lbu	t1,2(a1)
	addi	a2,t0,-16
	sw	a4,16(sp)
	addi	t0,a1,3
	srli	a4,a2,4
	srli	a3,a3,4
	sb	s0,0(a5)
	sb	a0,1(a5)
	sw	t2,24(sp)
	sw	t0,20(sp)
	sb	t1,2(a5)
	andi	s0,a4,1
	sw	a3,28(sp)
	mv	a0,t2
	bne	s0,zero,.LBBmemcpyL130
	lw	a0,0(sp)
	lw	t1,1(t0)
	lw	a3,9(t0)
	lw	t2,5(t0)
	srli	s0,a0,24
	lw	a0,13(t0)
	slli	a2,t1,8
	srli	a4,t1,24
	slli	t0,t2,8
	slli	t1,a3,8
	srli	t2,t2,24
	or	s0,s0,a2
	sw	a0,0(sp)
	or	a2,a4,t0
	slli	a0,a0,8
	or	a4,t2,t1
	srli	a3,a3,24
	lw	t2,24(sp)
	or	t0,a3,a0
	addi	a0,a5,19
	lw	a5,16(sp)
	sw	t0,12(t2)
	sw	s0,0(t2)
	sw	a2,4(t2)
	sw	a4,8(t2)
	addi	t0,a1,19
	beq	a0,a5,.LBBmemcpyL124
.LBBmemcpyL130:
	sw	s1,32(sp)
.LBBmemcpyL16:
	lw	t1,5(t0)
	lw	a4,13(t0)
	lw	s1,0(sp)
	lw	a2,29(t0)
	lw	a1,1(t0)
	lw	a3,9(t0)
	srli	s0,s1,24
	slli	a5,a4,8
	slli	s1,t1,8
	sw	a2,0(sp)
	srli	a2,t1,24
	lw	t1,17(t0)
	sw	a5,12(sp)
	slli	t2,a1,8
	srli	a1,a1,24
	or	a1,a1,s1
	lw	s1,12(sp)
	sw	s0,8(sp)
	slli	a5,t1,8
	slli	s0,a3,8
	srli	a4,a4,24
	or	a2,a2,s0
	srli	a3,a3,24
	or	s0,a4,a5
	lw	a4,17(t0)
	lw	t1,8(sp)
	or	a3,a3,s1
	sw	s0,8(sp)
	lw	s1,21(t0)
	lw	s0,25(t0)
	srli	a5,a4,24
	or	t2,t1,t2
	srli	a4,s1,24
	slli	t1,s1,8
	sw	a5,12(sp)
	slli	s1,s0,8
	lw	a5,25(t0)
	lw	s0,0(sp)
	sw	a1,4(a0)
	lw	a1,12(sp)
	sw	t2,0(a0)
	lw	t2,8(sp)
	sw	a3,12(a0)
	srli	a5,a5,24
	or	a3,a4,s1
	slli	s0,s0,8
	lw	a4,16(sp)
	sw	a2,8(a0)
	or	a2,a1,t1
	or	t1,a5,s0
	sw	t2,16(a0)
	sw	a2,20(a0)
	sw	a3,24(a0)
	sw	t1,28(a0)
	addi	a0,a0,32
	addi	t0,t0,32
	bne	a0,a4,.LBBmemcpyL16
	lw	s1,32(sp)
.LBBmemcpyL124:
	lw	t0,28(sp)
	lw	a5,24(sp)
	lw	t2,20(sp)
	addi	a0,t0,1
	slli	s0,a0,4
	addi	s1,s1,-19
	slli	t0,t0,4
	sub	s1,s1,t0
	add	a5,a5,s0
	add	a1,t2,s0
.LBBmemcpyL13:
	andi	s0,s1,16
	andi	a0,s1,8
	andi	a3,s1,4
	andi	a4,s1,2
	andi	a2,s1,1
	beq	s0,zero,.LBBmemcpyL27
	lbu	s0,0(a1)
	lbu	t2,1(a1)
	lbu	t0,2(a1)
	lbu	s1,3(a1)
	sb	s0,0(a5)
	sb	t2,1(a5)
	lbu	s0,4(a1)
	lbu	t2,5(a1)
	sb	t0,2(a5)
	lbu	t0,6(a1)
	sb	s1,3(a5)
	sb	s0,4(a5)
	lbu	s1,7(a1)
	lbu	s0,8(a1)
	sb	t2,5(a5)
	sb	t0,6(a5)
	lbu	t2,9(a1)
	lbu	t0,10(a1)
	lbu	t1,15(a1)
	sb	s1,7(a5)
	sb	s0,8(a5)
	lbu	s1,11(a1)
	lbu	s0,12(a1)
	sb	t2,9(a5)
	sb	t0,10(a5)
	lbu	t2,13(a1)
	lbu	t0,14(a1)
	sb	s1,11(a5)
	sb	s0,12(a5)
	sb	t0,14(a5)
	sb	t2,13(a5)
	addi	a1,a1,16
	addi	t0,a5,16
	sb	t1,15(a5)
.LBBmemcpyL19:
	beq	a0,zero,.LBBmemcpyL28
	lbu	s0,0(a1)
	lbu	t2,1(a1)
	lbu	a0,3(a1)
	lbu	t1,2(a1)
	lbu	s1,4(a1)
	lbu	a5,7(a1)
	sb	s0,0(t0)
	sb	t2,1(t0)
	lbu	s0,5(a1)
	lbu	t2,6(a1)
	sb	a0,3(t0)
	sb	t1,2(t0)
	sb	s1,4(t0)
	sb	s0,5(t0)
	sb	t2,6(t0)
	addi	a1,a1,8
	addi	a0,t0,8
	sb	a5,7(t0)
.LBBmemcpyL20:
	beq	a3,zero,.LBBmemcpyL29
	lbu	s1,0(a1)
	lbu	t1,1(a1)
	lbu	a5,2(a1)
	lbu	a3,3(a1)
	sb	s1,0(a0)
	sb	t1,1(a0)
	sb	a5,2(a0)
	addi	a1,a1,4
	addi	t0,a0,4
	sb	a3,3(a0)
.LBBmemcpyL21:
	beq	a4,zero,.LBBmemcpyL30
	lbu	a0,0(a1)
	lbu	s0,1(a1)
	addi	a4,t0,2
	addi	a1,a1,2
	sb	a0,0(t0)
	sb	s0,1(t0)
.LBBmemcpyL22:
	beq	a2,zero,.LBBmemcpyL12
	lbu	a1,0(a1)
	sb	a1,0(a4)
.LBBmemcpyL12:
	lw	s0,40(sp)
	lw	a0,4(sp)
	lw	s1,36(sp)
	addi	sp,sp,44
	jr	ra
.LBBmemcpyL146:
	li	a2,15
	bleu	s1,a2,.LBBmemcpyL25
	addi	t2,s1,-16
	andi	t0,t2,-16
	addi	t2,t0,16
	addi	t1,t2,-16
	srli	a3,t1,4
	addi	a4,a3,1
	andi	s0,a4,3
	add	a3,a5,t2
	mv	a4,a1
	beq	s0,zero,.LBBmemcpyL7
	li	a0,1
	beq	s0,a0,.LBBmemcpyL91
	li	a2,2
	bne	s0,a2,.LBBmemcpyL147
.LBBmemcpyL92:
	lw	t0,4(a4)
	lw	t1,8(a4)
	lw	a2,12(a4)
	lw	a0,0(a4)
	sw	t0,4(a5)
	sw	t1,8(a5)
	sw	a0,0(a5)
	sw	a2,12(a5)
	addi	a4,a4,16
	addi	a5,a5,16
.LBBmemcpyL91:
	lw	s0,4(a4)
	lw	t0,8(a4)
	lw	t1,12(a4)
	lw	a2,0(a4)
	sw	s0,4(a5)
	sw	t0,8(a5)
	sw	a2,0(a5)
	sw	t1,12(a5)
	addi	a5,a5,16
	addi	a4,a4,16
	beq	a3,a5,.LBBmemcpyL123
.LBBmemcpyL7:
	lw	t0,4(a4)
	lw	s0,0(a4)
	lw	a0,12(a4)
	lw	a2,16(a4)
	sw	s0,0(a5)
	sw	t0,4(a5)
	lw	s0,24(a4)
	lw	t0,20(a4)
	lw	t1,8(a4)
	sw	a0,12(a5)
	sw	a2,16(a5)
	lw	a0,40(a4)
	lw	a2,36(a4)
	sw	t0,20(a5)
	sw	s0,24(a5)
	lw	t0,28(a4)
	lw	s0,32(a4)
	sw	t1,8(a5)
	sw	t0,28(a5)
	lw	t1,44(a4)
	lw	t0,48(a4)
	sw	s0,32(a5)
	sw	a2,36(a5)
	lw	s0,52(a4)
	lw	a2,56(a4)
	sw	a0,40(a5)
	lw	a0,60(a4)
	sw	t1,44(a5)
	sw	t0,48(a5)
	sw	s0,52(a5)
	sw	a2,56(a5)
	sw	a0,60(a5)
	addi	a5,a5,64
	addi	a4,a4,64
	bne	a3,a5,.LBBmemcpyL7
.LBBmemcpyL123:
	andi	s1,s1,15
	add	a1,a1,t2
.LBBmemcpyL6:
	andi	t2,s1,8
	andi	a4,s1,4
	andi	a5,s1,2
	andi	s1,s1,1
	beq	t2,zero,.LBBmemcpyL8
	lw	t1,4(a1)
	lw	t0,0(a1)
	addi	a3,a3,8
	addi	a1,a1,8
	sw	t1,-4(a3)
	sw	t0,-8(a3)
.LBBmemcpyL8:
	beq	a4,zero,.LBBmemcpyL9
	lw	s0,0(a1)
	addi	a3,a3,4
	addi	a1,a1,4
	sw	s0,-4(a3)
.LBBmemcpyL9:
	beq	a5,zero,.LBBmemcpyL26
	lbu	a0,0(a1)
	lbu	t2,1(a1)
	addi	a2,a3,2
	addi	a1,a1,2
	sb	a0,0(a3)
	sb	t2,1(a3)
.LBBmemcpyL10:
	beq	s1,zero,.LBBmemcpyL12
	lbu	a1,0(a1)
	lw	a0,4(sp)
	sb	a1,0(a2)
	lw	s0,40(sp)
	lw	s1,36(sp)
	addi	sp,sp,44
	jr	ra
.LBBmemcpyL15:
	addi	a3,s1,-20
	andi	t0,a3,-16
	addi	a2,t0,17
	add	t1,a5,a2
	addi	a0,a5,1
	sub	a4,t1,a0
	addi	t2,a4,-16
	srli	a2,t2,4
	addi	t0,a1,1
	srli	a3,a3,4
	sb	s0,0(a5)
	sw	a0,24(sp)
	sw	t1,16(sp)
	sw	t0,20(sp)
	andi	s0,a2,1
	sw	a3,28(sp)
	bne	s0,zero,.LBBmemcpyL131
	lw	a0,3(t0)
	lw	a4,11(t0)
	lw	t1,0(sp)
	lw	t2,7(t0)
	slli	a2,a0,24
	srli	s0,t1,8
	srli	a0,a0,8
	lw	t1,15(t0)
	slli	a3,a4,24
	slli	t0,t2,24
	srli	t2,t2,8
	or	s0,s0,a2
	or	a2,a0,t0
	or	a0,t2,a3
	lw	t2,24(sp)
	sw	t1,0(sp)
	srli	a4,a4,8
	slli	t1,t1,24
	sw	a0,8(t2)
	addi	a0,a5,17
	lw	a5,16(sp)
	or	t0,a4,t1
	sw	t0,12(t2)
	sw	s0,0(t2)
	sw	a2,4(t2)
	addi	t0,a1,17
	beq	a0,a5,.LBBmemcpyL125
.LBBmemcpyL131:
	sw	s1,32(sp)
.LBBmemcpyL18:
	lw	s1,3(t0)
	lw	t1,7(t0)
	lw	a4,15(t0)
	lw	a1,0(sp)
	lw	a2,31(t0)
	lw	a3,11(t0)
	srli	s0,a1,8
	slli	a5,a4,24
	sw	a2,0(sp)
	slli	t2,s1,24
	srli	a1,s1,8
	srli	a2,t1,8
	slli	s1,t1,24
	lw	t1,19(t0)
	sw	a5,12(sp)
	or	a1,a1,s1
	lw	s1,12(sp)
	sw	s0,8(sp)
	slli	a5,t1,24
	slli	s0,a3,24
	srli	a4,a4,8
	or	a2,a2,s0
	srli	a3,a3,8
	or	s0,a4,a5
	lw	a4,19(t0)
	lw	t1,8(sp)
	or	a3,a3,s1
	sw	s0,8(sp)
	lw	s1,23(t0)
	lw	s0,27(t0)
	srli	a5,a4,8
	or	t2,t1,t2
	srli	a4,s1,8
	slli	t1,s1,24
	sw	a5,12(sp)
	slli	s1,s0,24
	lw	a5,27(t0)
	lw	s0,0(sp)
	sw	a1,4(a0)
	lw	a1,12(sp)
	sw	t2,0(a0)
	lw	t2,8(sp)
	sw	a3,12(a0)
	srli	a5,a5,8
	or	a3,a4,s1
	slli	s0,s0,24
	lw	a4,16(sp)
	sw	a2,8(a0)
	or	a2,a1,t1
	or	t1,a5,s0
	sw	t2,16(a0)
	sw	a2,20(a0)
	sw	a3,24(a0)
	sw	t1,28(a0)
	addi	a0,a0,32
	addi	t0,t0,32
	bne	a0,a4,.LBBmemcpyL18
	lw	s1,32(sp)
.LBBmemcpyL125:
	lw	t0,28(sp)
	addi	s1,s1,-17
	addi	a0,t0,1
	slli	s0,a0,4
.LBBmemcpyL143:
	lw	a5,24(sp)
	lw	t2,20(sp)
	slli	t0,t0,4
	sub	s1,s1,t0
	add	a5,a5,s0
	add	a1,t2,s0
	j	.LBBmemcpyL13
.LBBmemcpyL145:
	lbu	a0,0(a1)
	addi	a1,a1,1
	andi	a4,a1,3
	sb	a0,0(a5)
	mv	s1,a2
	addi	a5,a5,1
	bne	a4,zero,.LBBmemcpyL90
	j	.LBBmemcpyL2
.LBBmemcpyL14:
	addi	t2,s1,-20
	andi	t1,t2,-16
	addi	a3,t1,18
	add	t0,a5,a3
	addi	t1,a5,2
	sub	a2,t0,t1
	lbu	a4,1(a1)
	addi	a0,a2,-16
	srli	a3,a0,4
	sw	t0,16(sp)
	srli	t2,t2,4
	addi	t0,a1,2
	sb	s0,0(a5)
	sw	t1,24(sp)
	sw	t0,20(sp)
	sb	a4,1(a5)
	andi	s0,a3,1
	sw	t2,28(sp)
	mv	a0,t1
	bne	s0,zero,.LBBmemcpyL132
	lw	t1,0(sp)
	lw	a0,2(t0)
	lw	a3,6(t0)
	lw	a4,10(t0)
	srli	s0,t1,16
	lw	t1,14(t0)
	slli	a2,a0,16
	slli	t0,a3,16
	srli	t2,a3,16
	srli	a0,a0,16
	slli	a3,a4,16
	or	s0,s0,a2
	or	a2,a0,t0
	or	a0,t2,a3
	lw	t2,24(sp)
	sw	t1,0(sp)
	srli	a4,a4,16
	slli	t1,t1,16
	or	t0,a4,t1
	sw	t0,12(t2)
	addi	t0,a1,18
	lw	a1,16(sp)
	sw	a0,8(t2)
	sw	s0,0(t2)
	sw	a2,4(t2)
	addi	a0,a5,18
	beq	a0,a1,.LBBmemcpyL126
.LBBmemcpyL132:
	sw	s1,32(sp)
.LBBmemcpyL17:
	lw	s1,2(t0)
	lw	t1,6(t0)
	lw	a4,14(t0)
	lw	a5,0(sp)
	lw	a2,30(t0)
	lw	a3,10(t0)
	srli	s0,a5,16
	sw	a2,0(sp)
	lw	a5,18(t0)
	slli	t2,s1,16
	srli	a1,s1,16
	srli	a2,t1,16
	slli	s1,t1,16
	slli	t1,a4,16
	sw	t1,12(sp)
	or	a1,a1,s1
	lw	s1,12(sp)
	sw	s0,8(sp)
	srli	a4,a4,16
	slli	s0,a3,16
	slli	a5,a5,16
	or	a2,a2,s0
	srli	a3,a3,16
	or	s0,a4,a5
	lw	a4,18(t0)
	lw	t1,8(sp)
	or	a3,a3,s1
	sw	s0,8(sp)
	lw	s1,22(t0)
	lw	s0,26(t0)
	srli	a5,a4,16
	or	t2,t1,t2
	srli	a4,s1,16
	slli	t1,s1,16
	sw	a5,12(sp)
	slli	s1,s0,16
	lw	a5,26(t0)
	lw	s0,0(sp)
	sw	a1,4(a0)
	lw	a1,12(sp)
	sw	t2,0(a0)
	lw	t2,8(sp)
	sw	a3,12(a0)
	srli	a5,a5,16
	or	a3,a4,s1
	slli	s0,s0,16
	lw	a4,16(sp)
	sw	a2,8(a0)
	or	a2,a1,t1
	or	t1,a5,s0
	sw	t2,16(a0)
	sw	a2,20(a0)
	sw	a3,24(a0)
	sw	t1,28(a0)
	addi	a0,a0,32
	addi	t0,t0,32
	bne	a0,a4,.LBBmemcpyL17
	lw	s1,32(sp)
.LBBmemcpyL126:
	lw	t0,28(sp)
	addi	s1,s1,-18
	addi	a0,t0,1
	slli	s0,a0,4
	j	.LBBmemcpyL143
.LBBmemcpyL30:
	mv	a4,t0
	j	.LBBmemcpyL22
.LBBmemcpyL28:
	mv	a0,t0
	j	.LBBmemcpyL20
.LBBmemcpyL24:
	mv	a5,a0
	j	.LBBmemcpyL2
.LBBmemcpyL147:
	lw	a4,12(a1)
	lw	t0,4(a1)
	lw	t1,8(a1)
	lw	s0,0(a1)
	sw	a4,12(a5)
	sw	t0,4(a5)
	sw	t1,8(a5)
	sw	s0,0(a5)
	addi	a4,a1,16
	addi	a5,a5,16
	j	.LBBmemcpyL92
.LBBmemcpyL25:
	mv	a3,a5
	j	.LBBmemcpyL6
.LBBmemcpyL29:
	mv	t0,a0
	j	.LBBmemcpyL21
.LBBmemcpyL27:
	mv	t0,a5
	j	.LBBmemcpyL19
.LBBmemcpyL26:
	mv	a2,a3
	j	.LBBmemcpyL10
	.size	memcpy, .-memcpy
	.ident	"GCC: () 10.2.0"
