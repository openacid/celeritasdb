#!/usr/bin/env python2
# coding: utf-8

import redis
import random

r = redis.StrictRedis("127.0.0.1", 6379)
v = random.randint(1,10000)
print r.set("foo", str(v)), v
print r.get("foo")

