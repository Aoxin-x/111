#!/bin/bash
# SQLRustGo 发布门禁脚本
# 使用方法: ./gatekeeper.sh

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${YELLOW}============================================"
echo -e "  SQLRustGo 发布门禁检查"
echo -e "============================================${NC}"

total_checks=6
passed_checks=0
failed_checks=0

run_check() {
    echo -e "\n${YELLOW}[$passed_checks/$total_checks] $1${NC}"
    echo -e "${YELLOW}--------------------------------------------${NC}"
    if $2; then
        echo -e "${GREEN}✓ $1 通过${NC}"
        passed_checks=$((passed_checks + 1))
        return 0
    else
        echo -e "${RED}✗ $1 失败${NC}"
        failed_checks=$((failed_checks + 1))
        return 1
    fi
}

run_check "1. 编译检查" "cargo build --all-features"

run_check "2. 测试检查" "cargo test --all-features"

run_check "3. Clippy 代码质量检查" "cargo clippy --all-features -- -D warnings"

run_check "4. rustfmt 代码格式检查" "cargo fmt --check --all"

run_check "5. cargo-audit 安全审计" "cargo audit"

run_check "6. cargo-outdated 依赖更新检查" "cargo outdated --exit-code 1"

echo -e "\n${YELLOW}============================================"
echo -e "  门禁检查结果"
echo -e "============================================${NC}"

echo -e "\n通过: ${GREEN}$passed_checks/$total_checks${NC}"
echo -e "失败: ${RED}$failed_checks/$total_checks${NC}"

if [ $failed_checks -eq 0 ]; then
    echo -e "\n${GREEN}🎉 所有门禁检查通过！可以发布。${NC}"
    exit 0
else
    echo -e "\n${RED}❌ 门禁检查未通过，修复后重试。${NC}"
    exit 1
fi