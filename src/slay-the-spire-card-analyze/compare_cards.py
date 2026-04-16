#!/usr/bin/env python3
"""比较两个 JSON 文件中指定 color 的卡牌差异。用法: python compare_cards.py [ironclad|colorless]"""

import json
import sys
from pathlib import Path

SCRIPT_DIR = Path(__file__).parent
FILE1 = SCRIPT_DIR / "1st-cards.json"
FILE2 = SCRIPT_DIR / "2nd-cards.json"
FILE2_ZH = SCRIPT_DIR / "2nd-cards-zh.json"

COLOR_CONFIG = {
    "ironclad": {
        "colors": ("ironclad", "red"),
        "title": "Ironclad/Red",
        "output": "compare_result.txt",
    },
    "colorless": {
        "colors": ("colorless",),
        "title": "Colorless",
        "output": "compare_result_colorless.txt",
    },
}


def norm_name(s: str) -> str:
    """标准化名称用于比较"""
    return s.strip()


def load_1st(filepath: Path, colors: tuple[str, ...]) -> dict[str, dict]:
    """加载 1st-cards.json，返回 {name: card}"""
    data = json.loads(filepath.read_text(encoding="utf-8"))
    cards = data.get("Cards", data) if isinstance(data, dict) else data
    return {
        norm_name(c["Name"]): c
        for c in cards
        if c.get("Color", "").lower() in colors
    }


def load_2nd(filepath: Path, colors: tuple[str, ...]) -> dict[str, dict]:
    """加载 2nd-cards.json，返回 {name: card}"""
    data = json.loads(filepath.read_text(encoding="utf-8"))
    cards = data if isinstance(data, list) else data.get("Cards", [])
    return {
        norm_name(c["name"]): c
        for c in cards
        if c.get("color", "").lower() in colors
    }


def load_id_to_zh_name(filepath: Path) -> dict[str, str]:
    """加载 2nd-cards-zh.json，返回 {id: 中文名}"""
    data = json.loads(filepath.read_text(encoding="utf-8"))
    cards = data if isinstance(data, list) else data.get("Cards", [])
    return {c["id"]: c["name"] for c in cards if "id" in c and "name" in c}


def main():
    mode = (sys.argv[1] if len(sys.argv) > 1 else "ironclad").lower()
    if mode not in COLOR_CONFIG:
        print(f"用法: python compare_cards.py [ironclad|colorless]")
        sys.exit(1)
    cfg = COLOR_CONFIG[mode]
    colors = cfg["colors"]
    title = cfg["title"]
    output_file = SCRIPT_DIR / cfg["output"]

    cards1 = load_1st(FILE1, colors)
    cards2 = load_2nd(FILE2, colors)
    id_to_zh = load_id_to_zh_name(FILE2_ZH)

    names1 = set(cards1.keys())
    names2 = set(cards2.keys())

    in_both = names1 & names2
    only_1st = names1 - names2
    only_2nd = names2 - names1

    lines = [
        "=" * 60,
        f"{title} 卡牌对比分析",
        "=" * 60,
        f"",
        f"1st-cards.json 中此类卡牌数量: {len(cards1)}",
        f"2nd-cards.json 中此类卡牌数量: {len(cards2)}",
        f"",
        f"两个文件共有卡牌数量: {len(in_both)}",
        f"仅在 1st 中出现: {len(only_1st)}",
        f"仅在 2nd 中出现: {len(only_2nd)}",
        "",
        "--- 两个文件都有的卡牌 ---",
    ]
    lines.extend(f"  {n}" for n in sorted(in_both))
    lines.extend([
        "",
        "--- 仅在 1st-cards.json 中的卡牌 ---",
    ])
    lines.extend(f"  {n}" for n in sorted(only_1st))
    lines.extend([
        "",
        "--- 仅在 2nd-cards.json 中的卡牌（含中文名） ---",
    ])
    for n in sorted(only_2nd):
        card = cards2.get(n, {})
        zh = id_to_zh.get(card.get("id", ""), "（未找到）")
        lines.append(f"  {n} / {zh}")

    content = "\n".join(lines)
    output_file.write_text(content, encoding="utf-8")
    print(content)
    print(f"\n结果已保存到 {output_file}")


if __name__ == "__main__":
    main()
