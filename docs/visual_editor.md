# RuleGo 可视化编辑器设计

## 1. 技术栈选型

- 前端框架: Vue 3 + TypeScript 
- 图形引擎: LogicFlow
- UI 组件: Element Plus
- 状态管理: Pinia
- API 客户端: Axios

## 2. 核心功能模块

### 2.1 规则链列表页
```typescript
interface RuleChain {
    id: string;
    name: string;
    root: boolean;
    created_at: number;
    updated_at: number;
    version: number;
}

// 列表页功能
- 规则链 CRUD
- 版本管理
- 导入/导出
- 复制/克隆
```

### 2.2 规则链编辑器
```typescript
interface Node {
    id: string;
    type: string;
    position: { x: number, y: number };
    data: {
        type_name: string;
        chain_id: string;
        config: Record<string, any>;
    }
}

// 编辑器功能
- 拖拽添加节点
- 连线管理
- 节点配置
- 画布操作
```

### 2.3 组件面板
```typescript
interface ComponentMeta {
    type_name: string;
    name: string;
    description: string;
    config_schema: JSONSchema;
    icon: string;
}

// 组件面板功能
- 组件分类
- 组件搜索
- 拖拽支持
- 配置说明
```

## 3. 页面布局

```
+------------------------+
|        Header         |
+------------------------+
|      |                |
| 组件  |     画布        |
| 面板  |                |
|      |                |
+------+----------------+
|      |    属性面板     |
+------------------------+
```

## 4. 数据流转

```mermaid
graph LR
    A[编辑器操作] --> B[状态更新]
    B --> C[JSON生成]
    C --> D[后端API]
    D --> E[规则引擎]
```

## 5. API 接口

### 5.1 规则链管理
```typescript
// 规则链列表
GET /api/chains

// 加载规则链
GET /api/chains/:id

// 保存规则链
POST /api/chains

// 删除规则链
DELETE /api/chains/:id
```

### 5.2 组件管理
```typescript
// 获取组件列表
GET /api/components

// 获取组件配置模板
GET /api/components/:type/schema
```

## 6. 节点配置表单

为每种节点类型定义配置表单:

```typescript
const NODE_FORMS = {
    // 日志节点
    log: {
        template: {
            type: 'textarea',
            label: '日志模板',
            required: true
        }
    },
    
    // 脚本节点
    script: {
        script: {
            type: 'monaco-editor',
            label: 'JavaScript代码',
            language: 'javascript'
        },
        output_type: {
            type: 'input',
            label: '输出类型'
        }
    },
    
    // 转换节点
    transform: {
        template: {
            type: 'json-editor',
            label: '转换模板'
        }
    },
    
    // REST客户端节点
    rest_client: {
        url: {
            type: 'input',
            label: 'URL',
            required: true
        },
        method: {
            type: 'select',
            label: '请求方法',
            options: ['GET', 'POST', 'PUT', 'DELETE']
        },
        headers: {
            type: 'key-value',
            label: '请求头'
        }
    }
};
```

## 7. 开发计划

### 第一阶段
- [x] 基础框架搭建
- [x] 规则链列表
- [x] 简单画布操作

### 第二阶段
- [ ] 组件面板
- [ ] 节点配置
- [ ] 连线管理

### 第三阶段
- [ ] 版本管理
- [ ] 导入导出
- [ ] 调试功能

### 第四阶段
- [ ] 在线测试
- [ ] 监控集成
- [ ] 文档完善

## 8. 部署架构

```mermaid
graph TD
    A[Nginx] --> B[前端应用]
    A --> C[后端API]
    C --> D[规则引擎]
    D --> E[Redis]
    D --> F[PostgreSQL]
```

## 9. 安全考虑

1. API 认证
2. CSRF 防护
3. 节点配置验证
4. 用户权限控制
5. 操作审计日志

## 10. 本地开发

```bash
# 安装依赖
pnpm install

# 启动开发服务器
pnpm dev

# 构建
pnpm build

# 测试
pnpm test
``` 