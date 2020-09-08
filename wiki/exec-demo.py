#!/usr/bin/env python
# coding: utf-8

'''
A demo of local-min execution algo.
It create a random instance space and execute all instances in it.
Execution is repeated several times.
We can see different execution sequence always have the same result.

A report is generated in ``local-min-rst/``
You need graphviz installed: ``brew install graphviz``.

To test:

    python -m doctest -v exec-demo.py

To run it: see ``usage`` or

    python exec-demo.py -h
'''

usage = r'''
This is a demo of local-min execution algo:

Create a random instance space to run and dump result in `local-min-rst/`:

    python exec-demo.py

Load a dumpped instance space and re-run:

    # path/to/instance_space.yaml is generated with `python exec-demo.py`
    python exec-demo.py load path/to/instance_space.yaml

A prebuilt report is generated in local-min-rst-0/
'''

import os
import yaml
import sys
import copy
import subprocess
import random

alphabet = 'abcdefghijklmnopqrstuvwxyz'
nums = '12345'
operators = '+-*'

class Cmd(object):
    """
    Command

    >>> sorted(list(Cmd('x=y*2').vars))
    ['x', 'y']

    >>> Cmd('x=y*2').exec({'x':0, 'y':4})
    8

    >>> str(Cmd('x=y*2'))
    'x=y*2'

    >>> Cmd('x=y*2').statement
    'x=y*2'

    >>> Cmd('x=y*2').interfere_with(Cmd('y=x+1'))
    True
    >>> Cmd('x=y*2').interfere_with(Cmd('z=x+1'))
    True
    >>> Cmd('x=y*2').interfere_with(Cmd('z=a+1'))
    False

    """

    @classmethod
    def rand_cmd(clz, nvars, alphabet):
        """
        Create a random Cmd, with at most ``nvars`` variables from ``alphabet``.
        Such as ``x=y*2``
        """
        assert nvars >= 2

        nv = random.randint(2, nvars)

        expr = random.choice(alphabet) +'='
        for i in range(nv-1):
            expr += random.choice(alphabet+nums)
            expr += random.choice(operators)

        expr = expr[:-1]
        return Cmd(expr)

    def __init__(self, statement):

        self.statement = statement
        self.assignee, self.expr = statement.split('=', 1)

        self.lamb = None
        self.vars = set(self.assignee)

        self.parse_expr()

    def parse_expr(self):

        #  collect all variable this command used
        #  build the lambda for exec
        expr = ''
        for x in self.expr:
            if x in alphabet:
                self.vars.add(x)
                expr += 'sto["' + x + '"]'
            else:
                expr += ' ' + x + ' '

        lamb = 'lambda sto: {}'.format(expr)
        self.lamb = eval(lamb)

    def exec(self, storage):
        storage[self.assignee] = self.lamb(storage)
        return storage[self.assignee]

    def interfere_with(self, b):
        return len(self.vars & b.vars) > 0

    def __str__(self):
        return self.assignee +'=' + self.expr


class Inst(object):

    r"""
    Instance

    >>> str(Inst(0, [Cmd("x=y+2"),Cmd("y=4")], 5))
    '{0,5,x=y+2,y=4,[]}'

    >>> Inst(0, [Cmd("x=y+2"),Cmd("y=4")], 0).interfere_with(Inst(0, [Cmd("x=2"),Cmd("y=4")], 0))
    True

    >>> Inst(0, [Cmd("x=y+2"),Cmd("y=4")], 0).interfere_with(Inst(0, [Cmd("z=2"),Cmd("w=4")], 0))
    False

    >>> Inst(0, [Cmd("x=y+2"),Cmd("y=4")], 0).exec({'x': 1, 'y':3})
    [('x', 5), ('y', 4)]


    >>> Inst(1, [Cmd("x=y+2"),Cmd("y=4")], 2, deps=[3, 4]).dump()
    'cmds:\n- x=y+2\n- y=4\ndeps:\n- 3\n- 4\nid: 1\nseq: 2\n'

    >>> str(Inst.load('cmds:\n- x=y+2\n- y=4\ndeps:\n- 3\n- 4\nid: 1\nseq: 2\n'))
    '{1,2,x=y+2,y=4,[3, 4]}'

    """

    def __init__(self, id, cmds, seq, deps=None):
        self.id = id
        self.cmds = cmds
        self.seq = seq
        if deps is None:
            self.deps = []
        else:
            self.deps = deps

        self.ord = (seq, id)
        self.execed = False

    def to_dict(self):
        dic = dict(id=self.id, seq=self.seq, deps=self.deps,
                   cmds=[str(x) for x in self.cmds])
        return dic

    @classmethod
    def from_dict(self, dic):
        dic['cmds'] = [Cmd(x) for x in dic['cmds']]
        return Inst(**dic)

    def dump(self):
        """
        Dump an instance to yaml
        """
        dic = self.to_dict()
        return yaml.dump(dic, default_flow_style=False)

    @classmethod
    def load(clz, s):
        """
        Load an instance from yaml
        """
        dic = yaml.safe_load(s)
        return Inst.from_dict(dic)

    def vars(self):
        vs = set(self.cmds[0].vars)
        for c in self.cmds[1:]:
            vs = vs | c.vars
        return vs

    def interfere_with(self, b):
        for c in self.cmds:
            for c2 in b.cmds:
                if c.interfere_with(c2):
                    return True
        return False


    def exec(self, storage):
        rst = []
        for c in self.cmds:
            v = c.exec(storage)
            rst.append((c.assignee, v))
        return rst

    def __str__(self):
        return "{{{id},{seq},{cmds},{deps}}}".format(
            id=self.id,
            seq=self.seq,
            cmds=','.join([str(x) for x in self.cmds]),
            deps=str(self.deps)
        )


class Space(object):
    r"""
    Instance space

    >>> Space({0:Inst(0, [Cmd("x=y+2"),Cmd("y=4")], 0), 1:Inst(1, [Cmd("x=z+2")], 2)}, alphabet='abc').dump()
    'alphabet: abc\ninsts:\n  0:\n    cmds: [x=y+2, y=4]\n    deps: []\n    id: 0\n    seq: 0\n  1:\n    cmds: [x=z+2]\n    deps: []\n    id: 1\n    seq: 2\n'

    >>> Space.load('alphabet: abc\ninsts:\n  0:\n    cmds:\n    - x=y+2\n    - y=4\n    deps: []\n    id: 0\n    seq: 0\n  1:\n    cmds:\n    - x=z+2\n    deps: []\n    id: 1\n    seq: 2\n').dump()
    'alphabet: abc\ninsts:\n  0:\n    cmds: [x=y+2, y=4]\n    deps: []\n    id: 0\n    seq: 0\n  1:\n    cmds: [x=z+2]\n    deps: []\n    id: 1\n    seq: 2\n'
    """

    def __init__(self, insts, alphabet):
        self.insts = insts
        self.alphabet = alphabet

    @classmethod
    def rand_space(clz, ninsts, ncmds, nvars, alphabet):
        """
        Create a instance space with ``ninsts`` instances.

        Every instance has at most ``ncmds`` commands.
        Every command has at most ``nvars`` operand, including the assignee, e.g. x=y+2 has 3 operands.
        Choose operand naems from ``alphabet``, which is an iterable of single chars of single char.
        """

        insts = {}
        for i in range(ninsts):

            iid = i
            cmds = []
            seq = random.randint(0, ninsts)

            for j in range(random.randint(1, ncmds)):
                cmds.append(Cmd.rand_cmd(nvars, alphabet))

            inst = Inst(iid, cmds, seq)
            insts[iid] = inst

        for iid, inst in insts.items():
            for did, dinst in insts.items():
                if iid == did:
                    continue

                #  build random deps
                if not inst.interfere_with(dinst):
                    continue

                overlap = 10
                if random.randint(0, overlap) > did-iid + (overlap//2):
                    inst.deps.append(did)

        # ensure interferings always have at least one depends-on
        for iid, inst in insts.items():
            for did, dinst in insts.items():
                if iid == did:
                    continue

                #  build random deps
                if not inst.interfere_with(dinst):
                    continue

                if iid not in dinst.deps:
                    inst.deps.append(did)

        return Space(insts, alphabet)

    def to_dict(self):
        insts = {k: v.to_dict() for k, v in self.insts.items()}

        dic = dict(
            insts=insts,
            alphabet=self.alphabet
        )
        return dic

    def dump(self):
        dic = self.to_dict()
        return yaml.dump(dic, default_flow_style=None)

    @classmethod
    def load(clz, s):
        dic = yaml.safe_load(s)
        al = dic['alphabet']
        insts = {
                k: Inst.from_dict(v) for k, v in dic['insts'].items()
        }

        return Space(insts, al)

    def output_dot(self):

        """
        Output graphviz(dot) source file for generating picture of a dependency graph.
        """

        insts = self.insts
        tmpl ="""
strict digraph "inst-graph" {
    graph [
        rankdir = "TB",
    ]
    rank = same
    node [
          fontsize=24
          shape=none
    ];
"""
        for iid, inst in insts.items():
            nid = 'n' + str(iid)

            cmds = '\\n'.join([str(x) for x in inst.cmds])
            line = '{} [label="id:{} seq:{}\\n{}"]'.format(nid, iid, inst.seq, cmds)
            tmpl += line + '\n'

            for d in inst.deps:
                tmpl += '{} -> {} [label="{}"]\n'.format(
                        nid, 'n' + str(d),
                        ''.join(sorted(inst.vars() & insts[d].vars()))
                )

        tmpl += '}\n'
        return tmpl

class Exec(object):
    """
    Execute all instances with a given storage.
    This impl is based on the local-min algo.

    Instances are executed in an topology order.
    """

    def __init__(self, sp, storage):
        self.sp = sp
        self.storage = storage

        self.insts = copy.deepcopy(sp.insts)
        self.exec_seq = []

    def min_ord(self, path):
        """
        Find the instance with minimal ``ord`` along a walking path.

        Returns:
            the ``ord``
        """
        minord = None
        for instance_id in path:
            inst = self.insts[instance_id]
            if minord is None or minord > inst.ord:
                minord = inst.ord

        return minord

    def get_min_dep(self, instance_id):
        """
        Find the unexecuted dependent instance with the minimal ``ord``.
        """
        inst = self.insts[instance_id]
        depids = [x for x in inst.deps if x in self.insts]
        depids = sorted(depids, key=lambda x: self.insts[x].ord)
        if len(depids) > 0:
            return depids[0]
        return None

    def walk(self, iid):

        """
        Walk along the min-edge and remove the minimal ord instance in a cycle.
        If an instance without outgoing edge is found, executed it.
        """

        path = [iid]

        while len(path) > 0:

            iid = path[-1]

            did = self.get_min_dep(iid)
            #  print("walk path:", path, 'min-dep:', did)

            if did is None:
                self.do_exec(iid)
                path.pop(-1)
                continue

            if did not in path:
                path.append(did)
                continue

            #  cycle found, find the min ord instance in this cycle
            dindex = path.index(did)
            minord = self.min_ord(path[dindex:])

            minid = minord[1]
            minindex = path.index(minid)

            #  remove min edge in this cycle
            nxt = self.get_min_dep(minid)
            self.insts[minid].deps.remove(nxt)

            #  continue from the min instance
            path = path[:minindex+1]


    def exec(self):
        """
        Execute all instances, it starts from a random unexecuted instance
        everytime until no instance left.
        """

        self.insts = copy.deepcopy(sp.insts)

        while len(self.insts) > 0:
            #  pick a random instance to start
            iid = random.choice(list(self.insts))
            self.walk(iid)

    def do_exec(self, iid):

        inst = self.insts[iid]

        rst = inst.exec(self.storage)
        del self.insts[iid]

        self.exec_seq.append(iid)


def popen(cmds, input=None):
    """
    Open a sub process.
    """

    defenc = None

    if hasattr(sys, 'getfilesystemencoding'):
        defenc = sys.getfilesystemencoding()

    if defenc is None:
        defenc = sys.getdefaultencoding()

    subproc = subprocess.Popen(cmds,
                               stdin=subprocess.PIPE,
                               stdout=subprocess.PIPE,
                               stderr=subprocess.PIPE,
                               encoding=defenc
                               )

    out, err = subproc.communicate(input=input, timeout=1)
    subproc.wait()
    if subproc.returncode != 0:
        print("failed to run:", cmds)
        print(out)
        print(err)

    return subproc.returncode, out, err


def output(instance_space, init_storage, runs):
    """
    Output execution results and execution sequence of instances to stdout,
    and to a markdown file with a dependency graph image.
    """

    os.makedirs('local-min-rst', exist_ok=True)

    print('A detailed report is generated in local-min-rst/report.md')
    for r in runs:
        print("result:", r[0], "exec sequence:", r[1])

    with open('local-min-rst/instance_space.yaml', 'w') as f:
        f.write(instance_space.dump())

    dot = instance_space.output_dot()
    with open('local-min-rst/dep_graph.dot', 'w') as f:
        f.write(dot)

    with open('local-min-rst/report.md', 'w') as f:
        f.write('# Local-min exec algo demo report\n')
        f.write('To rerun: `python exec-demo.py load local-min-rst/instance_space.yaml`\n')
        f.write('\n')
        f.write('To run with a random instance space: `python exec-demo.py`\n')

        f.write("## Instance space\n")
        f.write("```yaml\n")
        f.write(instance_space.dump() + '\n')
        f.write("```\n")

        f.write('![](dep_graph.jpg)\n')

        f.write('## Executions\n')
        f.write('Initial storage `{}`\n'.format(init_storage))
        f.write('\n')

        r0 = runs[0][0]
        f.write('Execution Result `{}`\n'.format(r0))
        f.write('\n')
        f.write('Execution sequences:\n')

        f.write('```\n')

        for (i, r) in enumerate(runs):
            f.write("{0:>02}-th: {1}\n".format(i, str(r[1])))
            if r[0] != r0:
                f.write("inconsistent result: " + str(r[0]) + '\n')

        f.write('```\n')


    popen(['dot', '-Tjpg', '-o', 'local-min-rst/dep_graph.jpg'], input=dot)


if __name__ == "__main__":

    sp = Space.rand_space(ninsts=10, ncmds=1, nvars=3, alphabet="xyzwab")
    #  sp = Space.rand_space(ninsts=20, ncmds=1, nvars=3, alphabet="xyzwab")

    if len(sys.argv) > 1:
        if sys.argv[1] in ('-h', '--help', 'help'):
            print(usage)
            sys.exit(0)

        if sys.argv[1] == 'load':
            fn = sys.argv[2]
            with open(fn, 'r') as f:
                yml = f.read()
            sp = Space.load(yml)

    runs = []
    init_storage = {}
    for i, k in enumerate(sp.alphabet):
        init_storage[k] = i+1
    for ii in range(10):

        storage = copy.deepcopy(init_storage)

        ee = Exec(sp, storage)
        ee.exec()
        rst = storage
        seq = ee.exec_seq

        runs.append((rst, seq))
        if runs[0][0] != rst:
            break

    output(sp, init_storage, runs)
    if runs[0][0] != runs[-1][0]:
        print("Inconsistent result found! see local-min-rst/rst.md")
