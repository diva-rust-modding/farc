# farchive [![Build Status](https://travis-ci.com/Waelwindows/farc.svg?branch=master)](https://travis-ci.com/Waelwindows/farc)
A command line tool and library used to create/manipulate SEGA's File Archive format.

## Getting Started

These instructions will get you a copy of the project up and running on your local machine for development and usage.

### Prerequisites

What things you need to install the software

* git
* [cargo](https://www.rust-lang.org/tools/install)

### Installing

* Clone this repository
```
$ git clone https://github.com/Waelwindows/farc.git
```

* Install the project using `cargo install`

```
$ cargo install
```

* (Only for developers) Build/test the project using `cargo`

```
$ cargo build
$ cargo test
```

And then you should be ready to run the program

```
$ farc
farc 0.1.0
Waelwindows <waelwindows9922@gmail.com>
manipulates SEGA File Archive file formats

USAGE:
    farc <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    create
    extract
    help       Prints this message or the help of the given subcommand(s)
    view
```

## Versioning

We use [SemVer](http://semver.org/) for versioning.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details

## Acknowledgments

* [DIVA_Tools](https://github.com/Waelwindows/DIVA_Tools)
* The amazing [MikuMikuLibrary](https://github.com/blueskythlikesclouds/MikuMikuLibrary) made by [blueskythlikesclouds](https://github.com/blueskythlikesclouds)
* [s117](https://github.com/s117) for making [DIVAFILE_Tool](https://github.com/s117/DIVAFILE_Tool)
