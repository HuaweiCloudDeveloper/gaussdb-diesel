#!/bin/bash

# GitHub Actions Workflow 验证脚本
# 验证所有 workflow 文件的语法和配置

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🔍 GitHub Actions Workflow 验证${NC}"
echo "=============================================="

# 检查是否安装了必要的工具
check_dependencies() {
    echo -e "${YELLOW}检查依赖工具...${NC}"
    
    # 检查 yq (YAML 处理器)
    if ! command -v yq &> /dev/null; then
        echo -e "${YELLOW}⚠️ yq 未安装，尝试安装...${NC}"
        if command -v brew &> /dev/null; then
            brew install yq
        elif command -v apt-get &> /dev/null; then
            sudo apt-get update && sudo apt-get install -y yq
        else
            echo -e "${RED}❌ 无法自动安装 yq，请手动安装${NC}"
            exit 1
        fi
    fi
    
    echo -e "${GREEN}✅ 依赖工具检查完成${NC}"
}

# 验证 YAML 语法
validate_yaml_syntax() {
    local file=$1
    echo -e "${YELLOW}验证 YAML 语法: $(basename $file)${NC}"
    
    if yq eval '.' "$file" > /dev/null 2>&1; then
        echo -e "${GREEN}✅ YAML 语法正确${NC}"
        return 0
    else
        echo -e "${RED}❌ YAML 语法错误${NC}"
        yq eval '.' "$file" 2>&1 | head -5
        return 1
    fi
}

# 验证 workflow 结构
validate_workflow_structure() {
    local file=$1
    local filename=$(basename "$file")
    echo -e "${YELLOW}验证 workflow 结构: $filename${NC}"
    
    local errors=0
    
    # 检查必需的顶级字段
    if ! yq eval '.name' "$file" > /dev/null 2>&1; then
        echo -e "${RED}❌ 缺少 'name' 字段${NC}"
        ((errors++))
    fi
    
    if ! yq eval '.on' "$file" > /dev/null 2>&1; then
        echo -e "${RED}❌ 缺少 'on' 字段${NC}"
        ((errors++))
    fi
    
    if ! yq eval '.jobs' "$file" > /dev/null 2>&1; then
        echo -e "${RED}❌ 缺少 'jobs' 字段${NC}"
        ((errors++))
    fi
    
    # 检查 jobs 结构
    local job_count=$(yq eval '.jobs | keys | length' "$file" 2>/dev/null || echo "0")
    if [ "$job_count" -eq 0 ]; then
        echo -e "${RED}❌ 没有定义任何 job${NC}"
        ((errors++))
    else
        echo -e "${GREEN}✅ 定义了 $job_count 个 job${NC}"
    fi
    
    # 检查每个 job 的基本结构
    yq eval '.jobs | keys | .[]' "$file" 2>/dev/null | while read job_name; do
        if ! yq eval ".jobs.\"$job_name\".runs-on" "$file" > /dev/null 2>&1; then
            echo -e "${RED}❌ Job '$job_name' 缺少 'runs-on' 字段${NC}"
            ((errors++))
        fi
        
        if ! yq eval ".jobs.\"$job_name\".steps" "$file" > /dev/null 2>&1; then
            echo -e "${RED}❌ Job '$job_name' 缺少 'steps' 字段${NC}"
            ((errors++))
        fi
    done
    
    if [ $errors -eq 0 ]; then
        echo -e "${GREEN}✅ Workflow 结构验证通过${NC}"
        return 0
    else
        echo -e "${RED}❌ 发现 $errors 个结构问题${NC}"
        return 1
    fi
}

# 验证 action 版本
validate_action_versions() {
    local file=$1
    echo -e "${YELLOW}验证 Action 版本: $(basename $file)${NC}"
    
    local warnings=0
    
    # 检查常用 action 的版本
    local actions_to_check=(
        "actions/checkout"
        "actions/cache"
        "actions/upload-artifact"
        "actions/download-artifact"
        "dtolnay/rust-toolchain"
    )
    
    for action in "${actions_to_check[@]}"; do
        local versions=$(grep -o "${action}@v[0-9]*" "$file" | sort | uniq || true)
        if [ -n "$versions" ]; then
            local version_count=$(echo "$versions" | wc -l)
            if [ $version_count -gt 1 ]; then
                echo -e "${YELLOW}⚠️ $action 使用了多个版本:${NC}"
                echo "$versions" | sed 's/^/  /'
                ((warnings++))
            else
                echo -e "${GREEN}✅ $action 版本一致: $versions${NC}"
            fi
        fi
    done
    
    if [ $warnings -eq 0 ]; then
        echo -e "${GREEN}✅ Action 版本检查通过${NC}"
        return 0
    else
        echo -e "${YELLOW}⚠️ 发现 $warnings 个版本警告${NC}"
        return 0  # 警告不算错误
    fi
}

# 验证环境变量和 secrets
validate_env_and_secrets() {
    local file=$1
    echo -e "${YELLOW}验证环境变量和 secrets: $(basename $file)${NC}"
    
    # 检查常见的环境变量
    local env_vars=$(grep -o '\${{ env\.[A-Z_]* }}' "$file" | sort | uniq || true)
    if [ -n "$env_vars" ]; then
        echo -e "${BLUE}📋 使用的环境变量:${NC}"
        echo "$env_vars" | sed 's/^/  /'
    fi
    
    # 检查 secrets
    local secrets=$(grep -o '\${{ secrets\.[A-Z_]* }}' "$file" | sort | uniq || true)
    if [ -n "$secrets" ]; then
        echo -e "${BLUE}🔐 使用的 secrets:${NC}"
        echo "$secrets" | sed 's/^/  /'
    fi
    
    echo -e "${GREEN}✅ 环境变量和 secrets 检查完成${NC}"
}

# 生成验证报告
generate_report() {
    local total_files=$1
    local passed_files=$2
    local failed_files=$3
    
    echo ""
    echo -e "${BLUE}📊 验证报告${NC}"
    echo "=============================================="
    echo "总文件数: $total_files"
    echo "通过验证: $passed_files"
    echo "验证失败: $failed_files"
    echo ""
    
    if [ $failed_files -eq 0 ]; then
        echo -e "${GREEN}🎉 所有 workflow 文件验证通过！${NC}"
        return 0
    else
        echo -e "${RED}❌ 有 $failed_files 个文件验证失败${NC}"
        return 1
    fi
}

# 主验证函数
main() {
    local workflow_dir=".github/workflows"
    
    if [ ! -d "$workflow_dir" ]; then
        echo -e "${RED}❌ 找不到 .github/workflows 目录${NC}"
        exit 1
    fi
    
    check_dependencies
    echo ""
    
    local total_files=0
    local passed_files=0
    local failed_files=0
    
    # 遍历所有 YAML 文件
    for file in "$workflow_dir"/*.yml "$workflow_dir"/*.yaml; do
        if [ -f "$file" ]; then
            ((total_files++))
            echo -e "${BLUE}🔍 验证文件: $(basename $file)${NC}"
            echo "----------------------------------------------"
            
            local file_passed=true
            
            # 验证 YAML 语法
            if ! validate_yaml_syntax "$file"; then
                file_passed=false
            fi
            
            # 验证 workflow 结构
            if ! validate_workflow_structure "$file"; then
                file_passed=false
            fi
            
            # 验证 action 版本
            validate_action_versions "$file"
            
            # 验证环境变量和 secrets
            validate_env_and_secrets "$file"
            
            if [ "$file_passed" = true ]; then
                echo -e "${GREEN}✅ $(basename $file) 验证通过${NC}"
                ((passed_files++))
            else
                echo -e "${RED}❌ $(basename $file) 验证失败${NC}"
                ((failed_files++))
            fi
            
            echo ""
        fi
    done
    
    if [ $total_files -eq 0 ]; then
        echo -e "${YELLOW}⚠️ 没有找到任何 workflow 文件${NC}"
        exit 1
    fi
    
    generate_report $total_files $passed_files $failed_files
}

# 运行主函数
main "$@"
