from StringIO import StringIO
import sys, os
sys.path.append(os.path.abspath('../'))
from average import main, calculate_average_and_interval

real_stdin = sys.stdin

"""
Tests for average.
Must be run using 'nosetests' in the 'tests/' directory.
"""

class TestEverything(object):
    def test_empty_file(self):
        sys.stdin = StringIO()
        out = StringIO()
        sys.stdout = out
        main() # test with an empty file
        output = out.getvalue().strip()
        assert output == '' # no data, don't print anything

    def test_invalid_file(self):
        sys.stdin = open('invalid_file', 'r')
        out = StringIO()
        sys.stdout = out
        main() # test with a file with only invalid data
        output = out.getvalue().strip()
        assert output == '' # no data, don't print anything

    def test_valid_file(self):
        sys.stdin = open('valid_file', 'r')
        out = StringIO()
        sys.stdout = out
        main() # test with a file with valid data
        output = out.getvalue().strip()
        assert output == '6.0075694375\n2\n4' # no data, don't print anything

