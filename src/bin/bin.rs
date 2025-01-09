use std::fs::File;
use std::io::{self, BufRead, BufReader, stdin};
use std::env;
use std::process;
use lyagushka::Lyagushka;

/// The entry point for the command-line tool that reads a dataset of integers from either a file or stdin,
/// performs cluster and gap analysis using specified parameters, and prints the results as a JSON string.
///
/// This tool expects either a filename as an argument or a list of integers piped into stdin. It also requires
/// two additional command-line arguments: a factor for adjusting clustering and gap detection thresholds,
/// and a minimum cluster size. The tool reads the dataset, performs the analysis by identifying clusters
/// and significant gaps, calculates z-scores for each, and prints the JSON-serialized results to stdout.
///
/// # Usage
/// To read from a file:
/// ```
/// cargo run -- filename.txt 0.5 2
/// ```
///
/// To read from stdin:
/// ```
/// echo "1\n2\n10\n20" | cargo run -- 0.5 2
/// ```
///
/// # Arguments
/// - A filename (if not receiving piped input) to read the dataset from.
/// - `factor`: A floating-point value used to adjust the sensitivity of cluster and gap detection.
/// - `min_cluster_size`: The minimum number of contiguous points required to be considered a cluster.
///
/// # Exit Codes
/// - `0`: Success.
/// - `1`: Incorrect usage or failure to parse the input data.
///
/// # Errors
/// This tool will exit with an error if the required arguments are not provided, if the specified file cannot be opened,
/// or if the input data cannot be parsed into integers.
///
/// # Note
/// This function does not return a value but directly exits the process in case of failure.
///
fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    // Input handling
    let dataset: Vec<i32> = if atty::is(atty::Stream::Stdin) {
        if args.len() != 4 {
            eprintln!("Usage: {} <filename> <factor> <min_cluster_size>", args[0]);
            process::exit(1);
        }
        let filename = &args[1];
        let file = File::open(filename)?;
        BufReader::new(file).lines().filter_map(Result::ok)
            .filter_map(|line| line.trim().parse::<i32>().ok()) // Directly parse to i32
            .collect()
    } else {
        stdin().lock().lines().filter_map(Result::ok)
            .filter_map(|line| line.trim().parse::<i32>().ok()) // Directly parse to i32
            .collect()
    };

    let factor: f32 = args[args.len() - 2].parse().expect("Factor must be a float");
    let min_cluster_size: usize = args[args.len() - 1].parse().expect("Min cluster size must be an integer");

    // Analysis and output
    let mut zhaba = Lyagushka::new(dataset);
    println!("{}", zhaba.search(factor, min_cluster_size));

    Ok(())
}