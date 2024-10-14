// This is musl-libc commit 0784374d561435f7c787a555aeab8ede699ed298
//
// src/string/memcpy.c
//
// This was compiled into assembly with:
//
// clang-18 -target riscv32 -march=rv32em -mabi=ilp32e -O3 -nostdlib -fno-builtin -funroll-loops -S memcpy.c -o memcpy.s
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
	.text
	.attribute	4, 4
	.attribute	5, "rv32e2p0_m2p0"
	.file	"memcpy.c"
	.globl	memcpy                          # -- Begin function memcpy
	.p2align	2
	.type	memcpy,@function
memcpy:                                 # @memcpy
# %bb.0:
	andi	a3, a1, 3
	beqz	a3, .LBB0_16
# %bb.1:
	beqz	a2, .LBB0_5
# %bb.2:
	addi	a4, a1, 1
	li	a5, 1
	mv	a3, a0
.LBB0_3:                                # =>This Inner Loop Header: Depth=1
	lbu	t1, 0(a1)
	mv	t0, a2
	addi	a1, a1, 1
	sb	t1, 0(a3)
	addi	a3, a3, 1
	andi	t1, a4, 3
	addi	a2, a2, -1
	beqz	t1, .LBB0_6
# %bb.4:                                #   in Loop: Header=BB0_3 Depth=1
	addi	a4, a4, 1
	bne	t0, a5, .LBB0_3
	j	.LBB0_6
.LBB0_5:
	mv	a3, a0
.LBB0_6:
	andi	a5, a3, 3
	beqz	a5, .LBB0_17
.LBB0_7:
	li	a4, 32
	bgeu	a2, a4, .LBB0_11
# %bb.8:
	li	a4, 16
	bgeu	a2, a4, .LBB0_30
.LBB0_9:
	andi	a4, a2, 8
	bnez	a4, .LBB0_31
.LBB0_10:
	andi	a4, a2, 4
	bnez	a4, .LBB0_32
	j	.LBB0_33
.LBB0_11:
	lw	a4, 0(a1)
	li	t0, 3
	beq	a5, t0, .LBB0_24
# %bb.12:
	li	t0, 2
	bne	a5, t0, .LBB0_27
# %bb.13:
	sb	a4, 0(a3)
	srli	a5, a4, 8
	sb	a5, 1(a3)
	addi	a3, a3, 2
	addi	a2, a2, -2
	addi	a1, a1, 16
	li	a5, 17
.LBB0_14:                               # =>This Inner Loop Header: Depth=1
	lw	t0, -12(a1)
	srli	a4, a4, 16
	slli	t1, t0, 16
	lw	t2, -8(a1)
	or	a4, t1, a4
	sw	a4, 0(a3)
	srli	a4, t0, 16
	slli	t0, t2, 16
	lw	t1, -4(a1)
	or	a4, t0, a4
	sw	a4, 4(a3)
	srli	t0, t2, 16
	slli	t2, t1, 16
	lw	a4, 0(a1)
	or	t0, t2, t0
	sw	t0, 8(a3)
	srli	t0, t1, 16
	slli	t1, a4, 16
	or	t0, t1, t0
	sw	t0, 12(a3)
	addi	a3, a3, 16
	addi	a2, a2, -16
	addi	a1, a1, 16
	bltu	a5, a2, .LBB0_14
# %bb.15:
	addi	a1, a1, -14
	li	a4, 16
	bltu	a2, a4, .LBB0_9
	j	.LBB0_30
.LBB0_16:
	mv	a3, a0
	andi	a5, a0, 3
	bnez	a5, .LBB0_7
.LBB0_17:
	li	a4, 16
	bltu	a2, a4, .LBB0_20
# %bb.18:
	li	a4, 15
.LBB0_19:                               # =>This Inner Loop Header: Depth=1
	lw	a5, 0(a1)
	lw	t0, 4(a1)
	lw	t1, 8(a1)
	lw	t2, 12(a1)
	sw	a5, 0(a3)
	sw	t0, 4(a3)
	sw	t1, 8(a3)
	sw	t2, 12(a3)
	addi	a1, a1, 16
	addi	a2, a2, -16
	addi	a3, a3, 16
	bltu	a4, a2, .LBB0_19
.LBB0_20:
	li	a4, 8
	bltu	a2, a4, .LBB0_22
# %bb.21:
	lw	a4, 0(a1)
	lw	a5, 4(a1)
	sw	a4, 0(a3)
	sw	a5, 4(a3)
	addi	a3, a3, 8
	addi	a1, a1, 8
.LBB0_22:
	andi	a4, a2, 4
	beqz	a4, .LBB0_33
# %bb.23:
	lw	a4, 0(a1)
	sw	a4, 0(a3)
	addi	a3, a3, 4
	addi	a1, a1, 4
	j	.LBB0_33
.LBB0_24:
	sb	a4, 0(a3)
	addi	a3, a3, 1
	addi	a2, a2, -1
	addi	a1, a1, 16
	li	a5, 18
.LBB0_25:                               # =>This Inner Loop Header: Depth=1
	lw	t0, -12(a1)
	srli	a4, a4, 8
	slli	t1, t0, 24
	lw	t2, -8(a1)
	or	a4, t1, a4
	sw	a4, 0(a3)
	srli	a4, t0, 8
	slli	t0, t2, 24
	lw	t1, -4(a1)
	or	a4, t0, a4
	sw	a4, 4(a3)
	srli	t0, t2, 8
	slli	t2, t1, 24
	lw	a4, 0(a1)
	or	t0, t2, t0
	sw	t0, 8(a3)
	srli	t0, t1, 8
	slli	t1, a4, 24
	or	t0, t1, t0
	sw	t0, 12(a3)
	addi	a3, a3, 16
	addi	a2, a2, -16
	addi	a1, a1, 16
	bltu	a5, a2, .LBB0_25
# %bb.26:
	addi	a1, a1, -15
	li	a4, 16
	bltu	a2, a4, .LBB0_9
	j	.LBB0_30
.LBB0_27:
	sb	a4, 0(a3)
	srli	a5, a4, 8
	sb	a5, 1(a3)
	srli	a5, a4, 16
	sb	a5, 2(a3)
	addi	a3, a3, 3
	addi	a2, a2, -3
	addi	a1, a1, 16
	li	a5, 16
.LBB0_28:                               # =>This Inner Loop Header: Depth=1
	lw	t0, -12(a1)
	srli	a4, a4, 24
	slli	t1, t0, 8
	lw	t2, -8(a1)
	or	a4, t1, a4
	sw	a4, 0(a3)
	srli	a4, t0, 24
	slli	t0, t2, 8
	lw	t1, -4(a1)
	or	a4, t0, a4
	sw	a4, 4(a3)
	srli	t0, t2, 24
	slli	t2, t1, 8
	lw	a4, 0(a1)
	or	t0, t2, t0
	sw	t0, 8(a3)
	srli	t0, t1, 24
	slli	t1, a4, 8
	or	t0, t1, t0
	sw	t0, 12(a3)
	addi	a3, a3, 16
	addi	a2, a2, -16
	addi	a1, a1, 16
	bltu	a5, a2, .LBB0_28
# %bb.29:
	addi	a1, a1, -13
	li	a4, 16
	bltu	a2, a4, .LBB0_9
.LBB0_30:
	lbu	a4, 0(a1)
	lbu	a5, 1(a1)
	lbu	t0, 2(a1)
	sb	a4, 0(a3)
	sb	a5, 1(a3)
	lbu	a4, 3(a1)
	sb	t0, 2(a3)
	lbu	a5, 4(a1)
	lbu	t0, 5(a1)
	sb	a4, 3(a3)
	lbu	a4, 6(a1)
	sb	a5, 4(a3)
	sb	t0, 5(a3)
	lbu	a5, 7(a1)
	sb	a4, 6(a3)
	lbu	a4, 8(a1)
	lbu	t0, 9(a1)
	sb	a5, 7(a3)
	lbu	a5, 10(a1)
	sb	a4, 8(a3)
	sb	t0, 9(a3)
	lbu	a4, 11(a1)
	sb	a5, 10(a3)
	lbu	a5, 12(a1)
	lbu	t0, 13(a1)
	sb	a4, 11(a3)
	lbu	a4, 14(a1)
	sb	a5, 12(a3)
	sb	t0, 13(a3)
	lbu	a5, 15(a1)
	sb	a4, 14(a3)
	addi	a1, a1, 16
	addi	a4, a3, 16
	sb	a5, 15(a3)
	mv	a3, a4
	andi	a4, a2, 8
	beqz	a4, .LBB0_10
.LBB0_31:
	lbu	a4, 0(a1)
	lbu	a5, 1(a1)
	lbu	t0, 2(a1)
	sb	a4, 0(a3)
	sb	a5, 1(a3)
	lbu	a4, 3(a1)
	sb	t0, 2(a3)
	lbu	a5, 4(a1)
	lbu	t0, 5(a1)
	sb	a4, 3(a3)
	lbu	a4, 6(a1)
	sb	a5, 4(a3)
	sb	t0, 5(a3)
	lbu	a5, 7(a1)
	sb	a4, 6(a3)
	addi	a1, a1, 8
	addi	a4, a3, 8
	sb	a5, 7(a3)
	mv	a3, a4
	andi	a4, a2, 4
	beqz	a4, .LBB0_33
.LBB0_32:
	lbu	a4, 0(a1)
	lbu	a5, 1(a1)
	lbu	t0, 2(a1)
	sb	a4, 0(a3)
	sb	a5, 1(a3)
	lbu	a4, 3(a1)
	sb	t0, 2(a3)
	addi	a1, a1, 4
	addi	a5, a3, 4
	sb	a4, 3(a3)
	mv	a3, a5
.LBB0_33:
	andi	a4, a2, 2
	bnez	a4, .LBB0_36
# %bb.34:
	andi	a2, a2, 1
	bnez	a2, .LBB0_37
.LBB0_35:
	ret
.LBB0_36:
	lbu	a4, 0(a1)
	lbu	a5, 1(a1)
	sb	a4, 0(a3)
	addi	a1, a1, 2
	addi	a4, a3, 2
	sb	a5, 1(a3)
	mv	a3, a4
	andi	a2, a2, 1
	beqz	a2, .LBB0_35
.LBB0_37:
	lbu	a1, 0(a1)
	sb	a1, 0(a3)
	ret
.Lfunc_end0:
	.size	memcpy, .Lfunc_end0-memcpy
                                        # -- End function
	.ident	"clang version 18.1.8 (https://github.com/llvm/llvm-project.git 3b5b5c1ec4a3095ab096dd780e84d7ab81f3d7ff)"
	.section	".note.GNU-stack","",@progbits
	.addrsig
