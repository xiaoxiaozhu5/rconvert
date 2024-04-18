# rconvert
convert visual studio solution to pnproj

# usage
```bash
rconvert 0.1.0

USAGE:
    rconvert.exe [FLAGS] --sln <sln>

FLAGS:
    -c, --clean      remove generated pnws/pnproj
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -s, --sln <sln>
```

# example
```bash
# generate pnws/pnproj
rconvert.exe -s c:\projects\test.sln
```
```bash
# clean generated pnws/pnproj
rconvert.exe -c -s c:\projects\test.sln
```
