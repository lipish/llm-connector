#!/usr/bin/env python3
"""
批量修复示例文件中的 Message 构造
将 Message { role: Role::User, content: "...".to_string(), ... }
改为 Message::text(Role::User, "...")
"""

import re
import sys
from pathlib import Path

def fix_message_construction(content):
    """修复 Message 构造"""
    
    # 模式 1: Message { role: Role::User, content: "...".to_string(), ..Default::default() }
    pattern1 = r'Message\s*\{\s*role:\s*Role::(\w+),\s*content:\s*"([^"]+)"\.to_string\(\),\s*\.\.Default::default\(\)\s*\}'
    replacement1 = r'Message::text(Role::\1, "\2")'
    content = re.sub(pattern1, replacement1, content)
    
    # 模式 2: Message { role: Role::User, content: variable.to_string(), ..Default::default() }
    pattern2 = r'Message\s*\{\s*role:\s*Role::(\w+),\s*content:\s*(\w+)\.to_string\(\),\s*\.\.Default::default\(\)\s*\}'
    replacement2 = r'Message::text(Role::\1, \2)'
    content = re.sub(pattern2, replacement2, content)
    
    # 模式 3: Message { role: Role::User, content: "...".to_string(), name: None, ... }
    pattern3 = r'Message\s*\{\s*role:\s*Role::(\w+),\s*content:\s*"([^"]+)"\.to_string\(\),\s*name:\s*None,\s*tool_calls:\s*None,\s*tool_call_id:\s*None,\s*reasoning_content:\s*None,\s*reasoning:\s*None,\s*thought:\s*None,\s*thinking:\s*None,\s*\}'
    replacement3 = r'Message::text(Role::\1, "\2")'
    content = re.sub(pattern3, replacement3, content)
    
    return content

def main():
    examples_dir = Path("examples")
    
    if not examples_dir.exists():
        print("❌ examples 目录不存在")
        sys.exit(1)
    
    print("🔧 批量修复 Message 构造...")
    print()
    
    fixed_count = 0
    
    for rs_file in examples_dir.glob("*.rs"):
        content = rs_file.read_text()
        new_content = fix_message_construction(content)
        
        if content != new_content:
            rs_file.write_text(new_content)
            print(f"  ✅ {rs_file.name}")
            fixed_count += 1
    
    print()
    print(f"✅ 修复完成！共修复 {fixed_count} 个文件")

if __name__ == "__main__":
    main()

