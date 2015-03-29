#!/usr/bin/env python

import os
import sys
import string
import math
import re
import random
from subprocess import call, check_output, STDOUT, CalledProcessError

"""
Use: ./test

This program generates a set of random files of increasing size.
Then, it compresses those files using the gzip format.
Finally, it runs the original Silver Searcher and our
Rust-ified Silver Searcher against each file with --stats
to determine how long it took to decompress and search it.

"""

SILVER_SEARCHER = "ag"
RUSTY_SEARCHER = "../the_silver_searcher/ag"
TEMP_NAME = "temp.txt"
COMP_NAME = TEMP_NAME + ".gz"
LINE_SIZE = 128

# 1 MB
MIN_FILE_SIZE = 1024
# 512 MB
MAX_FILE_SIZE = 512 * 1024 * 1024

def main():
    cleanup(TEMP_NAME)
    cleanup(COMP_NAME)
    results = []
    curr_file_size = MIN_FILE_SIZE
    while curr_file_size < MAX_FILE_SIZE:
        # Printable size (in MB)
        fsize = float(curr_file_size) / (1024 * 1024)
        if fsize >= 1:
            fsize = int(fsize)
        print "Creating", fsize, "mb file"

        f = open(TEMP_NAME, 'w')
        f.write(create_file(curr_file_size))
        f.close()
        print "Testing", fsize, "mb file"
        call(["gzip", TEMP_NAME])
        out1 = extract_seconds(run_search(SILVER_SEARCHER))
        out2 = extract_seconds(run_search(RUSTY_SEARCHER))
        results.append((curr_file_size, out1, out2))

        print "Original: " + str(out1) + ", Rusty: " + str(out2) + "\n"

        curr_file_size *= 2
        cleanup(COMP_NAME)
    print results

def extract_seconds(buf):
    """ Extract the number of seconds it took the silver searcher to run from the output """
    regex = re.compile(r"(?P<sec>\d+\.\d+) seconds")
    result = regex.search(buf)
    return float(result.group('sec'))

def run_search(searcher):
    """ Run the search and return the output as a string """
    try:
        out = check_output([searcher, "--stats", "-z", "abcd", COMP_NAME],
                stderr=STDOUT)
        return out
    except CalledProcessError as e:
        return e.output

def cleanup(filename):
    """ Remove the file at the filename if it exists """
    if os.path.exists(filename):
        os.remove(filename)

def create_file(num_bytes):
    """ Create a random file of size num_bytes """
    buf = ""
    for line in range(0, int(math.ceil(num_bytes / LINE_SIZE))):
        for i in range(1, min(LINE_SIZE, num_bytes)):
            buf += random.choice(string.printable)
        # insert a newline every LINE_SIZE characters to make it
        # printable and have characteristics a little more like
        # a normal source code file
        buf += "\n"
    return buf



if __name__ == '__main__':
    main()
