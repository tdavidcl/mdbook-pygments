from pygments import highlight
from pygments.lexers import PythonLexer
from pygments.formatters import HtmlFormatter


def highlight_block( language_tag, code ):

    code = 'print "Hello World"'
    return (highlight(code, PythonLexer(), HtmlFormatter()))
