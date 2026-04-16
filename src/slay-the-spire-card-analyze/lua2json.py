#!/usr/bin/env python3
"""将 Slay the Spire 卡片 Lua 数据转换为 JSON"""

import json
import re
import sys
from pathlib import Path


def extract_cards_content(text: str) -> str:
    """提取 ["Cards"] = { ... } 之间的内容"""
    match = re.search(r'\[\s*["\']Cards["\']\s*\]\s*=\s*\{', text)
    if not match:
        raise ValueError("未找到 Cards 表")
    start = match.end()
    depth = 1
    i = start
    while i < len(text) and depth > 0:
        c = text[i]
        if c == "{" and (i == 0 or text[i - 1] != "\\"):
            depth += 1
        elif c == "}" and (i == 0 or text[i - 1] != "\\"):
            depth -= 1
        i += 1
    return text[start : i - 1]


def parse_lua_value(s: str):
    """解析 Lua 值"""
    s = s.strip()
    if not s:
        return None
    if s == "{}":
        return []
    if s.startswith('{"') or s.startswith("{'"):
        # 数组 {"a","b"} 或 {'a','b'}
        parts = []
        in_str = False
        quote = None
        current = ""
        i = 1
        while i < len(s) - 1:
            c = s[i]
            if not in_str:
                if c in '"\'':
                    in_str = True
                    quote = c
                    current = ""
                    i += 1
                    continue
                i += 1
                continue
            if c == "\\" and i + 1 < len(s):
                current += s[i : i + 2]
                i += 2
                continue
            if c == quote:
                parts.append(current.replace('\\"', '"').replace("\\'", "'"))
                in_str = False
                i += 1
                continue
            current += c
            i += 1
        return parts
    if s.startswith('"') or s.startswith("'"):
        quote = s[0]
        content = s[1:-1].replace('\\' + quote, quote).replace("\\n", "\n")
        return content
    if re.match(r"^-?\d+$", s):
        return int(s)
    # 标识符 X, Unplayable 等
    return s


def parse_card_block(block: str) -> dict:
    """解析单个卡牌块"""
    card = {}
    i = 0
    while i < len(block):
        # 跳过空白和逗号
        while i < len(block) and block[i] in " \t\n\r,":
            i += 1
        if i >= len(block):
            break
        # 匹配 Key = value
        key_match = re.match(r"([A-Za-z_][A-Za-z0-9_]*)\s*=\s*", block[i:])
        if not key_match:
            i += 1
            continue
        key = key_match.group(1)
        i += key_match.end()
        # 提取 value
        value_str = ""
        depth = 0
        in_str = False
        str_char = None
        start = i
        while i < len(block):
            c = block[i]
            if not in_str:
                if c in '"\'':
                    in_str = True
                    str_char = c
                    value_str += c
                    i += 1
                    continue
                if c == "{":
                    depth += 1
                elif c == "}":
                    depth -= 1
                elif c == "," and depth == 0:
                    i += 1
                    break
                elif c == "\n" and depth == 0:
                    i += 1
                    break
                value_str += c
                i += 1
                continue
            if c == "\\":
                value_str += c
                if i + 1 < len(block):
                    value_str += block[i + 1]
                    i += 2
                continue
            if c == str_char:
                in_str = False
            value_str += c
            i += 1
        value_str = value_str.strip().rstrip(",")
        card[key] = parse_lua_value(value_str)
    return card


def split_card_blocks(content: str) -> list[str]:
    """按卡牌块分割"""
    cards = []
    i = 0
    while i < len(content):
        # 找 {Name =
        match = re.search(r"\{\s*Name\s*=", content[i:])
        if not match:
            break
        start = i + match.start()
        i += match.end()
        # 匹配到对应 }
        depth = 1
        while i < len(content) and depth > 0:
            c = content[i]
            if c == "{" and (i == 0 or content[i - 1] != "\\"):
                depth += 1
            elif c == "}" and (i == 0 or content[i - 1] != "\\"):
                depth -= 1
            i += 1
        block = content[start:i]
        cards.append(block)
    return cards


def main():
    script_dir = Path(__file__).parent
    lua_path = script_dir / "1st.lua"
    json_path = script_dir / "cards.json"

    if len(sys.argv) > 1:
        lua_path = Path(sys.argv[1])
    if len(sys.argv) > 2:
        json_path = Path(sys.argv[2])

    text = lua_path.read_text(encoding="utf-8")
    cards_content = extract_cards_content(text)
    blocks = split_card_blocks(cards_content)
    cards = [parse_card_block(b) for b in blocks]
    json_path.write_text(json.dumps({"Cards": cards}, ensure_ascii=False, indent=2), encoding="utf-8")
    print(f"已转换 {len(cards)} 张卡牌 -> {json_path}")


if __name__ == "__main__":
    main()
