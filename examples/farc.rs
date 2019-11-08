use farc::*;
use structopt::*;
use std::path::*;

#[derive(StructOpt)]
#[structopt(name = "farc", about = "manipulates SEGA File Archive file formats")]
enum Opt {
    #[structopt(name = "create")]
    Create {
        #[structopt(parse(from_os_str))]
        create: PathBuf,
        #[structopt(short)]
        compress: bool,
        #[structopt(short)]
        encrypt: bool,
    },
    #[structopt(name = "extract")]
    Extract {
        path: PathBuf,
        #[structopt(long, short)]
        ///Extract to the root directory of the archive instead of a nested folder
        root: bool,
    },
    //Only view the archive without extracting
    #[structopt(name = "view")]
    View { path: PathBuf },
}

use std::fs::File;
use std::io::Read;
use farc::entry::*;

fn main() {
    let opts = Opt::from_args();

    match opts {
        Opt::View { path } => {
            let path = Path::new(&path);
            let mut file = File::open(&path).expect("Failed to open file");
            let mut input = vec![];
            file.read_to_end(&mut input).unwrap();

            let farc = BaseArchive::read(&input)
                .expect("Failed to parse archive")
                .1;
            println!("FArc archive with {} entries", farc.entries.len());
            for (i, entry) in farc.entries.iter().enumerate() {
                println!("#{} {}", i + 1, entry.name());
            }
        }
        _ => ()
    };
}
