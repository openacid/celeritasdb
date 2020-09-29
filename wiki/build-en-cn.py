#!/usr/bin/env python
# coding: utf-8

"""
This script helps for writing doc in multiple language:
It convert a en/cn mixed doc to seperate en and cn version.

A template markdown marks en/cn lines with ``en:`` and ``cn:``.
A line without mark is always output to both en and cn versions.
E.g.:

    > cat x.tmpl
    ## en: Title
    ## cn: 标题

    - Frist:
      en: doit
      cn: 整它

    > build-en-cn.py x

    > cat x.md:
    ## Title

    - Frist:
      doit

    > cat x-cn.md:
    ## 标题

    - Frist:
      整它
"""
import re
import sys

def build(fn, cnfn, enfn):
    with open(fn, 'r') as f:
        lines = f.readlines()

    cnlines = []
    enlines = []
    for l in lines:
        cn = re.match(r'^([0-9-.# ∴∵>]*)cn: (.*)', l)
        en = re.match(r'^([0-9-.# ∴∵>]*)en: (.*)', l)
        if cn is None:
            if en is None:
                enlines.append(l)
                cnlines.append(l)
            else:
                enlines.append(en.groups()[0] + en.groups()[1] + '\n')
        else:
            cnlines.append(cn.groups()[0] + cn.groups()[1] + '\n')

    with open(cnfn, 'w') as f:
        f.write('<!-- built with "make i18n", do not edit-->\n')
        f.write(''.join(cnlines))

    with open(enfn, 'w') as f:
        f.write('<!-- built with "make i18n", do not edit-->\n')
        f.write(''.join(enlines))

if __name__ == "__main__":
    fn = sys.argv[1]
    cnfn = fn + '-cn.md'
    enfn = fn + '.md'
    tmpl_fn = fn + '.tmpl.md'
    build(tmpl_fn, cnfn, enfn)



