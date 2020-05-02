use rcut;

fn main() {
    let input_args: Vec<_> = std::env::args().collect();
    let input_args = input_args.iter().map(|s| s.as_str()).collect();

    rcut::do_rcut(&input_args)
}
