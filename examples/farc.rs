use farc::*;
use std::path::*;
use structopt::*;
use anyhow::*;

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

fn main() -> Result<()> {
    let opts = Opt::from_args();

    match opts {
        Opt::Extract { path, root } => {
            let path = Path::new(&path);
            let mut file = File::open(&path).context("Failed to open file")?;
            let mut input = vec![];
            file.read_to_end(&mut input)?;

            let (_, farc) = GenericArchive::read(&input).expect("Failed to parse archive");
            let root_dir = path.parent().context("Path has no parent")?;
            let root_dir = if root {
                root_dir.to_owned()
            } else {
                root_dir.join(path.file_stem().context("No valid file stem found")?)
            };
            std::fs::create_dir(&root_dir);
            match farc {
                GenericArchive::Base(a) => extract(&root_dir, &a.entries),
                GenericArchive::Compress(a) => extract(&root_dir, &a.entries),
                GenericArchive::Extended(a) => {
                    use ExtendedArchives::*;
                    match a {
                        Base(a) => extract(&root_dir, &a.0.entries),
                        Compress(a) => extract(&root_dir, &a.0.entries),
                        _ => unimplemented!("Extracting encrypted archives is not yet supported"),
                    }
                }
                GenericArchive::Future(a) => {
                    use FutureArchives::*;
                    match a {
                        Base(a) => extract(&root_dir, &a.0.entries),
                        Compress(a) => extract(&root_dir, &a.0.entries),
                        _ => unimplemented!("Extracting encrypted archives is not yet supported"),
                    }
                }
            }?;
        }
        Opt::View { path } => {
            let path = Path::new(&path);
            let mut file = File::open(&path).context("Failed to open file")?;
            let mut input = vec![];
            file.read_to_end(&mut input)?;

            let farc = BaseArchive::read(&input)
                .expect("Failed to parse archive")
                .1;
            println!("FArc archive with {} entries", farc.entries.len());
            for (i, entry) in farc.entries.iter().enumerate() {
                println!("#{} {}", i + 1, entry.name());
            }
        }
        _ => (),
    };
    Ok(())
}

fn extract<'a, E>(
    root_dir: &Path,
    entries: &'a [E],
) -> Result<()>
where
    E: EntryExtract<'a>,
    E::Error: 'static + Send + Sync
{
    for entry in entries {
        let mut file = File::create(root_dir.join(&entry.name())).context("failed to create file")?;
        let mut read = match entry.extractor()? {
            Some(a) => a,
            None => continue
        };
        io::copy(&mut read, &mut file)?;
    }
    Ok(())
}
