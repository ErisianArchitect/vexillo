"""
Builds the doc comment for `lib.rs` from `README.md`.
"""
from typing import (
    Pattern,
    Callable,
    Iterator,
)
import re, sys
from pathlib import Path

_newline_re = re.compile(r'\n')

def split(text: str, separator: str | Pattern = ',', keepends: bool | Pattern = False, keepemptyend: bool = False) -> Iterator[str]:
    """Split `text` by `separator`."""
    if isinstance(separator, str):
        separator = re.compile(re.escape(separator))
    index = 0
    while index <= len(text):
        match = separator.search(text, index)
        if not match:
            end = text[index:]
            if len(end) != 0 or keepemptyend:
                yield text[index:]
            break
        match keepends:
            case Pattern():
                endmat = keepends.match(text, match.start())
                if endmat:
                    next_index = endmat.end()
                else:
                    next_index = match.end()
            case True:
                next_index = match.end()
            case False | None:
                next_index = match.start()
            case _:
                raise ValueError(keepends)
        yield text[index:next_index]
        index = match.end()

def split_lines(text: str, keepends: bool = False, keepemptyend: bool = False) -> Iterator[str]:
    """Split lines iterator style."""
    yield from split(text, _newline_re, keepends, keepemptyend)

def prepend_lines(prefix: str | Callable[[int, str], str], text: str):
    """Prepend `prefix` to each line of `text`.
    `prefix` can be a callable that takes `(line_number: int, line_text: str)`.
    """
    if not callable(prefix):
        prefix = lambda *args, **kwargs: prefix
    return '\n'.join((''.join((prefix(i, line), line)) for i, line in enumerate(split_lines(text))))

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

    cutoff = after_prefixed_lines(source, '//! ')

    if readme_text:
        lib.write_text('\n'.join((
            prepend_lines('//! ', readme_text),
            cutoff
        )))
    else:
        lib.write_text(cutoff)
    print("Readme written.")
    return 0

if __name__ == '__main__':
    sys.exit(main(sys.argv[1:]))