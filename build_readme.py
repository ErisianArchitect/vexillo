import re, os, sys
from pathlib import Path
from hydra.toolbox import text

lib = Path('./src/lib.rs')
readme = Path('./README.md')
line_start = re.compile(r'^', re.MULTILINE)

def main(args: list[str]):
    lib_mod_time = lib.stat().st_mtime_ns
    readme_mod_time = readme.stat().st_mtime_ns
    
    # readme isn't newer.
    if readme_mod_time <= lib_mod_time:
        print("Not newer.")
        return 1
    
    source = lib.read_text()
    readme_text = readme.read_text()

    def after_prefixed_lines(src: str, prefix: str | re.Pattern) -> str:
        if isinstance(prefix, str):
            prefix = re.compile(re.escape(prefix))
        for m in line_start.finditer(src):
            if not prefix.match(src, m.start()):
                next_line = line_start.match(src, m.end())
                if not next_line:
                    return src[len(src):]
                return src[next_line.start():]
        return src[len(src):]

    cutoff = after_prefixed_lines(source, '//! ')

    if readme_text:
        lib.write_text('\n'.join((
            text.prepend_lines('//! ', readme_text),
            cutoff
        )))
    else:
        lib.write_text(cutoff)
    print("Readme written.")
    return 0

if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))