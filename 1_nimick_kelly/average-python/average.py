#!/usr/bin/env python

from __future__ import print_function
import sys

"""
Use: ./average < [data file]

This program accepts a rainfall data file on stdin and provides output on
stdout.

In the rainfall data file, each input line contains one raw measurement.
This measurement is valid if it can be parsed as a 64-bit floating point
number. If the measurement is invalid or is less than 0, it is ignored.
If the measurement is the string '999', average stops consuming input
and prints the output immediately.
If there are no valid measurements, average exits without output.
Otherwise, it prints output on EOF.

The output consists of three lines:
- the average
- the number of measurements in the interval [average,average + 5]
- the number of measurements in the interval [average - 5,average]

TESTING
run 'nosetests' in the tests/ directory.
you must have the 'nose' python package installed.
"""

def main():
    values = []
    for line in sys.stdin:
        trimmed = line.strip()
        if trimmed == '999':
            break
        try:
            val = float(trimmed)
        except ValueError:
            continue
        if val >= 0:
            values.append(val)
    if not values:
        return
    average, above, below = calculate_average_and_interval(values)
    print(average)
    print(above)
    print(below)


def calculate_average_and_interval(values, bound=5):
    """
    values is a non-empty list of measurements, which are floating point numbers
    bound defines the interval above and below the average, inclusive.

    returns a tuple:
    (the average of the measurements,
     the number of measurements in the interval [average, average + bound],
     the number of measurements in the interval [average - bound, average])
    """
    average = sum(values) / len(values)
    above = 0
    below = 0
    lower_bound = average - bound
    upper_bound = average + bound
    for value in values:
        if lower_bound <= value <= average:
            below += 1
        if average <= value <= upper_bound:
            above += 1
    return (average, above, below)


if __name__ == '__main__':
    main()
