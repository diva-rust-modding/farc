use anyhow::*;
use farc::*;
use std::path::*;
use structopt::*;

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

    use GenericArchive::*;
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
                Base(a) => extract(&root_dir, &a.entries),
                Compress(a) => extract(&root_dir, &a.entries),
                Extended(a) => {
                    use ExtendedArchives::*;
                    match a {
                        Base(a) => extract(&root_dir, &a.0.entries),
                        Compress(a) => extract(&root_dir, &a.0.entries),
                        Encrypt(a) => extract(&root_dir, &a.0.entries),
                        CompressEncrypt(a) => extract(&root_dir, &a.0.entries),
                    }
                }
                Future(a) => {
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

            let farc = GenericArchive::read(&input).unwrap().1;
            println!("{} archive with {} entries", farc.magic(), farc.len());
            match farc {
                Base(a) => view(&a.entries),
                Compress(a) => view(&a.entries),
                Extended(a) => match a {
                    ExtendedArchives::Base(a) => view(&a.0.entries),
                    ExtendedArchives::Compress(a) => view(&a.0.entries),
                    ExtendedArchives::Encrypt(a) => view(&a.0.entries),
                    ExtendedArchives::CompressEncrypt(a) => view(&a.0.entries),
                },
                Future(a) => match a {
                    FutureArchives::Base(a) => view(&a.0.entries),
                    FutureArchives::Compress(a) => view(&a.0.entries),
                    FutureArchives::Encrypt(a) => view(&a.0.entries),
                    FutureArchives::CompressEncrypt(a) => view(&a.0.entries),
                },
            }
        }
        _ => (),
    };
    Ok(())
}

fn extract<'a, E>(root_dir: &Path, entries: &'a [E]) -> Result<()>
where
    E: EntryExtract<'a>,
    E::Error: 'static + Send + Sync,
{
    for entry in entries {
        let mut file =
            File::create(root_dir.join(&entry.name())).context("failed to create file")?;
        let mut read = match entry.extractor()? {
            Some(a) => a,
            None => {
                println!("skip");
                continue;
            }
        };
        io::copy(&mut read, &mut file)?;
    }
    Ok(())
}
fn view<'a, E: Entry>(entries: &'a [E]) {
    for (i, entry) in entries.iter().enumerate() {
        println!("#{} {}", i + 1, entry.name());
    }
}
