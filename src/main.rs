use {
    art_of_vm::{
        assembler::Assembler, vm::VirtualMachine
    },
    bincode::{deserialize, serialize},
    std::{env::args, fs::{self, read_to_string, File}, io::Read, path::Path, time::{Duration, Instant}},
};

const DEFAULT_HEAP_SIZE: usize = 1024; // (bytes)
const BENCHMARK_ATTEMPTS: usize = 1000;

#[inline(always)]
#[cold]
fn usage(exe: String) -> ! {
    panic!("usage: {exe} (exe|assemble|benchmark|dbg) (file) (out_file [if using 'assemble'])");
}

fn main() {
    let mut args: Vec<String> = args().collect();
    let exe = args[0].clone();
    args.remove(0);

    if args.len() < 2 {
        usage(exe);
    }

    match args[0].as_str() {
        "exe" => {
            let mut file = File::open(args[1].as_str())
                .expect(format!("unable to read file {:?}", args[1]).as_str());
            let mut buf: Vec<u8> = vec![];
            file.read_to_end(&mut buf).unwrap();

            let code: Vec<u8> = deserialize(&buf)
                .expect("err deserializing given ArtOfVM machine code");

            let mut vm = VirtualMachine::new(code, DEFAULT_HEAP_SIZE);
            let run_t = Instant::now();
            vm.exec();
            let took = run_t.elapsed();

            println!("[exited successfully in {took:?}]");
        },
        "benchmark" => {
            let mut file = File::open(args[1].as_str())
                .expect(format!("unable to read file {:?}", args[1]).as_str());
            let mut buf: Vec<u8> = vec![];
            file.read_to_end(&mut buf).unwrap();

            let code: Vec<u8> = deserialize(&buf)
                .expect("err deserializing given ArtOfVM machine code");

            let mut durs: Vec<Duration> = vec![];

            for _ in 0..BENCHMARK_ATTEMPTS {
                let mut vm = VirtualMachine::new(code.clone(), DEFAULT_HEAP_SIZE);
                let run_t = Instant::now();
                vm.exec();
                let took = run_t.elapsed();

                durs.push(took);
            }

            let mut ms: Vec<u128> = durs.iter().map(|d| d.as_micros()).collect();
            ms.sort_unstable();

            let fast = ms.iter().min().unwrap().clone();
            let slow = ms.iter().max().unwrap().clone();

            let median_i = ms.len() / 2;
            let median = if ms.len() % 2 == 0 {
                (ms[median_i - 1] + ms[median_i]) / 2
            } else {
                ms[median_i]
            };

            let avg = ms.iter().sum::<u128>() as f64 / ms.len() as f64;

            println!(
                "\n\n\
                benchmark fastest (microseconds): {fast}\n\
                benchmark slowest (microseconds): {slow}\n\
                benchmark median (microseconds): {median}\n\
                benchmark average (microseconds): {avg}"
            );
        },
        "dbg" => {
            let mut file = File::open(args[1].as_str())
                .expect(format!("unable to read file {:?}", args[1]).as_str());
            let mut buf: Vec<u8> = vec![];
            file.read_to_end(&mut buf).unwrap();

            let code: Vec<u8> = deserialize(&buf)
                .expect("err deserializing given ArtOfVM machine code");

            println!("machine code:\n{code:?}");
        }
        "assemble" => {
            let mut file = read_to_string(args[1].clone())
                .expect(format!("unable to read file {:?}", args[1]).as_str());
            file.push('\0');

            if args.len() != 3 {
                usage(exe);
            }

            let out_file = args[2].clone();

            let mut assembler = Assembler::new(file);

            let assemble_t = Instant::now();
            let assembled = assembler.assemble();
            let took = assemble_t.elapsed();

            println!("took {took:?}");
            fs::write(Path::new(&out_file), serialize(&assembled).unwrap()).unwrap();
            println!("wrote to {out_file:?}");
        },
        _ => usage(exe),
    }
}
