import os
import re
import subprocess

gate_block = False

def check_branch():
    res = subprocess.check_output(["git", "rev-parse", "--abbrev-ref", "HEAD"]).decode().strip()
    if res == "main":
        print("[阻断门禁] 禁止直接在main分支提交代码！")
        global gate_block
        gate_block = True

def check_sensitive_config():
    sensitive_words = ["password", "secret", "ak=", "sk=", "root@127.0.0.1"]
    for root, _, files in os.walk("."):
        for file in files:
            if file.endswith((".yml", ".json", ".ini")):
                path = os.path.join(root, file)
                with open(path, "r", encoding="utf-8", errors="ignore") as f:
                    content = f.read()
                    for word in sensitive_words:
                        if word in content:
                            print(f"[阻断门禁] 文件{path}包含敏感字段 {word}")
                            global gate_block
                            gate_block = True

def code_lint_check():
    try:
        subprocess.check_output(["python", "-m", "flake8", "."], stderr=subprocess.STDOUT)
    except subprocess.CalledProcessError as e:
        print("[告警门禁] 代码存在规范问题：", e.output.decode())

if __name__ == "__main__":
    print("===== 发布门禁自动化校验开始 =====")
    check_branch()
    check_sensitive_config()
    code_lint_check()
    print("===== 门禁校验结束 =====")
    if gate_block:
        exit(1)
    else:
        print("所有阻断门禁校验通过，可进入下一发布步骤")
        exit(0)