#!/bin/bash

# diesel-gaussdb 快速真实数据库测试
# 用于快速验证核心功能

set -e

echo "⚡ diesel-gaussdb 快速真实数据库测试"
echo "================================="

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# 检查Docker
if ! command -v docker &> /dev/null; then
    echo -e "${RED}❌ 需要安装 Docker${NC}"
    exit 1
fi

# 启动数据库（如果未运行）
echo -e "${BLUE}🚀 启动测试数据库...${NC}"
docker-compose -f docker-compose.test.yml up -d opengauss

# 等待数据库就绪
echo -e "${YELLOW}⏳ 等待数据库就绪...${NC}"
for i in {1..20}; do
    if docker-compose -f docker-compose.test.yml exec -T opengauss gsql -U gaussdb -d diesel_test -c "SELECT 1;" &>/dev/null; then
        echo -e "${GREEN}✅ 数据库已就绪${NC}"
        break
    fi
    echo -n "."
    sleep 1
done

# 设置环境变量
export GAUSSDB_TEST_URL="host=localhost port=5434 user=gaussdb password=Gaussdb@123 dbname=diesel_test"

# 运行核心测试
echo -e "${BLUE}🧪 运行核心测试...${NC}"

echo -e "${YELLOW}1. 测试数据库连接...${NC}"
cargo test --test diesel_integration test_basic_connection -- --nocapture

echo -e "${YELLOW}2. 测试CRUD操作...${NC}"
cargo test --test diesel_integration test_basic_crud_operations -- --nocapture

echo -e "${YELLOW}3. 测试事务...${NC}"
cargo test --test diesel_integration test_transaction_support -- --nocapture

echo -e "${GREEN}🎉 快速测试完成！${NC}"
echo -e "${GREEN}✅ diesel-gaussdb 真实数据库功能验证成功${NC}"
