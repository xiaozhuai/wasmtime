# `cranelift-assembler-meta`

This crate generates Cranelift-specific assembly code for x64 instructions. It
is designed to fit in with Cranelift-specific logic (e.g., register allocation)
and only needs to define the x64 instructions Cranelift emits. It is written in
the style of `cranelift-codegen-meta` and _could_ be migrated there (though not
necessarily).

### Structure

- [`dsl.rs`](src/dsl.rs): defines a domain-specific language (DSL) for
  describing x64 instructions; this language is intended to be compact--i.e.,
  define an x64 instruction on a single line--and a close-to-direct mapping of
  what we read in the x64 developer manual
- [`instructions.rs`](src/instructions.rs): defines x64 instructions using the
  DSL; add new instructions here
- [`generate.rs`](src/generate.rs): generates Rust code from the defined
  instructions to: assemble machine code, pretty-print, register-allocate.

### Use

This is primarily intended to be used for generating Rust code, i.e.,
`generate("some-file.rs")`. It also has the ability to print a list of the
defined instructions:

```console
$ cargo run
andb: I(al, imm8) => 0x24 ib
andw: I(ax, imm16) => 0x25 iw
andl: I(eax, imm32) => 0x25 id
...
```
