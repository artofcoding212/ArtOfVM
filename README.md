<p align="center">
    <img height="256" src="logos/ArtOfVMLogo.png">
    <img height="256" src="logos/ArtOfASMLogo.png">
</p>

<div align="center">
    <strong>ArtOfVM</strong>
    <p>A virtual machine written in Rust that hosts the best Assembly ever, ArtOfASM.</p>
    <p>
    <img src="https://img.shields.io/badge/version-0.0.2-blue?style=for-the-badge" alt="version">
    <img src="https://img.shields.io/badge/platforms-Linux %7C Windows-blue?style=for-the-badge" alt="platforms">
    <img src="https://tokei.rs/b1/github/artofcoding212/ArtOfVM?category=code&style=for-the-badge" alt="lines">
    </p>
</div>


# ArtOfVM
A virtual machine written in Rust with the goal of hosting my own Assembly architecture: ArtOfAssembly, or ArtOfASM in short.

### How do I use it?
Download the executable targeted towards your platform (Windows x86_64-msvc and Linux x86_64-gnu are the only supported platforms) under [the latest release](https://github.com/artofcoding212/ArtOfVM/releases). You can use it with the following arguments:
* assemble [asm_file] [out_file]
    * Assembles the `[asm_file]` and writes the output to the `[out_file]`.
* dbg [out_file]
    * Prints out the machine code that the `[out_file]` represents.
* benchmark [out_file]
    * Executes the `[out_file]` 1,000 times and prints out the fastest & slowest times recorded as well as the median and average time in microseconds.
* exe [out_file]
    * Executes the `[out_file]`.

You can also download the repository and use the library in your Rust projects.

### How can I learn ArtOfASM?
As of now, ArtOfASM doesn't have any documentation. The best I have to offer are some examples I made under the [`tests`](https://github.com/artofcoding212/ArtOfVM/tree/master/tests) directory. If you want to get a more extensive education upon the language,
go check out the assembler in [`src/assembler.rs`](https://github.com/artofcoding212/ArtOfVM/blob/master/src/assembler.rs).