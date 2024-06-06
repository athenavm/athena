// This is musl-libc memset commit 0784374d561435f7c787a555aeab8ede699ed298
//
// src/string/memset.c
//
// This was compiled into assembly with:
//
// riscv64-unknown-elf-gcc -march=rv32em -mabi=ilp32e -O3 -nostdlib -fno-builtin -funroll-loops -I../../obj/include -I../../include -S memset.c -o memset.s
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
	.file	"memset.c"
	.option nopic
	.attribute arch, "rv32em"
	.attribute unaligned_access, 0
	.attribute stack_align, 4
	.text
	.align	2
	.globl	memset
	.type	memset, @function
memset:
	beq	a2,zero,.LBBmemsetL2
	andi	a5,a1,0xff
	sb	a5,0(a0)
	add	a3,a0,a2
	sb	a5,-1(a3)
	li	a4,2
	bleu	a2,a4,.LBBmemsetL2
	sb	a5,1(a0)
	sb	a5,2(a0)
	sb	a5,-2(a3)
	sb	a5,-3(a3)
	li	t1,6
	bleu	a2,t1,.LBBmemsetL2
	sb	a5,3(a0)
	sb	a5,-4(a3)
	li	t0,8
	bleu	a2,t0,.LBBmemsetL2
	andi	a1,a1,255
	slli	a3,a1,8
	neg	t2,a0
	andi	a5,t2,3
	add	t1,a3,a1
	sub	a2,a2,a5
	slli	a1,t1,16
	add	t2,a0,a5
	add	a3,t1,a1
	andi	a5,a2,-4
	sw	a3,0(t2)
	add	a2,t2,a5
	sw	a3,-4(a2)
	bleu	a5,t0,.LBBmemsetL2
	sw	a3,4(t2)
	sw	a3,8(t2)
	sw	a3,-12(a2)
	sw	a3,-8(a2)
	li	t0,24
	bleu	a5,t0,.LBBmemsetL2
	andi	t1,t2,4
	sw	a3,12(t2)
	sw	a3,16(t2)
	sw	a3,20(t2)
	sw	a3,24(t2)
	addi	t0,t1,24
	sw	a3,-28(a2)
	sw	a3,-24(a2)
	sw	a3,-20(a2)
	sw	a3,-16(a2)
	li	a1,31
	sub	a2,a5,t0
	add	a5,t2,t0
	bleu	a2,a1,.LBBmemsetL2
	addi	t2,a2,-32
	andi	t1,t2,-32
	srli	a1,t1,5
	addi	t2,a1,1
	addi	a2,t1,32
	andi	t0,t2,7
	add	a2,a5,a2
	beq	t0,zero,.LBBmemsetL3
	li	t1,1
	beq	t0,t1,.LBBmemsetL28
	beq	t0,a4,.LBBmemsetL29
	li	a4,3
	beq	t0,a4,.LBBmemsetL30
	li	a1,4
	beq	t0,a1,.LBBmemsetL31
	li	t2,5
	beq	t0,t2,.LBBmemsetL32
	li	t1,6
	beq	t0,t1,.LBBmemsetL33
	sw	a3,0(a5)
	sw	a3,8(a5)
	sw	a3,16(a5)
	sw	a3,24(a5)
	addi	a5,a5,32
.LBBmemsetL33:
	sw	a3,0(a5)
	sw	a3,8(a5)
	sw	a3,16(a5)
	sw	a3,24(a5)
	addi	a5,a5,32
.LBBmemsetL32:
	sw	a3,0(a5)
	sw	a3,8(a5)
	sw	a3,16(a5)
	sw	a3,24(a5)
	addi	a5,a5,32
.LBBmemsetL31:
	sw	a3,0(a5)
	sw	a3,8(a5)
	sw	a3,16(a5)
	sw	a3,24(a5)
	addi	a5,a5,32
.LBBmemsetL30:
	sw	a3,0(a5)
	sw	a3,8(a5)
	sw	a3,16(a5)
	sw	a3,24(a5)
	addi	a5,a5,32
.LBBmemsetL29:
	sw	a3,0(a5)
	sw	a3,8(a5)
	sw	a3,16(a5)
	sw	a3,24(a5)
	addi	a5,a5,32
.LBBmemsetL28:
	sw	a3,0(a5)
	sw	a3,8(a5)
	sw	a3,16(a5)
	sw	a3,24(a5)
	addi	a5,a5,32
	beq	a5,a2,.LBBmemsetL38
.LBBmemsetL3:
	sw	a3,0(a5)
	sw	a3,8(a5)
	sw	a3,16(a5)
	sw	a3,24(a5)
	sw	a3,32(a5)
	sw	a3,40(a5)
	sw	a3,48(a5)
	sw	a3,56(a5)
	sw	a3,64(a5)
	sw	a3,72(a5)
	sw	a3,80(a5)
	sw	a3,88(a5)
	sw	a3,96(a5)
	sw	a3,104(a5)
	sw	a3,112(a5)
	sw	a3,120(a5)
	sw	a3,128(a5)
	sw	a3,136(a5)
	sw	a3,144(a5)
	sw	a3,152(a5)
	sw	a3,160(a5)
	sw	a3,168(a5)
	sw	a3,176(a5)
	sw	a3,184(a5)
	sw	a3,192(a5)
	sw	a3,200(a5)
	sw	a3,208(a5)
	sw	a3,216(a5)
	sw	a3,224(a5)
	sw	a3,232(a5)
	sw	a3,240(a5)
	sw	a3,248(a5)
	addi	a5,a5,256
	bne	a5,a2,.LBBmemsetL3
.LBBmemsetL2:
	ret
.LBBmemsetL38:
	ret
	.size	memset, .-memset
	.ident	"GCC: () 10.2.0"
