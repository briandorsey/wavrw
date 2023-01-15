#! /usr/bin/env python3

import subprocess

def update_readme(path):
    marker = b"## Help overview\n"
    file = open(path, mode="rb")
    data = file.read()
    file.close()
    data = data.split(marker)[0]
    #print(data)
    output = subprocess.check_output(["cargo", "run", "--", "help"])
    #print(output)
    file = open(path, mode="wb")
    file.write(data)
    file.write(marker)
    file.write(b"\n```\n" + output + b"\n```\n")
    file.close()

if __name__ == "__main__":
    update_readme("README.md")
