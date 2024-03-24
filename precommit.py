#! /usr/bin/env python3

import subprocess
import codecs

commands = (["cargo", "run", "--", "help"],
    ["cargo", "run", "--", "topic", "chunks"],
)
    
def update_readme(path):
    marker = b"## Help overview\n"
    file = open(path, mode="rb")
    data = file.read()
    file.close()
    data = data.split(marker)[0]
    #print(data)
    file = open(path, mode="wb")
    file.write(data)
    file.write(marker)
    for args in commands:
        output = subprocess.check_output(args)
        file.write(b"\n```\n$ %s \n%s```\n" % (bytes(" ".join(["wavrw"] + args[3:]), "ascii"), output))
    file.close()

if __name__ == "__main__":
    update_readme("README.md")
