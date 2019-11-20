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
use std::io::{self, Read};

fn main() {
    let opts = Opt::from_args();

    match opts {
        Opt::Extract { path, root } => {
            let path = Path::new(&path);
            let mut file = File::open(&path).expect("Failed to open file");
            let mut input = vec![];
            file.read_to_end(&mut input).unwrap();

            let (_, farc) = GenericArchive::read(&input).expect("Failed to parse archive");
            let root_dir =  path.parent().unwrap();
            let root_dir = if root { root_dir.to_owned() } else { root_dir.join(path.file_stem().unwrap()) };
            std::fs::create_dir(&root_dir);
            match farc {
                GenericArchive::Base(a) => extract(&root_dir, &a.entries),
                GenericArchive::Compress(a) => extract(&root_dir, &a.entries),
                GenericArchive::Extended(a) => {
                    match a {
                        ExtendedArchives::Base(a) => extract(&root_dir, &a.0.entries),
                        ExtendedArchives::Compress(a) => extract(&root_dir, &a.0.entries),
                        _ => unimplemented!("Extracting encrypted archives is not yet supported")
                    }
                }
            };
        },
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

fn extract<'a, E: EntryExtract<'a>>(root_dir: &Path, entries: &'a [E]) -> Result<(), Box<dyn std::error::Error>> {
    for entry in entries {
        let mut file = File::create(root_dir.join(&entry.name()))?;
        io::copy(&mut entry.extractor(), &mut file)?;
    }
    Ok(())
}
