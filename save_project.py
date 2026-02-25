#!/usr/bin/env python3
"""
项目快照生成器 - 支持多级拆分
用法:
    python save_project.py --summary              # 只生成摘要 summary.md
    python save_project.py --split-depth 1        # 按顶层目录拆分（默认）
    python save_project.py --split-depth 2        # 按二级目录拆分（将 src 拆成 core, app, ui 等）
    python save_project.py --modules src/core     # 只生成 core 子模块
"""

import os
import sys
import argparse
from pathlib import Path

# ===== 默认配置 =====
INCLUDE_DIRS = {'src', 'tests', 'benches', 'locales', 'examples'}
INCLUDE_EXTS = {'.rs', '.toml', '.yml', '.yaml', '.md'}
ROOT_CONFIGS = {'Cargo.toml', 'README.md', 'LICENSE'}
IGNORE_DIRS = {'target', '.git', '.idea', '.vscode', 'node_modules', '__pycache__'}
IGNORE_EXTS = {'.lock', '.log', '.bak', '.swp', '.tmp'}
KEEP_LANGS = {'zh-CN.yml', 'en.yml'}

# 模块中文名映射（支持多级目录，用 '/' 分隔）
MODULE_NAME_MAP = {
    # 顶层目录
    'src': '源代码',
    'tests': '测试',
    'benches': '基准测试',
    'locales': '本地化',
    'examples': '示例',
    'root': '根目录文件',
    
    # src 下的二级目录（按需添加）
    'src/core': '核心模块',
    'src/app': '应用模块',
    'src/ui': '界面模块',
    'src/tools': '工具模块',
    'src/render': '渲染模块',
    'src/format': '格式模块',
    'src/history': '历史记录',
    'src/animation': '动画模块',
    # 你可以在这里继续添加更多映射
}

# ===== 参数解析 =====
parser = argparse.ArgumentParser(description='生成项目代码快照')
parser.add_argument('--summary', action='store_true', help='只生成项目摘要 (summary.md)')
parser.add_argument('--split-depth', type=int, default=1, help='拆分层级，1=按顶层目录，2=按二级目录，...')
parser.add_argument('--modules', type=str, help='要生成快照的模块路径，逗号分隔，例如 "src/core,src/app"')
parser.add_argument('--lang', type=str, help='保留的语言文件，逗号分隔，例如 "zh-CN,en"')
parser.add_argument('--strip-comments', action='store_true', help='去除代码中的单行注释和空行')
parser.add_argument('--output-dir', type=str, default='.', help='输出目录')
args = parser.parse_args()

# 处理语言参数
if args.lang:
    keep_langs = {f"{lang}.yml" for lang in args.lang.split(',')}
else:
    keep_langs = KEEP_LANGS

output_dir = Path(args.output_dir)
output_dir.mkdir(parents=True, exist_ok=True)
project_root = Path.cwd()

# ===== 核心函数 =====
def should_include_file(rel_path: Path) -> bool:
    """判断是否应该包含该文件"""
    if rel_path.parent == Path('.'):
        return rel_path.name in ROOT_CONFIGS
    for part in rel_path.parts:
        if part in IGNORE_DIRS:
            return False
    if rel_path.suffix in IGNORE_EXTS:
        return False
    if rel_path.parent.name == 'locales':
        if rel_path.name not in keep_langs:
            return False
    return rel_path.suffix in INCLUDE_EXTS

def collect_files():
    """收集所有符合条件的文件路径"""
    files = []
    for root, dirs, _ in os.walk(project_root):
        root_path = Path(root)
        dirs[:] = [d for d in dirs if d not in IGNORE_DIRS]
        for file in os.listdir(root):
            file_path = root_path / file
            rel_path = file_path.relative_to(project_root)
            if should_include_file(rel_path):
                files.append(rel_path)
    return sorted(files)

def get_module_key(rel_path: Path, depth: int) -> str:
    """
    根据深度获取模块键名
    depth=1: 返回顶层目录名（如 'src'）或 'root'
    depth=2: 返回 'src/core' 这样的路径
    """
    parts = rel_path.parts
    if len(parts) == 1:  # 根目录文件
        return 'root'
    if depth == 1:
        return parts[0]
    else:
        # 取前 depth 个部分，用 '/' 连接
        key_parts = parts[:min(depth, len(parts)-1)]  # 至少保留一个目录
        return '/'.join(key_parts)

def group_by_depth(files, depth):
    """按指定深度分组文件"""
    modules = {}
    for rel_path in files:
        key = get_module_key(rel_path, depth)
        modules.setdefault(key, []).append(rel_path)
    return modules

def write_file_tree(f, files, title):
    """写入文件树"""
    f.write(f"## {title}\n\n")
    f.write("```text\n")
    tree = {}
    for rel_path in files:
        parts = list(rel_path.parts)
        current = tree
        for part in parts[:-1]:
            current = current.setdefault(part, {})
        current[parts[-1]] = None
    def write_tree(d, indent=''):
        for name, sub in sorted(d.items()):
            if sub is None:
                f.write(f"{indent}{name}\n")
            else:
                f.write(f"{indent}{name}/\n")
                write_tree(sub, indent + '    ')
    write_tree(tree)
    f.write("```\n\n")

def write_file_contents(f, rel_path):
    """写入单个文件内容"""
    full_path = project_root / rel_path
    lang = rel_path.suffix.lstrip('.')
    if lang == 'yml':
        lang = 'yaml'
    elif lang == 'rs':
        lang = 'rust'
    f.write(f"### 文件: {rel_path}\n")
    f.write(f"```{lang}\n")
    try:
        with open(full_path, 'r', encoding='utf-8') as cf:
            content = cf.read()
            if args.strip_comments and lang == 'rust':
                lines = content.splitlines()
                new_lines = []
                for line in lines:
                    stripped = line.strip()
                    if stripped.startswith('//') and not stripped.startswith('///'):
                        continue
                    if stripped == '':
                        continue
                    new_lines.append(line)
                content = '\n'.join(new_lines)
            f.write(content)
    except Exception as e:
        f.write(f"读取失败: {e}")
    f.write("\n```\n\n")

def generate_summary(modules_dict, depth):
    """生成摘要文件 summary.md"""
    output_file = output_dir / 'summary.md'
    with open(output_file, 'w', encoding='utf-8') as f:
        f.write("# 项目摘要\n\n")
        f.write(f"## 项目结构 (拆分层级: {depth})\n\n")
        # 收集所有文件用于显示完整树
        all_files = []
        for flist in modules_dict.values():
            all_files.extend(flist)
        write_file_tree(f, all_files, "完整项目结构")
        
        f.write("## 模块列表\n\n")
        for key in sorted(modules_dict.keys()):
            # 获取中文名
            cn_name = MODULE_NAME_MAP.get(key, key)  # 如果没有映射，就用英文
            f.write(f"- **{cn_name}** (`{key}`)：")
            # 简单描述（可以根据 key 添加更多逻辑）
            if key == 'src':
                f.write(" 核心源代码")
            elif key == 'tests':
                f.write(" 集成测试")
            elif key == 'locales':
                f.write(" 本地化文件")
            elif key.startswith('src/'):
                f.write(f" 源代码子模块 `{key}`")
            else:
                f.write(" (暂无描述)")
            f.write("\n")
        
        f.write("\n## 如何获取详细代码\n\n")
        f.write("如果你需要查看某个模块的详细代码，请告诉我模块键名（如 `src/core`、`src/app`），我会生成对应的快照文件。\n")
    print(f"摘要已生成: {output_file}")

def generate_module_snapshot(key, files):
    """生成单个模块的快照文件"""
    cn_name = MODULE_NAME_MAP.get(key, key)
    # 将路径中的 '/' 替换为其他符号，确保文件名安全
    safe_name = cn_name.replace('/', '_')
    output_file = output_dir / f"{safe_name}.md"
    with open(output_file, 'w', encoding='utf-8') as f:
        f.write(f"# 模块: {cn_name}\n\n")
        write_file_tree(f, files, f"{cn_name} 文件树")
        f.write("## 源代码详情\n\n")
        for rel_path in files:
            write_file_contents(f, rel_path)
    print(f"已生成: {output_file}")

# ===== 主流程 =====
print("正在收集文件...")
all_files = collect_files()
depth = args.split_depth
modules = group_by_depth(all_files, depth)

if args.summary:
    generate_summary(modules, depth)
    sys.exit(0)

if args.modules:
    # 生成指定模块（支持多级路径，如 "src/core"）
    module_list = [m.strip() for m in args.modules.split(',') if m.strip()]
    for key in module_list:
        if key in modules:
            generate_module_snapshot(key, modules[key])
        else:
            print(f"警告: 模块 '{key}' 不存在")
else:
    # 生成所有模块
    for key, files in modules.items():
        generate_module_snapshot(key, files)
    # 也生成一个摘要作为参考
    generate_summary(modules, depth)